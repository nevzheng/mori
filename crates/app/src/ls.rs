//! `mori ls`: every repo mori manages and every tree in it, matched against what jj reports.
//! Reads only; it never snapshots a working copy.

use std::collections::{BTreeMap, BTreeSet};

use mori_api::v1alpha1::{
    Disk, ListTreesResponse, PushedBookmark, RepoTrees, Tree, TreeRow, TreeState, UnmanagedRepo,
    tree_row::Status,
};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, clone_path};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::forest::{Entry, Workspace, reconcile};
use mori_core::paths::Paths;
use mori_core::tree::Role;
use mori_core::vcs::Forge;
use mori_store::database::Database;
use mori_store::records::{RepoRecord, TreeRecord};

use crate::disk;
use crate::landing;
use crate::state::{self, boxed};
use crate::{App, Backend};

/// What `mori ls` was asked for.
#[derive(Clone, Debug, Default)]
pub struct LsArgs {
    /// Only this repo.
    pub repo: Option<String>,
    /// Measure each tree's size (`--size`).
    pub sizes: bool,
    /// Measure again instead of reusing recent sizes (`--fresh`).
    pub fresh: bool,
}

/// Runs `ls` for every repo mori manages, or only `repo`.
///
/// # Errors
///
/// The plan's refusal, or a failure of the disk, the database or an adapter, with its code and
/// reason.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    args: &LsArgs,
) -> Result<ListTreesResponse, Box<dyn ErrorDetails>> {
    let repo = args.repo.as_deref();
    let paths = state::paths(&app.host)?;
    let mut db = state::open_database(&paths)?;
    let mut records = db.repos().map_err(boxed)?;
    if let Some(repo) = repo {
        let wanted = state::repo_id(&db, repo)?.to_string();
        records.retain(|record| record.remote == wanted);
        if records.is_empty() {
            return Err(boxed(RepoError::NotManaged { repo: wanted }));
        }
    }
    let managed: BTreeSet<String> = records.iter().map(|repo| repo.remote.clone()).collect();
    let mut repos: Vec<RepoTrees> = records
        .iter()
        .map(|repo| list_repo(&paths, &mut db, &app.vcs, repo))
        .collect::<Result<_, _>>()?;
    if args.sizes {
        for row in repos.iter_mut().flat_map(|repo| repo.trees.iter_mut()) {
            add_size(&mut db, row, app.host.now, args.fresh);
        }
    }
    let space = disk::space(&paths);
    // Every repo was measured, so a repo absent here (when one repo was asked for) is unknown.
    let sizes: Option<BTreeMap<String, u64>> = args.sizes.then(|| {
        repos
            .iter()
            .map(|repo| {
                (
                    repo.repo.clone(),
                    repo.trees.iter().map(|row| row.size_bytes).sum(),
                )
            })
            .collect()
    });
    let unmanaged_repos = if repo.is_some() {
        Vec::new()
    } else {
        let clones = mori_store::init::find_clones(&paths.repos()).map_err(boxed)?;
        mori_core::init::unmanaged(&paths, &clones, &managed)
            .into_iter()
            .map(|repo| UnmanagedRepo {
                repo: repo.repo,
                path: repo.path.display().to_string(),
            })
            .collect()
    };
    Ok(ListTreesResponse {
        repos,
        unmanaged_repos,
        disk: space.map(|space| Disk {
            free_bytes: space.free,
            total_bytes: space.total,
        }),
        warnings: disk::warnings(&paths, &db, space, sizes.as_ref()),
    })
}

/// Measures a tree that exists on disk, or reuses its recent size.
fn add_size(db: &mut Database, row: &mut TreeRow, now: u64, fresh: bool) {
    if row.status() == Status::Missing {
        return;
    }
    let Some(tree) = &row.tree else {
        return;
    };
    let size = disk::tree_size(db, &tree.id, std::path::Path::new(&tree.path), now, fresh);
    row.size_bytes = size.bytes;
    row.size_partial = size.partial;
    row.size_measured_at = disk::measured_at(&size);
}

fn list_repo(
    paths: &Paths,
    db: &mut Database,
    vcs: &impl Backend,
    repo: &RepoRecord,
) -> Result<RepoTrees, Box<dyn ErrorDetails>> {
    let id = CloneUrl::parse(&repo.remote).map_err(boxed)?.repo;
    let clone = clone_path(paths, &id);
    let records = db.trees(&repo.id).map_err(boxed)?;
    // A clone that is gone is a mismatch like any other: its trees are all missing.
    let workspaces = if clone.exists() {
        vcs.list(&clone).map_err(boxed)?
    } else {
        Vec::new()
    };
    let mut trees = Vec::new();
    for entry in reconcile(records, workspaces, |record| record.name.as_str()) {
        // A task tree's landed work counts as saved, so a squash-merged tree isn't shown as
        // unpushed; looking also records the bookmarks it pushed, for when the remote deletes
        // them.
        let (landed, seen) = match &entry {
            Entry::Tree { record, workspace } if Role::of(&record.name) == Role::Task => {
                let (landing, seen) = landing::observe_each(
                    db,
                    vcs,
                    None,
                    &landing::TreeRef {
                        repo: &id,
                        clone: &clone,
                        id: &record.id,
                        name: &workspace.name,
                    },
                )?;
                (landing.commits, seen)
            }
            _ => (Vec::new(), Vec::new()),
        };
        let mut tree = row(paths, vcs, repo, &clone, entry, &landed)?;
        tree.bookmarks = seen
            .into_iter()
            .map(|seen| PushedBookmark {
                remote: seen.remote,
                name: seen.bookmark,
                commit_id: seen.commit_id,
                on_remote: seen.on_remote,
                landed: seen.landed,
            })
            .collect();
        trees.push(tree);
    }
    Ok(RepoTrees {
        repo: repo.remote.clone(),
        path: clone.display().to_string(),
        trees,
        vcs: if clone.exists() {
            crate::api_vcs(crate::routed::kind_of(&clone)).into()
        } else {
            mori_api::v1alpha1::Vcs::Unspecified.into()
        },
    })
}

fn row(
    paths: &Paths,
    vcs: &impl Backend,
    repo: &RepoRecord,
    clone: &std::path::Path,
    entry: Entry<TreeRecord>,
    landed: &[String],
) -> Result<TreeRow, Box<dyn ErrorDetails>> {
    let state = |workspace: &Workspace| {
        vcs.state_covering(clone, &workspace.name, landed)
            .map(|state| TreeState {
                change: state.change,
                changed: state.changed,
                unpushed: state.unpushed,
            })
            .map_err(boxed)
    };
    Ok(match entry {
        Entry::Tree { record, workspace } => TreeRow {
            state: Some(state(&workspace)?),
            tree: Some(recorded_tree(
                repo,
                record,
                workspace.root.display().to_string(),
            )),
            status: Status::Tree.into(),
            bookmarks: Vec::new(),
            ..TreeRow::default()
        },
        Entry::Missing { record } => {
            let path = if record.name == BASE_TREE_NAME {
                clone.to_path_buf()
            } else {
                paths.trees().join(&repo.dir_name).join(&record.name)
            };
            TreeRow {
                tree: Some(recorded_tree(repo, record, path.display().to_string())),
                status: Status::Missing.into(),
                bookmarks: Vec::new(),
                state: None,
                ..TreeRow::default()
            }
        }
        Entry::Foreign { workspace } => TreeRow {
            state: Some(state(&workspace)?),
            tree: Some(Tree {
                repo: repo.remote.clone(),
                name: workspace.name,
                path: workspace.root.display().to_string(),
                ..Tree::default()
            }),
            status: Status::Foreign.into(),
            bookmarks: Vec::new(),
            ..TreeRow::default()
        },
    })
}

fn recorded_tree(repo: &RepoRecord, record: TreeRecord, path: String) -> Tree {
    Tree {
        id: record.id,
        repo: repo.remote.clone(),
        name: record.name,
        path,
        owner: record.owner,
        task: record.task.unwrap_or_default(),
        lifetime: record.lifetime,
        purpose: record.purpose.unwrap_or_default(),
    }
}

//! `mori ls`: every repo mori manages and every tree in it, matched against what jj reports.
//! Reads only; it never snapshots a working copy.

use std::collections::BTreeSet;

use mori_api::v1alpha1::{
    ListTreesResponse, RepoTrees, Tree, TreeRow, TreeState, UnmanagedRepo, tree_row::Status,
};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, clone_path};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::forest::{Entry, Workspace, Workspaces, reconcile};
use mori_core::paths::Paths;
use mori_core::tree::Role;
use mori_jj::JjCli;
use mori_store::database::Database;
use mori_store::records::{RepoRecord, TreeRecord};

use crate::landing;
use crate::state::{self, boxed};

/// Runs `ls` for every repo mori manages, or only `repo`.
pub fn run(repo: Option<&str>) -> Result<ListTreesResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths()?;
    let mut db = state::open_database(&paths)?;
    let jj = JjCli::from_path();
    let mut records = db.repos().map_err(boxed)?;
    if let Some(repo) = repo {
        let wanted = CloneUrl::parse(repo).map_err(boxed)?.repo.to_string();
        records.retain(|record| record.remote == wanted);
        if records.is_empty() {
            return Err(boxed(RepoError::NotManaged { repo: wanted }));
        }
    }
    let managed: BTreeSet<String> = records.iter().map(|repo| repo.remote.clone()).collect();
    let repos = records
        .iter()
        .map(|repo| list_repo(&paths, &mut db, &jj, repo))
        .collect::<Result<_, _>>()?;
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
    })
}

fn list_repo(
    paths: &Paths,
    db: &mut Database,
    jj: &JjCli,
    repo: &RepoRecord,
) -> Result<RepoTrees, Box<dyn ErrorDetails>> {
    let id = CloneUrl::parse(&repo.remote).map_err(boxed)?.repo;
    let clone = clone_path(paths, &id);
    let records = db.trees(&repo.id).map_err(boxed)?;
    // A clone that is gone is a mismatch like any other: its trees are all missing.
    let workspaces = if clone.exists() {
        jj.list(&clone).map_err(boxed)?
    } else {
        Vec::new()
    };
    let mut trees = Vec::new();
    for entry in reconcile(records, workspaces, |record| record.name.as_str()) {
        // A task tree's landed work counts as saved, so a squash-merged tree isn't shown as
        // unpushed; looking also records the bookmarks it pushed, for when the remote deletes
        // them.
        let landed = match &entry {
            Entry::Tree { record, workspace } if record.role == Role::Task.as_str() => {
                landing::observe(
                    db,
                    jj,
                    None,
                    &landing::TreeRef {
                        repo: &id,
                        clone: &clone,
                        id: &record.id,
                        name: &workspace.name,
                    },
                )?
                .commits
            }
            _ => Vec::new(),
        };
        trees.push(row(paths, jj, repo, &clone, entry, &landed)?);
    }
    Ok(RepoTrees {
        repo: repo.remote.clone(),
        path: clone.display().to_string(),
        trees,
    })
}

fn row(
    paths: &Paths,
    jj: &JjCli,
    repo: &RepoRecord,
    clone: &std::path::Path,
    entry: Entry<TreeRecord>,
    landed: &[String],
) -> Result<TreeRow, Box<dyn ErrorDetails>> {
    let state = |workspace: &Workspace| {
        jj.state_covering(clone, &workspace.name, landed)
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
                state: None,
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
        },
    })
}

fn recorded_tree(repo: &RepoRecord, record: TreeRecord, path: String) -> Tree {
    Tree {
        id: record.id,
        repo: repo.remote.clone(),
        name: record.name,
        path,
        role: record.role,
        owner: record.owner,
        task: record.task.unwrap_or_default(),
        lifetime: record.lifetime,
    }
}

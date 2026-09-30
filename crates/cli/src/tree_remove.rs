//! `mori tree remove`: check the tree is safe to remove, then forget its workspace, delete its
//! directory and drop its record, in that order.

use mori_api::v1alpha1::{RemoveTreeResponse, Tree};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, clone_path};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::forest::Workspaces;
use mori_core::tree::{Lifetime, Role};
use mori_core::tree_remove::{Observed, Recorded, RemovePlan, Request, plan};
use mori_jj::JjCli;
use mori_store::StoreError;
use mori_store::records::TreeRecord;

use crate::state::{self, boxed};
use crate::{landing, tree};

/// What `mori tree remove` was asked for.
pub struct RemoveArgs {
    pub repo: String,
    pub name: String,
    pub agent: Option<String>,
    pub pinned: bool,
    pub dry_run: bool,
}

/// Runs `tree remove` against the real environment, disk and `jj` on `PATH`.
pub fn run(args: RemoveArgs) -> Result<RemoveTreeResponse, Box<dyn ErrorDetails>> {
    let repo_id = CloneUrl::parse(&args.repo).map_err(boxed)?.repo;
    let request = Request {
        name: args.name,
        owner: tree::owner(args.agent)?,
        pinned_ok: args.pinned,
    };
    let paths = state::paths()?;
    let mut db = state::open_database(&paths)?;
    let jj = JjCli::from_path();
    let repo = db
        .repo(&repo_id.to_string())
        .map_err(boxed)?
        .ok_or_else(|| {
            boxed(RepoError::NotManaged {
                repo: repo_id.to_string(),
            })
        })?;
    let clone = clone_path(&paths, &repo_id);
    let record = db
        .trees(&repo.id)
        .map_err(boxed)?
        .into_iter()
        .find(|tree| tree.name == request.name);
    let recorded = record.as_ref().map(|record| Recorded {
        name: record.name.clone(),
        role: if record.role == Role::Base.as_str() {
            Role::Base
        } else {
            Role::Task
        },
        owner: record.owner.clone(),
        // An unreadable lifetime counts as pinned: the choice that refuses.
        lifetime: record.lifetime.parse().unwrap_or(Lifetime::Pinned),
        path: if record.name == BASE_TREE_NAME {
            clone.clone()
        } else {
            paths.trees().join(&repo.dir_name).join(&record.name)
        },
    });
    let has_workspace = jj
        .list(&clone)
        .map_err(boxed)?
        .iter()
        .any(|workspace| workspace.name == request.name);
    // Work that landed (a recorded bookmark squash-merged and deleted on the remote) counts as
    // saved, as well as work on the remote. Recording the bookmarks seen now keeps that knowable
    // after the remote deletes them.
    let landed = match (&record, has_workspace) {
        (Some(record), true) => {
            let policy = state::tree_policy(&paths)?.landed.when;
            landing::observe(
                &mut db,
                &jj,
                None,
                &landing::TreeRef {
                    repo: &repo_id,
                    clone: &clone,
                    id: &record.id,
                    name: &request.name,
                },
                &policy,
            )?
            .commits
        }
        _ => Vec::new(),
    };
    // First decide from what jj last saw, so a tree that can't go anyway (someone else's, pinned,
    // the clone itself, foreign) is never snapshotted. Then snapshot and decide again, so edits
    // since jj last ran in the tree count.
    let read_state = || -> Result<_, Box<dyn ErrorDetails>> {
        if has_workspace {
            Ok(Some(
                jj.state_covering(&clone, &request.name, &landed)
                    .map_err(boxed)?,
            ))
        } else {
            Ok(None)
        }
    };
    let mut observed = Observed {
        recorded,
        state: read_state()?,
    };
    plan(&request, &observed).map_err(boxed)?;
    if let (Some(recorded), true) = (&observed.recorded, has_workspace) {
        jj.snapshot(&recorded.path).map_err(boxed)?;
        observed.state = read_state()?;
    }
    let plan = plan(&request, &observed).map_err(boxed)?;
    let mut response = RemoveTreeResponse {
        tree: record.map(|record| tree_message(&repo.remote, &plan, record)),
        validate_only: args.dry_run,
        ..RemoveTreeResponse::default()
    };
    if args.dry_run {
        return Ok(response);
    }
    remove(&jj, &clone, &plan, &mut response)?;
    db.delete_tree(&repo.id, &plan.name).map_err(boxed)?;
    Ok(response)
}

/// Forgets the workspace, then deletes the directory. A missing tree's directory is left alone:
/// with no workspace, there is no way to check what is in it.
fn remove(
    jj: &JjCli,
    clone: &std::path::Path,
    plan: &RemovePlan,
    response: &mut RemoveTreeResponse,
) -> Result<(), Box<dyn ErrorDetails>> {
    if !plan.forget_workspace {
        return Ok(());
    }
    jj.forget_workspace(clone, &plan.name).map_err(boxed)?;
    response.workspace_forgotten = true;
    std::fs::remove_dir_all(&plan.path).map_err(|source| {
        boxed(StoreError::Io {
            path: plan.path.clone(),
            source,
        })
    })?;
    response.directory_removed = true;
    Ok(())
}

fn tree_message(repo: &str, plan: &RemovePlan, record: TreeRecord) -> Tree {
    Tree {
        id: record.id,
        repo: repo.to_owned(),
        name: record.name,
        path: plan.path.display().to_string(),
        role: record.role,
        owner: record.owner,
        task: record.task.unwrap_or_default(),
        lifetime: record.lifetime,
    }
}

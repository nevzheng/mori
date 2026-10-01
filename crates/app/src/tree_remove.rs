//! `mori tree remove`: check the tree is safe to remove, then forget its workspace, delete its
//! directory and drop its record, in that order. A tree whose directory was deleted by hand goes
//! the way `mori gc --apply` removes one: pinned and journalled, so `mori restore` brings it back.

use mori_api::v1alpha1::{RemoveTreeResponse, Tree};
use mori_core::clone::{BASE_TREE_NAME, clone_path};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::paths::Paths;
use mori_core::tree::{Lifetime, Role};
use mori_core::tree_remove::{Observed, Recorded, RemovePlan, Request, plan};
use mori_core::vcs::Forge;
use mori_store::database::Database;
use mori_store::gc::new_id;
use mori_store::records::{RepoRecord, TreeRecord};

use crate::gc::Context;
use crate::state::{self, boxed};
use crate::{App, Backend};
use crate::{gc_apply, landing};

/// What `mori tree remove` was asked for.
pub struct RemoveArgs {
    pub repo: String,
    pub name: String,
    pub pinned: bool,
    pub dry_run: bool,
}

/// Runs `tree remove`: check, then forget, delete and drop.
///
/// # Errors
///
/// The plan's refusal, or a failure of the disk, the database or an adapter, with its code and
/// reason.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    args: RemoveArgs,
) -> Result<RemoveTreeResponse, Box<dyn ErrorDetails>> {
    let request = Request {
        name: args.name,
        pinned_ok: args.pinned,
    };
    let paths = state::paths(&app.host)?;
    let mut db = state::open_database(&paths)?;
    let repo_id = state::repo_id(&db, &args.repo)?;
    let vcs = &app.vcs;
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
        role: Role::of(&record.name),
        // An unreadable lifetime counts as pinned: the choice that refuses.
        lifetime: record.lifetime.parse().unwrap_or(Lifetime::Pinned),
        path: if record.name == BASE_TREE_NAME {
            clone.clone()
        } else {
            paths.trees().join(&repo.dir_name).join(&record.name)
        },
    });
    let workspace = vcs
        .list(&clone)
        .map_err(boxed)?
        .into_iter()
        .find(|workspace| workspace.name == request.name);
    // Nothing can be read or snapshotted in a directory that is gone: neither where mori put it
    // nor where the VCS says it is (jj reports no root once it is deleted).
    let dir_gone = workspace.as_ref().is_some_and(|workspace| {
        !workspace.root.exists()
            && recorded
                .as_ref()
                .is_some_and(|recorded| !recorded.path.exists())
    });
    let has_workspace = workspace.is_some() && !dir_gone;
    // Work that landed (a recorded bookmark squash-merged and deleted on the remote) counts as
    // saved, as well as work on the remote. Recording the bookmarks seen now keeps that knowable
    // after the remote deletes them.
    let landed = match (&record, has_workspace) {
        (Some(record), true) => {
            landing::observe(
                &mut db,
                vcs,
                None,
                &landing::TreeRef {
                    repo: &repo_id,
                    clone: &clone,
                    id: &record.id,
                    name: &request.name,
                },
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
                vcs.state_covering(&clone, &request.name, &landed)
                    .map_err(boxed)?,
            ))
        } else {
            Ok(None)
        }
    };
    let mut observed = Observed {
        recorded,
        state: read_state()?,
        dir_gone,
    };
    plan(&request, &observed).map_err(boxed)?;
    if let (Some(recorded), true) = (&observed.recorded, has_workspace) {
        vcs.snapshot(&recorded.path).map_err(boxed)?;
        observed.state = read_state()?;
    }
    let plan = plan(&request, &observed).map_err(boxed)?;
    let mut response = RemoveTreeResponse {
        tree: record.map(|record| tree_message(&repo.remote, &plan, record)),
        validate_only: args.dry_run,
        directory_gone: plan.pin,
        ..RemoveTreeResponse::default()
    };
    if args.dry_run {
        return Ok(response);
    }
    if plan.pin {
        pin_and_remove(app, &paths, &mut db, &repo, &plan, &mut response)?;
        return Ok(response);
    }
    remove(vcs, &clone, &plan, &mut response)?;
    db.delete_tree(&repo.id, &plan.name).map_err(boxed)?;
    Ok(response)
}

/// Removes a tree whose directory is gone as `mori gc --apply` would: pins its last commit, forgets
/// the workspace, drops the record and journals it.
fn pin_and_remove<V: Backend, F: Forge>(
    app: &App<V, F>,
    paths: &Paths,
    db: &mut Database,
    repo: &RepoRecord,
    plan: &RemovePlan,
    response: &mut RemoveTreeResponse,
) -> Result<(), Box<dyn ErrorDetails>> {
    let context = Context {
        paths,
        vcs: &app.vcs,
        forge: None,
        now: app.host.now,
        lru_max: None,
    };
    let Some(record) = db
        .trees(&repo.id)
        .map_err(boxed)?
        .into_iter()
        .find(|tree| tree.name == plan.name)
    else {
        return Ok(());
    };
    let entry_id = new_id("j");
    let entry = gc_apply::remove(&context, db, repo, &record, true, &entry_id)?;
    mori_store::gc::append_journal(paths, &entry).map_err(boxed)?;
    response.workspace_forgotten = true;
    response.journal_entry = entry_id;
    Ok(())
}

/// Forgets the workspace, then deletes the directory. A missing tree's directory is left alone:
/// with no workspace, there is no way to check what is in it.
fn remove(
    vcs: &impl Backend,
    clone: &std::path::Path,
    plan: &RemovePlan,
    response: &mut RemoveTreeResponse,
) -> Result<(), Box<dyn ErrorDetails>> {
    if !plan.forget_workspace {
        return Ok(());
    }
    vcs.forget_tree(clone, &plan.name).map_err(boxed)?;
    response.workspace_forgotten = true;
    state::remove_tree_dir(&plan.path)?;
    response.directory_removed = true;
    Ok(())
}

fn tree_message(repo: &str, plan: &RemovePlan, record: TreeRecord) -> Tree {
    Tree {
        id: record.id,
        repo: repo.to_owned(),
        name: record.name,
        path: plan.path.display().to_string(),
        owner: record.owner,
        task: record.task.unwrap_or_default(),
        lifetime: record.lifetime,
    }
}

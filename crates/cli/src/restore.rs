//! `mori restore`: bring back a tree `mori gc apply` removed, from its journal entry.

use mori_api::v1alpha1::{RestoreResponse, Tree};
use mori_core::clone::{CloneUrl, clone_path};
use mori_core::error::{CleanupError, ErrorDetails, RepoError, TreeError};
use mori_core::forest::Workspaces;
use mori_jj::JjCli;
use mori_store::records::NewTree;

use crate::state::{self, boxed};

/// Runs `mori restore`.
pub fn run(entry_id: &str) -> Result<RestoreResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths()?;
    let mut db = state::open_database(&paths)?;
    let jj = JjCli::from_path();
    let entry = mori_store::gc::read_journal(&paths)
        .map_err(boxed)?
        .into_iter()
        .find(|entry| entry.id == entry_id)
        .ok_or_else(|| {
            boxed(CleanupError::EntryNotFound {
                id: entry_id.to_owned(),
            })
        })?;
    if entry.commit_id.is_empty() {
        return Err(boxed(CleanupError::NothingToRestore { id: entry.id }));
    }
    let repo_id = CloneUrl::parse(&entry.repo).map_err(boxed)?.repo;
    let repo = db.repo(&entry.repo).map_err(boxed)?.ok_or_else(|| {
        boxed(RepoError::NotManaged {
            repo: entry.repo.clone(),
        })
    })?;
    let clone = clone_path(&paths, &repo_id);
    let path = paths.trees().join(&repo.dir_name).join(&entry.name);
    // Refuse before touching anything: the name and the path must be free again.
    if db
        .trees(&repo.id)
        .map_err(boxed)?
        .iter()
        .any(|tree| tree.name == entry.name)
    {
        return Err(boxed(TreeError::TreeExists { name: entry.name }));
    }
    if jj
        .list(&clone)
        .map_err(boxed)?
        .iter()
        .any(|workspace| workspace.name == entry.name)
    {
        return Err(boxed(TreeError::WorkspaceExists { name: entry.name }));
    }
    if path.symlink_metadata().is_ok() {
        return Err(boxed(TreeError::PathExists { path }));
    }
    if !jj.commit_exists(&clone, &entry.commit_id).map_err(boxed)? {
        return Err(boxed(CleanupError::CommitGone {
            id: entry.id,
            commit: entry.commit_id,
        }));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| {
            boxed(mori_store::StoreError::Io {
                path: parent.to_path_buf(),
                source,
            })
        })?;
    }
    jj.add_workspace_at(&clone, &entry.name, &path, &entry.commit_id)
        .map_err(boxed)?;
    let restored = db
        .restore_tree(
            &repo.id,
            &entry.tree_id,
            &NewTree {
                name: &entry.name,
                role: "task",
                owner: &entry.owner,
                task: entry.task.as_deref(),
                lifetime: &entry.lifetime,
            },
        )
        .map_err(|error| {
            boxed(TreeError::NotRecorded {
                name: entry.name.clone(),
                path: path.clone(),
                why: error.to_string(),
            })
        })?;
    for bookmark in &entry.bookmarks {
        // Best effort: the tree is back; its bookmarks only help decide later whether it landed.
        let _ = db.record_tree_bookmark(
            &restored.id,
            &bookmark.remote,
            &bookmark.bookmark,
            &bookmark.commit_id,
        );
    }
    Ok(RestoreResponse {
        tree: Some(Tree {
            id: restored.id,
            repo: entry.repo,
            name: restored.name,
            path: path.display().to_string(),
            role: restored.role,
            owner: restored.owner,
            task: restored.task.unwrap_or_default(),
            lifetime: restored.lifetime,
        }),
        commit_id: entry.commit_id,
    })
}

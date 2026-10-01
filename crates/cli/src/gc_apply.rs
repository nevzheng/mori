//! `mori gc --apply`: remove one tree from a confirmed batch, checking it again first.
//!
//! Per tree: snapshot, judge again with the report's own rules, and act only if it still may go
//! and is safe. Removing pins the working-copy commit (`refs/mori/removed/<entry>`), forgets the
//! workspace, deletes the directory, drops the record and journals it, in that order.

use mori_api::v1alpha1::gc_item::{Class as ItemClass, Outcome};
use mori_core::clone::{CloneUrl, clone_path};
use mori_core::error::ErrorDetails;
use mori_core::forest::Workspaces;
use mori_store::StoreError;
use mori_store::database::Database;
use mori_store::gc::{JournalBookmark, JournalEntry, new_id, now};
use mori_store::records::{RepoRecord, TreeRecord};

use crate::gc::{self, Context};
use crate::state::boxed;

/// Checks one tree again and, if it still may go and is safe, removes it.
pub fn apply_one(
    context: &Context,
    db: &mut Database,
    repo_name: &str,
    tree_name: &str,
    dry_run: bool,
) -> Result<(Outcome, String, String), Box<dyn ErrorDetails>> {
    let changed = |why: &str| Ok((Outcome::SkippedChanged, why.to_owned(), String::new()));
    let Some(repo) = db.repo(repo_name).map_err(boxed)? else {
        return changed("GONE");
    };
    let id = CloneUrl::parse(&repo.remote).map_err(boxed)?.repo;
    let clone = clone_path(context.paths, &id);
    let Some(record) = db
        .trees(&repo.id)
        .map_err(boxed)?
        .into_iter()
        .find(|tree| tree.name == tree_name)
    else {
        return changed("GONE");
    };
    let expected = context
        .paths
        .trees()
        .join(&repo.dir_name)
        .join(&record.name);
    let workspace = if clone.exists() {
        context
            .jj
            .list(&clone)
            .map_err(boxed)?
            .into_iter()
            .find(|workspace| workspace.name == record.name)
    } else {
        None
    };
    // mori only ever deletes a tree where it put it.
    if let Some(workspace) = &workspace {
        if !same_path(&workspace.root, &expected) {
            return changed("MOVED");
        }
        context.jj.snapshot(&workspace.root).map_err(boxed)?;
    }
    let judged = gc::report_repo(context, db, &repo, true)?
        .into_iter()
        .find(|judged| judged.name == record.name);
    let Some(judged) = judged else {
        return changed("GONE");
    };
    match judged.class() {
        ItemClass::Remove => {}
        ItemClass::Blocked => return Ok((Outcome::SkippedUnsaved, judged.reason, String::new())),
        _ => return Ok((Outcome::SkippedChanged, judged.reason, String::new())),
    }
    if dry_run {
        return Ok((Outcome::WouldRemove, judged.reason, String::new()));
    }
    let entry_id = new_id("j");
    let entry = remove(context, db, &repo, &record, workspace.is_some(), &entry_id)?;
    mori_store::gc::append_journal(context.paths, &entry).map_err(boxed)?;
    Ok((Outcome::Removed, judged.reason, entry_id))
}

/// Pins, forgets, deletes and drops one tree, and returns its journal entry.
fn remove(
    context: &Context,
    db: &mut Database,
    repo: &RepoRecord,
    record: &TreeRecord,
    has_workspace: bool,
    entry_id: &str,
) -> Result<JournalEntry, Box<dyn ErrorDetails>> {
    let id = CloneUrl::parse(&repo.remote).map_err(boxed)?.repo;
    let clone = clone_path(context.paths, &id);
    let path = context
        .paths
        .trees()
        .join(&repo.dir_name)
        .join(&record.name);
    let bookmarks = db
        .tree_bookmarks(&record.id)
        .map_err(boxed)?
        .into_iter()
        .map(|seen| JournalBookmark {
            remote: seen.remote,
            bookmark: seen.bookmark,
            commit_id: seen.commit_id,
        })
        .collect();
    let (commit_id, pin) = if has_workspace {
        let commit = context
            .jj
            .working_copy_commit(&clone, &record.name)
            .map_err(boxed)?;
        let pin = format!("refs/mori/removed/{entry_id}");
        context.jj.pin(&clone, &pin, &commit).map_err(boxed)?;
        context
            .jj
            .forget_workspace(&clone, &record.name)
            .map_err(boxed)?;
        std::fs::remove_dir_all(&path).map_err(|source| {
            boxed(StoreError::Io {
                path: path.clone(),
                source,
            })
        })?;
        (commit, pin)
    } else {
        (String::new(), String::new())
    };
    db.delete_tree(&repo.id, &record.name).map_err(boxed)?;
    Ok(JournalEntry {
        id: entry_id.to_owned(),
        removed_at: now(),
        repo: repo.remote.clone(),
        tree_id: record.id.clone(),
        name: record.name.clone(),
        path: path.display().to_string(),
        owner: record.owner.clone(),
        task: record.task.clone(),
        lifetime: record.lifetime.clone(),
        commit_id,
        pin,
        bookmarks,
    })
}

fn same_path(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

//! What mori knows about whether a tree's work landed: the bookmarks it saw pushed from the tree,
//! recorded as it sees them, checked against the remote (and, when asked, `gh`).

use std::path::Path;

use mori_core::clone::RepoId;
use mori_core::error::ErrorDetails;
use mori_core::gc::{Landing, SeenBookmark, landing};
use mori_github::{GhCli, Merged};
use mori_jj::JjCli;
use mori_store::database::Database;

use crate::state::boxed;

/// Which recorded tree, and where.
pub struct TreeRef<'a> {
    /// The repo.
    pub repo: &'a RepoId,
    /// Its clone.
    pub clone: &'a Path,
    /// The tree's record ID.
    pub id: &'a str,
    /// The tree's name, which is also its workspace name.
    pub name: &'a str,
}

/// A recorded bookmark, as mori sees it now.
pub struct Seen {
    /// The remote.
    pub remote: String,
    /// The bookmark.
    pub bookmark: String,
    /// Where it pointed when last seen.
    pub commit_id: String,
    /// Whether the remote still has it.
    pub on_remote: bool,
    /// Whether its work landed.
    pub landed: bool,
}

/// Records the bookmarks now pointing into the tree, then decides from everything recorded
/// which of them landed. With `gh`, it also asks whether their pull requests merged.
pub fn observe(
    db: &mut Database,
    jj: &JjCli,
    gh: Option<&GhCli>,
    tree: &TreeRef,
) -> Result<Landing, Box<dyn ErrorDetails>> {
    observe_each(db, jj, gh, tree).map(|(landing, _)| landing)
}

/// Like [`observe`], and also says what is known about each recorded bookmark.
pub fn observe_each(
    db: &mut Database,
    jj: &JjCli,
    gh: Option<&GhCli>,
    tree: &TreeRef,
) -> Result<(Landing, Vec<Seen>), Box<dyn ErrorDetails>> {
    let current = jj.pushed_bookmarks(tree.clone, tree.name).map_err(boxed)?;
    for bookmark in &current {
        db.record_tree_bookmark(
            tree.id,
            &bookmark.remote,
            &bookmark.name,
            &bookmark.commit_id,
        )
        .map_err(boxed)?;
    }
    let mut each = Vec::new();
    let mut facts = Vec::new();
    for recorded in db.tree_bookmarks(tree.id).map_err(boxed)? {
        let on_remote = current
            .iter()
            .any(|now| now.remote == recorded.remote && now.name == recorded.bookmark);
        let pr_merged = gh.and_then(|gh| match gh.pr_merged(tree.repo, &recorded.bookmark) {
            Merged::Yes => Some(true),
            Merged::No => Some(false),
            Merged::Unknown => None,
        });
        let fact = SeenBookmark {
            commit_id: recorded.commit_id.clone(),
            on_remote,
            pr_merged,
        };
        each.push(Seen {
            landed: !landing(std::slice::from_ref(&fact)).commits.is_empty(),
            remote: recorded.remote,
            bookmark: recorded.bookmark,
            commit_id: recorded.commit_id,
            on_remote,
        });
        facts.push(fact);
    }
    Ok((landing(&facts), each))
}

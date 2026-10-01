//! The side effects mori's flows need from a version control system and a code host, as traits.
//!
//! The adapter crates implement them (`mori-jj` today), so a flow can run against any backend,
//! or against a fake in a unit test. Each method is one fact or one change; the rules that decide
//! what to do with them stay in this crate's pure functions.

use std::path::Path;

use serde::Deserialize;

use crate::clone::RepoId;
use crate::forest::{TreeState, Workspaces};

/// Which VCS a clone and its trees use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VcsKind {
    /// jj: trees are jj workspaces.
    #[default]
    Jj,
    /// git: trees are detached git worktrees.
    Git,
}

impl std::str::FromStr for VcsKind {
    type Err = String;

    /// Parses `jj` or `git`, as in `config.toml` and `--vcs`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "jj" => Ok(Self::Jj),
            "git" => Ok(Self::Git),
            _ => Err(format!("invalid vcs {text:?}: use jj or git")),
        }
    }
}

impl std::fmt::Display for VcsKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Jj => "jj",
            Self::Git => "git",
        })
    }
}

/// `[vcs]` in `config.toml`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VcsPolicy {
    /// The backend `mori clone` uses when `--vcs` isn't given.
    pub default: VcsKind,
}

/// A bookmark (branch) on a remote, as the VCS sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteBookmark {
    /// The bookmark, e.g. `claude/fix-login`.
    pub name: String,
    /// The remote, e.g. `origin`.
    pub remote: String,
    /// The full commit ID it points to.
    pub commit_id: String,
}

/// Clones and trees: everything mori asks of a VCS beyond reading workspaces.
///
/// A tree is named by its clone and its name, which is also the backend's own name for it (a jj
/// workspace, a git worktree).
pub trait Vcs: Workspaces {
    /// Clones `url` into `path`, which must not exist; `colocate` asks for a git repo alongside
    /// when the backend has a choice. A failed clone leaves nothing at `path`.
    ///
    /// # Errors
    ///
    /// If `path` exists (left alone), or the VCS fails or can't be run.
    fn clone_repo(&self, url: &str, path: &Path, colocate: bool) -> Result<(), Self::Error>;

    /// Adds the tree `name` at `path`, which must not exist, starting from the revision `from`.
    /// A failed add leaves nothing at `path` and no tree registered.
    ///
    /// # Errors
    ///
    /// If `path` exists (left alone), `from` matches nothing, or the VCS fails.
    fn add_tree(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        from: &str,
    ) -> Result<(), Self::Error>;

    /// Like [`Vcs::add_tree`], starting from `commit_id` even if the VCS no longer keeps it
    /// reachable (it is still in the object store, e.g. pinned by a ref).
    ///
    /// # Errors
    ///
    /// As [`Vcs::add_tree`], and if the commit can't be brought back.
    fn add_tree_at(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        commit_id: &str,
    ) -> Result<(), Self::Error>;

    /// Records the tree's on-disk edits, so they show in its state and can be pinned.
    ///
    /// # Errors
    ///
    /// If the VCS fails.
    fn snapshot(&self, tree: &Path) -> Result<(), Self::Error>;

    /// Unregisters the tree `name`; its files are the caller's to remove.
    ///
    /// # Errors
    ///
    /// If the VCS fails.
    fn forget_tree(&self, clone: &Path, name: &str) -> Result<(), Self::Error>;

    /// Like [`Workspaces::state`], but changes that are ancestors of any of `landed` (commit IDs
    /// of work that landed, e.g. a squash-merged bookmark's last target) count as saved too.
    ///
    /// # Errors
    ///
    /// As [`Workspaces::state`].
    fn state_covering(
        &self,
        clone: &Path,
        name: &str,
        landed: &[String],
    ) -> Result<TreeState, Self::Error>;

    /// The commit ID of the tree's working copy, after a [`Vcs::snapshot`].
    ///
    /// # Errors
    ///
    /// If the VCS fails or has no such tree.
    fn working_copy_commit(&self, clone: &Path, name: &str) -> Result<String, Self::Error>;

    /// The remote bookmarks whose history holds the tree's own work (its history not yet in
    /// trunk): those pointing at it and those stacked on top of it, but never trunk itself.
    ///
    /// # Errors
    ///
    /// If the VCS fails or its answer can't be read.
    fn pushed_bookmarks(
        &self,
        clone: &Path,
        name: &str,
    ) -> Result<Vec<RemoteBookmark>, Self::Error>;

    /// Every bookmark the clone knows the remotes have, wherever it points.
    ///
    /// # Errors
    ///
    /// If the VCS fails or its answer can't be read.
    fn remote_bookmarks(&self, clone: &Path) -> Result<Vec<RemoteBookmark>, Self::Error>;

    /// When the tree's working copy last changed, in seconds since the Unix epoch.
    ///
    /// # Errors
    ///
    /// If the VCS fails or its answer can't be read.
    fn last_change(&self, clone: &Path, name: &str) -> Result<u64, Self::Error>;

    /// Points the ref `name` (e.g. `refs/mori/removed/<entry>`) at `commit_id`, so the commit is
    /// kept whatever happens to the tree.
    ///
    /// # Errors
    ///
    /// If the VCS fails.
    fn pin(&self, clone: &Path, name: &str, commit_id: &str) -> Result<(), Self::Error>;

    /// Whether the clone still has the commit `commit_id`.
    ///
    /// # Errors
    ///
    /// If the VCS can't be asked.
    fn commit_exists(&self, clone: &Path, commit_id: &str) -> Result<bool, Self::Error>;

    /// Fetches from the clone's remotes.
    ///
    /// # Errors
    ///
    /// If the fetch fails (offline, for example).
    fn fetch(&self, clone: &Path) -> Result<(), Self::Error>;
}

/// Whether a bookmark's pull request merged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Merged {
    /// The host says the pull request merged.
    Yes,
    /// There is no pull request for the bookmark, or it is open or closed without merging.
    No,
    /// The host couldn't be asked: no tool, not logged in, offline, or an unexpected reply.
    Unknown,
}

/// A code host's view of pull requests. Every answer is a fact or [`Merged::Unknown`]; an
/// implementation never guesses "merged".
pub trait Forge {
    /// Whether the pull request whose head is `bookmark` in `repo` merged.
    fn pr_merged(&self, repo: &RepoId, bookmark: &str) -> Merged;
}

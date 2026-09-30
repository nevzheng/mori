//! Matching what mori recorded against what the VCS reports.
//!
//! The VCS is the truth. mori only pays attention to what it created, so a mismatch is reported,
//! never repaired: a record with no workspace is missing, and a workspace with no record is
//! foreign. Nothing here adopts, imports or removes anything.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A workspace the VCS reports for a clone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    /// Its name in the VCS. For a tree mori created, this is the tree's name.
    pub name: String,
    /// Its working directory.
    pub root: PathBuf,
}

/// What the VCS says about one workspace, as of its last snapshot. Read without snapshotting, so
/// someone working in the tree is never disturbed; edits since jj last ran there don't show yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeState {
    /// The working-copy change's short ID.
    pub change: String,
    /// True if the working-copy change has edits.
    pub changed: bool,
    /// How many non-empty changes in the tree's history are on no remote bookmark and not in
    /// trunk: work that exists only on this machine.
    pub unpushed: u32,
}

/// Reads the workspaces of a clone. Implemented by the VCS backends; read-only.
pub trait Workspaces {
    /// The backend's error.
    type Error;

    /// Every workspace of the clone at `clone`, in any order.
    ///
    /// # Errors
    ///
    /// When the VCS can't be run or its answer can't be read.
    fn list(&self, clone: &Path) -> Result<Vec<Workspace>, Self::Error>;

    /// The state of the workspace named `name` in the clone at `clone`.
    ///
    /// # Errors
    ///
    /// When the VCS can't be run, has no such workspace, or its answer can't be read.
    fn state(&self, clone: &Path, name: &str) -> Result<TreeState, Self::Error>;
}

/// One line of the forest: a recorded tree, its workspace, or both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry<R> {
    /// mori created it, and the VCS still has it.
    Tree {
        /// What mori recorded.
        record: R,
        /// What the VCS reports.
        workspace: Workspace,
    },
    /// mori recorded it, but the VCS has no workspace of that name.
    Missing {
        /// What mori recorded.
        record: R,
    },
    /// The VCS has it, but mori didn't create it.
    Foreign {
        /// What the VCS reports.
        workspace: Workspace,
    },
}

/// Pairs records with workspaces by name, using `name_of` to read a record's name. The result is
/// sorted by name. Never fails: every record and every workspace appears exactly once.
pub fn reconcile<R>(
    records: impl IntoIterator<Item = R>,
    workspaces: impl IntoIterator<Item = Workspace>,
    name_of: impl Fn(&R) -> &str,
) -> Vec<Entry<R>> {
    let mut by_name: BTreeMap<String, (Option<R>, Option<Workspace>)> = BTreeMap::new();
    for record in records {
        let name = name_of(&record).to_owned();
        by_name.entry(name).or_default().0 = Some(record);
    }
    for workspace in workspaces {
        let name = workspace.name.clone();
        by_name.entry(name).or_default().1 = Some(workspace);
    }
    by_name
        .into_values()
        .filter_map(|pair| match pair {
            (Some(record), Some(workspace)) => Some(Entry::Tree { record, workspace }),
            (Some(record), None) => Some(Entry::Missing { record }),
            (None, Some(workspace)) => Some(Entry::Foreign { workspace }),
            (None, None) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(name: &str) -> Workspace {
        Workspace {
            name: name.to_owned(),
            root: PathBuf::from(format!("/home/acme/mori/trees/widget/{name}")),
        }
    }

    fn names(entries: &[Entry<&str>]) -> Vec<String> {
        entries
            .iter()
            .map(|entry| match entry {
                Entry::Tree { record, .. } => format!("tree {record}"),
                Entry::Missing { record } => format!("missing {record}"),
                Entry::Foreign { workspace } => format!("foreign {}", workspace.name),
            })
            .collect()
    }

    #[test]
    fn records_and_workspaces_pair_up_by_name() {
        let entries = reconcile(
            ["default", "claude-fix-login"],
            [workspace("claude-fix-login"), workspace("default")],
            |name| name,
        );

        assert_eq!(names(&entries), ["tree claude-fix-login", "tree default"]);
    }

    #[test]
    fn a_record_without_a_workspace_is_missing() {
        let entries = reconcile(["default", "claude-gone"], [workspace("default")], |n| n);

        assert_eq!(names(&entries), ["missing claude-gone", "tree default"]);
    }

    #[test]
    fn a_workspace_without_a_record_is_foreign() {
        let entries = reconcile(
            ["default"],
            [workspace("default"), workspace("scratch")],
            |n| n,
        );

        assert_eq!(names(&entries), ["tree default", "foreign scratch"]);
    }

    #[test]
    fn nothing_recorded_means_everything_is_foreign() {
        let entries = reconcile(Vec::<&str>::new(), [workspace("default")], |n| n);

        assert_eq!(names(&entries), ["foreign default"]);
    }
}

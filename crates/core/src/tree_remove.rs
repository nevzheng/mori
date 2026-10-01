//! `mori tree remove`: remove a task tree whose work is safe elsewhere.
//!
//! A tree may go only when nothing in it exists only on this machine: no edits in its working
//! copy and no change missing from the remote ("pushed = safe"). The adapter snapshots the tree
//! first, so edits since jj last ran there count, then [`plan`] decides from what it observed.
//! Removing forgets the workspace, deletes the directory, then drops the record; a crash between
//! steps leaves a missing tree, which `mori ls` shows.
//!
//! A tree whose directory was deleted by hand can't be checked: its edits are gone, and only its
//! last commit is left in the clone. It may go anyway, with that commit pinned and journalled so
//! `mori restore` brings it back.

use std::path::PathBuf;

use crate::error::TreeError;
use crate::forest::TreeState;
use crate::tree::{Lifetime, Role};

/// The tree as mori recorded it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    /// Its name.
    pub name: String,
    /// Its role.
    pub role: Role,
    /// Its lifetime.
    pub lifetime: Lifetime,
    /// Where it is.
    pub path: PathBuf,
}

/// What the adapter found before removing anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed {
    /// The record, if mori made a tree of that name in the repo.
    pub recorded: Option<Recorded>,
    /// The workspace's state after a fresh snapshot, if the VCS has a workspace of that name and
    /// its directory exists.
    pub state: Option<TreeState>,
    /// The VCS has a workspace of that name, but its directory was deleted.
    pub dir_gone: bool,
}

/// What is asked for. Anyone may remove any task tree that is safe to remove: ownership is a
/// label, not a lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// The tree's name.
    pub name: String,
    /// Allow removing a pinned tree.
    pub pinned_ok: bool,
}

/// What `tree remove` will do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovePlan {
    /// The tree's name.
    pub name: String,
    /// The directory to delete.
    pub path: PathBuf,
    /// Whether there is a workspace to forget. A missing tree only has its record dropped.
    pub forget_workspace: bool,
    /// Pin the tree's last commit and journal the removal first: its directory is gone, so
    /// nothing could check that its work is safe elsewhere.
    pub pin: bool,
}

/// Decides whether the tree may go.
///
/// # Errors
///
/// [`TreeError::WorkspaceExists`] for a workspace mori didn't make (never removed);
/// [`TreeError::NotFound`] if there is no such tree at all; [`TreeError::BaseTree`] for the
/// clone itself; [`TreeError::Pinned`] for a
/// pinned tree without `pinned_ok`; [`TreeError::Unsaved`] if it has edits or unpushed changes.
pub fn plan(request: &Request, observed: &Observed) -> Result<RemovePlan, TreeError> {
    let name = request.name.clone();
    let Some(recorded) = &observed.recorded else {
        return Err(if observed.state.is_some() {
            TreeError::WorkspaceExists { name }
        } else {
            TreeError::NotFound { name }
        });
    };
    if recorded.role == Role::Base {
        return Err(TreeError::BaseTree { name });
    }
    if recorded.lifetime == Lifetime::Pinned && !request.pinned_ok {
        return Err(TreeError::Pinned { name });
    }
    if observed.dir_gone {
        return Ok(RemovePlan {
            name,
            path: recorded.path.clone(),
            forget_workspace: true,
            pin: true,
        });
    }
    if let Some(state) = &observed.state
        && (state.changed || state.unpushed > 0)
    {
        return Err(TreeError::Unsaved {
            name,
            edited: state.changed,
            unpushed: state.unpushed,
        });
    }
    Ok(RemovePlan {
        name,
        path: recorded.path.clone(),
        forget_workspace: observed.state.is_some(),
        pin: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Code, ErrorDetails};

    fn request() -> Request {
        Request {
            name: "claude-fix-login".to_owned(),
            pinned_ok: false,
        }
    }

    fn recorded() -> Recorded {
        Recorded {
            name: "claude-fix-login".to_owned(),
            role: Role::Task,
            lifetime: Lifetime::TaskDone,
            path: PathBuf::from("/home/acme/mori/trees/widget/claude-fix-login"),
        }
    }

    fn clean() -> TreeState {
        TreeState {
            change: "vmvywosutlnw".to_owned(),
            changed: false,
            unpushed: 0,
        }
    }

    fn observed() -> Observed {
        Observed {
            recorded: Some(recorded()),
            state: Some(clean()),
            dir_gone: false,
        }
    }

    #[test]
    fn a_clean_pushed_tree_may_go() {
        let plan = plan(&request(), &observed()).unwrap();

        assert_eq!(plan.path, recorded().path);
        assert!(plan.forget_workspace);
    }

    #[test]
    fn a_tree_whose_directory_is_gone_goes_with_its_commit_pinned() {
        let observed = Observed {
            state: None,
            dir_gone: true,
            ..observed()
        };

        let plan = plan(&request(), &observed).unwrap();

        assert!(plan.forget_workspace);
        assert!(plan.pin);
    }

    #[test]
    fn a_pinned_tree_whose_directory_is_gone_still_needs_pinned_ok() {
        let observed = Observed {
            recorded: Some(Recorded {
                lifetime: Lifetime::Pinned,
                ..recorded()
            }),
            state: None,
            dir_gone: true,
        };

        assert_eq!(
            plan(&request(), &observed).unwrap_err().reason(),
            "TREE_PINNED"
        );
    }

    #[test]
    fn a_missing_tree_only_loses_its_record() {
        let observed = Observed {
            state: None,
            ..observed()
        };

        assert!(!plan(&request(), &observed).unwrap().forget_workspace);
    }

    #[test]
    fn work_only_this_machine_has_blocks_removal() {
        for (changed, unpushed) in [(true, 0), (false, 1), (true, 3)] {
            let observed = Observed {
                state: Some(TreeState {
                    changed,
                    unpushed,
                    ..clean()
                }),
                ..observed()
            };

            let error = plan(&request(), &observed).unwrap_err();

            assert_eq!(error.code(), Code::FailedPrecondition);
            assert_eq!(error.reason(), "TREE_HAS_UNSAVED_WORK");
        }
    }

    #[test]
    fn only_mori_s_own_task_trees_go() {
        let cases = [
            (
                Observed {
                    recorded: None,
                    ..observed()
                },
                request(),
                "WORKSPACE_EXISTS",
            ),
            (
                Observed {
                    recorded: None,
                    state: None,
                    dir_gone: false,
                },
                request(),
                "TREE_NOT_FOUND",
            ),
            (
                Observed {
                    recorded: Some(Recorded {
                        role: Role::Base,
                        lifetime: Lifetime::Pinned,
                        ..recorded()
                    }),
                    ..observed()
                },
                Request {
                    pinned_ok: true,
                    ..request()
                },
                "BASE_TREE",
            ),
        ];
        for (observed, request, expected) in cases {
            assert_eq!(plan(&request, &observed).unwrap_err().reason(), expected);
        }
    }

    #[test]
    fn a_pinned_tree_needs_to_be_asked_for() {
        let observed = Observed {
            recorded: Some(Recorded {
                lifetime: Lifetime::Pinned,
                ..recorded()
            }),
            ..observed()
        };

        assert_eq!(
            plan(&request(), &observed).unwrap_err().reason(),
            "TREE_PINNED"
        );
        let request = Request {
            pinned_ok: true,
            ..request()
        };
        assert!(plan(&request, &observed).is_ok());
    }
}

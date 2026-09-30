//! `mori tree remove`: remove a task tree whose work is safe elsewhere.
//!
//! A tree may go only when nothing in it exists only on this machine: no edits in its working
//! copy and no change missing from the remote ("pushed = safe"). The adapter snapshots the tree
//! first, so edits since jj last ran there count, then [`plan`] decides from what it observed.
//! Removing forgets the workspace, deletes the directory, then drops the record; a crash between
//! steps leaves a missing tree, which `mori ls` shows.

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
    /// Who it's for.
    pub owner: String,
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
    /// The workspace's state after a fresh snapshot, if the VCS has a workspace of that name.
    pub state: Option<TreeState>,
}

/// Who is asking, and what they allow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// The tree's name.
    pub name: String,
    /// Who is asking: `--agent`, or the login name.
    pub owner: String,
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
}

/// Decides whether the tree may go.
///
/// # Errors
///
/// [`TreeError::WorkspaceExists`] for a workspace mori didn't make (never removed);
/// [`TreeError::NotFound`] if there is no such tree at all; [`TreeError::BaseTree`] for the
/// clone itself; [`TreeError::NotOwner`] if someone else owns it; [`TreeError::Pinned`] for a
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
    if recorded.owner != request.owner {
        return Err(TreeError::NotOwner {
            name,
            owner: recorded.owner.clone(),
        });
    }
    if recorded.lifetime == Lifetime::Pinned && !request.pinned_ok {
        return Err(TreeError::Pinned { name });
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Code, ErrorDetails};

    fn request() -> Request {
        Request {
            name: "claude-fix-login".to_owned(),
            owner: "claude".to_owned(),
            pinned_ok: false,
        }
    }

    fn recorded() -> Recorded {
        Recorded {
            name: "claude-fix-login".to_owned(),
            role: Role::Task,
            owner: "claude".to_owned(),
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
        }
    }

    #[test]
    fn a_clean_pushed_tree_may_go() {
        let plan = plan(&request(), &observed()).unwrap();

        assert_eq!(plan.path, recorded().path);
        assert!(plan.forget_workspace);
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
    fn only_mori_s_own_task_trees_of_the_right_owner_go() {
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
            (
                observed(),
                Request {
                    owner: "codex".to_owned(),
                    ..request()
                },
                "NOT_TREE_OWNER",
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

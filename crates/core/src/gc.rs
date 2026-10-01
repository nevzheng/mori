//! `mori gc`: which trees may go, and which of those are safe to go.
//!
//! Two questions per tree. **May it go?** Its lifetime says so: `task-done` and its work landed,
//! `ttl` and nothing changed in it for that long, or `lru` and its repo is over the cap. **Is it
//! safe?** Nothing in it exists only on this machine. The adapter gathers the facts from the VCS
//! and the remote; [`classify`] decides. Unknown facts never make a tree a candidate.

use crate::forest::TreeState;
use crate::tree::{Lifetime, Role};

/// What happens to a tree in a cleanup.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// It may go, and nothing would be lost.
    Remove,
    /// It may go, but it has work only this machine has.
    Blocked,
    /// Its lifetime doesn't let it go yet.
    Keep,
    /// Cleanup never removes it.
    Never,
}

/// Why a tree is in its class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// The clone itself.
    Base,
    /// A workspace mori didn't make.
    Foreign,
    /// Its lifetime is pinned.
    Pinned,
    /// Recorded, but its workspace is gone: only the record goes.
    Missing,
    /// `task-done`, and its work landed.
    Landed,
    /// `ttl`, and nothing changed in it for that long.
    Idle,
    /// Its lifetime doesn't make it a candidate yet.
    NotYet,
    /// Whether it may go can't be known now (offline, no GitHub access, no change time).
    Unknown,
    /// A candidate with edits or unpushed changes.
    Unsaved,
}

impl Reason {
    /// The reason code shown in reports.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::Base => "BASE",
            Self::Foreign => "FOREIGN",
            Self::Pinned => "PINNED",
            Self::Missing => "MISSING",
            Self::Landed => "LANDED",
            Self::Idle => "IDLE",
            Self::NotYet => "NOT_YET",
            Self::Unknown => "UNKNOWN",
            Self::Unsaved => "UNSAVED",
        }
    }
}

/// Everything known about one tree, as the adapter found it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The tree's role and lifetime, if mori recorded it; none for a foreign workspace.
    pub recorded: Option<(Role, Lifetime)>,
    /// The VCS state, if the workspace exists.
    pub state: Option<TreeState>,
    /// Whether its work landed (for `task-done`); none when that can't be checked.
    pub landed: Option<bool>,
    /// Seconds since its latest change (for `ttl`); none when unknown.
    pub idle_seconds: Option<u64>,
}

/// Sorts one tree into its class, with the reason. First match wins.
#[must_use]
pub fn classify(facts: &Facts) -> (Class, Reason) {
    let Some((role, lifetime)) = facts.recorded else {
        return (Class::Never, Reason::Foreign);
    };
    if role == Role::Base {
        return (Class::Never, Reason::Base);
    }
    if lifetime == Lifetime::Pinned {
        return (Class::Never, Reason::Pinned);
    }
    let Some(state) = &facts.state else {
        return (Class::Remove, Reason::Missing);
    };
    let candidate = match lifetime {
        Lifetime::Pinned => return (Class::Never, Reason::Pinned),
        Lifetime::TaskDone => match facts.landed {
            Some(true) => Reason::Landed,
            Some(false) => return (Class::Keep, Reason::NotYet),
            None => return (Class::Keep, Reason::Unknown),
        },
        Lifetime::Ttl(span) => match facts.idle_seconds {
            Some(idle) if idle >= span.as_secs() => Reason::Idle,
            Some(_) => return (Class::Keep, Reason::NotYet),
            None => return (Class::Keep, Reason::Unknown),
        },
    };
    if state.changed || state.unpushed > 0 {
        (Class::Blocked, Reason::Unsaved)
    } else {
        (Class::Remove, candidate)
    }
}

/// A bookmark mori recorded as pushed from a tree, and what is known about it now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeenBookmark {
    /// Where it pointed when mori last saw it.
    pub commit_id: String,
    /// Whether the remote still has it, as far as the clone knows.
    pub on_remote: bool,
    /// Whether its pull request merged; none when unknown or not asked.
    pub pr_merged: Option<bool>,
}

/// Whether a tree's work landed, and which commits hold the work that did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Landing {
    /// Some(true) if any recorded bookmark landed; Some(false) if none did and every fact the
    /// policy asks for is known; none otherwise.
    pub landed: Option<bool>,
    /// The recorded commits of the bookmarks that landed. The tree's changes that are their
    /// ancestors count as saved.
    pub commits: Vec<String>,
}

/// Decides which of a tree's recorded bookmarks landed: a bookmark landed when its pull request
/// merged, or when it is gone from the remote after mori saw it pushed.
#[must_use]
pub fn landing(seen: &[SeenBookmark]) -> Landing {
    let landed: Vec<&SeenBookmark> = seen
        .iter()
        .filter(|bookmark| !bookmark.on_remote || bookmark.pr_merged == Some(true))
        .collect();
    let unknown = seen
        .iter()
        .any(|bookmark| bookmark.on_remote && bookmark.pr_merged.is_none());
    Landing {
        landed: if !landed.is_empty() {
            Some(true)
        } else if unknown {
            None
        } else {
            Some(false)
        },
        commits: landed
            .into_iter()
            .map(|bookmark| bookmark.commit_id.clone())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    const DAY: u64 = 24 * 60 * 60;

    fn clean() -> TreeState {
        TreeState {
            change: "vmvywosutlnw".to_owned(),
            changed: false,
            unpushed: 0,
        }
    }

    fn task(lifetime: Lifetime) -> Facts {
        Facts {
            recorded: Some((Role::Task, lifetime)),
            state: Some(clean()),
            landed: None,
            idle_seconds: None,
        }
    }

    #[test]
    fn never_the_base_a_foreign_workspace_or_a_pinned_tree() {
        let base = Facts {
            recorded: Some((Role::Base, Lifetime::Pinned)),
            ..task(Lifetime::Pinned)
        };
        let foreign = Facts {
            recorded: None,
            ..task(Lifetime::TaskDone)
        };
        let pinned = Facts {
            landed: Some(true),
            ..task(Lifetime::Pinned)
        };

        assert_eq!(classify(&base), (Class::Never, Reason::Base));
        assert_eq!(classify(&foreign), (Class::Never, Reason::Foreign));
        assert_eq!(classify(&pinned), (Class::Never, Reason::Pinned));
    }

    #[test]
    fn a_missing_tree_only_loses_its_record() {
        let missing = Facts {
            state: None,
            ..task(Lifetime::TaskDone)
        };

        assert_eq!(classify(&missing), (Class::Remove, Reason::Missing));
    }

    #[test]
    fn task_done_goes_when_landed_and_waits_when_unknown() {
        let landed = Facts {
            landed: Some(true),
            ..task(Lifetime::TaskDone)
        };
        let open = Facts {
            landed: Some(false),
            ..task(Lifetime::TaskDone)
        };

        assert_eq!(classify(&landed), (Class::Remove, Reason::Landed));
        assert_eq!(classify(&open), (Class::Keep, Reason::NotYet));
        assert_eq!(
            classify(&task(Lifetime::TaskDone)),
            (Class::Keep, Reason::Unknown)
        );
    }

    #[test]
    fn ttl_goes_when_idle_long_enough() {
        let ttl = Lifetime::Ttl(Duration::from_secs(14 * DAY));
        let idle = Facts {
            idle_seconds: Some(15 * DAY),
            ..task(ttl)
        };
        let busy = Facts {
            idle_seconds: Some(DAY),
            ..task(ttl)
        };

        assert_eq!(classify(&idle), (Class::Remove, Reason::Idle));
        assert_eq!(classify(&busy), (Class::Keep, Reason::NotYet));
        assert_eq!(classify(&task(ttl)), (Class::Keep, Reason::Unknown));
    }

    #[test]
    fn a_candidate_with_unsaved_work_is_blocked() {
        let landed_but_edited = Facts {
            landed: Some(true),
            state: Some(TreeState {
                changed: true,
                ..clean()
            }),
            ..task(Lifetime::TaskDone)
        };
        let idle_but_unpushed = Facts {
            idle_seconds: Some(30 * DAY),
            state: Some(TreeState {
                unpushed: 2,
                ..clean()
            }),
            ..task(Lifetime::Ttl(Duration::from_secs(14 * DAY)))
        };

        assert_eq!(
            classify(&landed_but_edited),
            (Class::Blocked, Reason::Unsaved)
        );
        assert_eq!(
            classify(&idle_but_unpushed),
            (Class::Blocked, Reason::Unsaved)
        );
    }

    #[test]
    fn reason_codes_are_stable() {
        assert_eq!(Reason::Landed.code(), "LANDED");
        assert_eq!(Reason::Unsaved.code(), "UNSAVED");
    }

    fn seen(commit: &str, on_remote: bool, pr_merged: Option<bool>) -> SeenBookmark {
        SeenBookmark {
            commit_id: commit.to_owned(),
            on_remote,
            pr_merged,
        }
    }

    #[test]
    fn a_bookmark_gone_from_the_remote_landed() {
        let landing = landing(&[seen("aaaa", false, None)]);

        assert_eq!(landing.landed, Some(true));
        assert_eq!(landing.commits, ["aaaa"]);
    }

    #[test]
    fn a_merged_pull_request_landed() {
        let landing = landing(&[seen("aaaa", true, Some(true))]);

        assert_eq!(landing.landed, Some(true));
    }

    #[test]
    fn nothing_pushed_is_not_landed() {
        assert_eq!(landing(&[]).landed, Some(false));
        assert_eq!(
            landing(&[seen("aaaa", true, Some(false))]).landed,
            Some(false)
        );
    }

    #[test]
    fn an_open_bookmark_with_no_answer_is_unknown() {
        assert_eq!(landing(&[seen("aaaa", true, None)]).landed, None);
    }
}

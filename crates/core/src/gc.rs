//! `mori gc`: which trees may go, and which of those are safe to go.
//!
//! Two questions per tree. **May it go?** Its lifetime says so: `task-done` and its work landed,
//! `ttl` and nothing changed in it for that long, or `lru` and its repo is over the cap. **Is it
//! safe?** Nothing in it exists only on this machine. The adapter gathers the facts from the VCS
//! and the remote; [`classify`] decides. Unknown facts never make a tree a candidate.

use std::collections::BTreeSet;

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
    /// `lru`, and its repo is over the cap.
    OverCap,
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
            Self::OverCap => "OVER_CAP",
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
    /// Whether it is beyond its repo's `lru` cap (see [`over_cap`]).
    pub over_cap: bool,
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
        Lifetime::Lru if facts.over_cap => Reason::OverCap,
        Lifetime::Lru => return (Class::Keep, Reason::NotYet),
    };
    if state.changed || state.unpushed > 0 {
        (Class::Blocked, Reason::Unsaved)
    } else {
        (Class::Remove, candidate)
    }
}

/// The trees beyond a repo's `lru` cap: of its task trees (name and seconds since their latest
/// change), all but the `max` most recently changed. Trees with no known change time count as the
/// most recent, so they are never pushed over the cap by a guess.
#[must_use]
pub fn over_cap(task_trees: &[(String, Option<u64>)], max: Option<u32>) -> BTreeSet<String> {
    let Some(max) = max else {
        return BTreeSet::new();
    };
    let mut ranked: Vec<&(String, Option<u64>)> = task_trees.iter().collect();
    ranked.sort_by_key(|(_, idle)| idle.unwrap_or(0));
    ranked
        .into_iter()
        .skip(usize::try_from(max).unwrap_or(usize::MAX))
        .map(|(name, _)| name.clone())
        .collect()
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
            over_cap: false,
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
        let over_cap_but_unpushed = Facts {
            over_cap: true,
            state: Some(TreeState {
                unpushed: 2,
                ..clean()
            }),
            ..task(Lifetime::Lru)
        };

        assert_eq!(
            classify(&landed_but_edited),
            (Class::Blocked, Reason::Unsaved)
        );
        assert_eq!(
            classify(&over_cap_but_unpushed),
            (Class::Blocked, Reason::Unsaved)
        );
    }

    #[test]
    fn over_cap_is_the_least_recently_changed_beyond_max() {
        let trees = [
            ("fresh".to_owned(), Some(DAY)),
            ("old".to_owned(), Some(30 * DAY)),
            ("oldest".to_owned(), Some(90 * DAY)),
            ("unknown".to_owned(), None),
        ];

        assert_eq!(
            over_cap(&trees, Some(2)),
            BTreeSet::from(["old".to_owned(), "oldest".to_owned()])
        );
        assert!(over_cap(&trees, None).is_empty());
        assert!(over_cap(&trees, Some(10)).is_empty());
    }

    #[test]
    fn reason_codes_are_stable() {
        assert_eq!(Reason::OverCap.code(), "OVER_CAP");
        assert_eq!(Reason::Unsaved.code(), "UNSAVED");
    }
}

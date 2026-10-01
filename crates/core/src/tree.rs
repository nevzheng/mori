//! What a tree is: its role, its name and its lifetime, and the `[trees]` policy that picks them.
//!
//! The policy only sets defaults. Trees exist for different reasons, so every default can be
//! overridden per tree, and nothing here decides how a tree is used.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use serde::Deserialize;

use crate::error::TreeError;

/// What a tree is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The clone itself: yours. Always pinned.
    Base,
    /// Every tree mori creates: one piece of work, usually an agent's. A long-lived coordinating
    /// ("lead") tree is a task tree with a pinned lifetime.
    Task,
}

impl Role {
    /// The role of the recorded tree named `name`. It isn't stored: the base tree is the clone's
    /// own workspace, [`crate::clone::BASE_TREE_NAME`], and every other tree is a task tree.
    #[must_use]
    pub fn of(name: &str) -> Self {
        if name == crate::clone::BASE_TREE_NAME {
            Self::Base
        } else {
            Self::Task
        }
    }
}

/// When a tree becomes a candidate for cleanup. Being a candidate never removes anything by
/// itself: cleanup still runs every safety check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum Lifetime {
    /// Never.
    Pinned,
    /// When the remote says its work landed: its pull request merged, or a bookmark it pushed was
    /// deleted from the remote. Never by ancestry: a squash merge puts a new commit on trunk.
    TaskDone,
    /// When it hasn't been used for this long.
    Ttl(Duration),
    /// When its repo is over the `[trees.lru] max` cap and it is among the least recently used.
    Lru,
}

const HOUR: u64 = 60 * 60;
const DAY: u64 = 24 * HOUR;

impl FromStr for Lifetime {
    type Err = String;

    /// Parses the form used in `config.toml`, flags and the database: `pinned`, `task-done`,
    /// `lru`, or `ttl:<n>d` / `ttl:<n>h`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!(
                "invalid lifetime {text:?}: use pinned, task-done, lru, or ttl:<n>d or ttl:<n>h"
            )
        };
        match text {
            "pinned" => Ok(Self::Pinned),
            "task-done" => Ok(Self::TaskDone),
            "lru" => Ok(Self::Lru),
            _ => {
                let span = text.strip_prefix("ttl:").ok_or_else(invalid)?;
                let (count, unit) = span.split_at(span.len().saturating_sub(1));
                let unit = match unit {
                    "d" => DAY,
                    "h" => HOUR,
                    _ => return Err(invalid()),
                };
                let count: u64 = count.parse().map_err(|_| invalid())?;
                match count.checked_mul(unit) {
                    Some(seconds) if seconds > 0 => Ok(Self::Ttl(Duration::from_secs(seconds))),
                    _ => Err(invalid()),
                }
            }
        }
    }
}

impl TryFrom<String> for Lifetime {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl fmt::Display for Lifetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pinned => f.write_str("pinned"),
            Self::TaskDone => f.write_str("task-done"),
            Self::Lru => f.write_str("lru"),
            Self::Ttl(span) if span.as_secs() % DAY == 0 => {
                write!(f, "ttl:{}d", span.as_secs() / DAY)
            }
            Self::Ttl(span) => write!(f, "ttl:{}h", span.as_secs() / HOUR),
        }
    }
}

/// A tree name template, such as `{owner}-{task}`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct NameTemplate(String);

/// The most characters a tree name may have.
pub const MAX_NAME_LEN: usize = 64;

impl NameTemplate {
    /// The name for a tree owned by `owner` for `task`.
    ///
    /// # Errors
    ///
    /// [`TreeError::NameInvalid`] if the result isn't a valid tree name (see [`validate_name`]).
    pub fn render(&self, owner: &str, task: &str) -> Result<String, TreeError> {
        let name = self.0.replace("{owner}", owner).replace("{task}", task);
        validate_name(&name)?;
        Ok(name)
    }
}

impl Default for NameTemplate {
    fn default() -> Self {
        Self("{owner}-{task}".to_owned())
    }
}

impl TryFrom<String> for NameTemplate {
    type Error = String;

    /// Accepts `{owner}` and `{task}` placeholders, lowercase letters, digits and hyphens, and at
    /// least one placeholder (a fixed name would clash on the second tree).
    fn try_from(template: String) -> Result<Self, Self::Error> {
        let literal = template.replace("{owner}", "").replace("{task}", "");
        if literal.len() == template.len() {
            return Err(format!(
                "tree name template {template:?} needs {{owner}} or {{task}}"
            ));
        }
        if let Some(bad) = literal
            .chars()
            .find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
        {
            return Err(format!(
                "tree name template {template:?} has {bad:?}; outside {{owner}} and {{task}} \
                 use only lowercase letters, digits and hyphens"
            ));
        }
        Ok(Self(template))
    }
}

/// Checks that `name` can be a tree's name, its workspace name and one path component under
/// `trees/<repo>/`: 1 to [`MAX_NAME_LEN`] lowercase letters, digits and single hyphens, starting
/// and ending with a letter or digit.
///
/// # Errors
///
/// [`TreeError::NameInvalid`] saying why not.
pub fn validate_name(name: &str) -> Result<(), TreeError> {
    let invalid = |why: &str| {
        Err(TreeError::NameInvalid {
            name: name.to_owned(),
            why: why.to_owned(),
        })
    };
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return invalid(&format!("it must be 1 to {MAX_NAME_LEN} characters"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return invalid("use only lowercase letters, digits and hyphens");
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return invalid("hyphens go between letters or digits, one at a time");
    }
    Ok(())
}

/// The `[trees]` section of `config.toml`. Every key is optional.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TreePolicy {
    /// How new trees are named.
    pub name: NameTemplate,
    /// Default lifetimes.
    pub lifetime: LifetimeDefaults,
    /// The cap for [`Lifetime::Lru`].
    pub lru: LruPolicy,
}

impl TreePolicy {
    /// The lifetime for a new tree with `role`: the base tree is always pinned; otherwise
    /// `requested` if given, else the default.
    #[must_use]
    pub fn lifetime(&self, role: Role, requested: Option<Lifetime>) -> Lifetime {
        match role {
            Role::Base => Lifetime::Pinned,
            Role::Task => requested.unwrap_or(self.lifetime.task),
        }
    }
}

/// `[trees.lifetime]`. There is no key for the base tree: it is always pinned, so no policy can
/// make your clone a cleanup candidate.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LifetimeDefaults {
    /// The default for task trees.
    pub task: Lifetime,
}

impl Default for LifetimeDefaults {
    fn default() -> Self {
        Self {
            task: Lifetime::TaskDone,
        }
    }
}

/// `[trees.lru]`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LruPolicy {
    /// How many task trees a repo keeps before its least recently changed `lru` trees become
    /// cleanup candidates. Unset: `lru` trees are never candidates.
    pub max: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Code, ErrorDetails};

    #[test]
    fn lifetimes_round_trip_through_their_text_form() {
        for text in ["pinned", "task-done", "lru", "ttl:14d", "ttl:36h"] {
            let lifetime: Lifetime = text.parse().unwrap();

            assert_eq!(lifetime.to_string(), text);
        }
    }

    #[test]
    fn ttl_is_normalized_to_days_when_it_can_be() {
        let lifetime: Lifetime = "ttl:48h".parse().unwrap();

        assert_eq!(lifetime, Lifetime::Ttl(Duration::from_secs(2 * DAY)));
        assert_eq!(lifetime.to_string(), "ttl:2d");
    }

    #[test]
    fn bad_lifetimes_are_refused() {
        for text in [
            "",
            "forever",
            "ttl",
            "ttl:",
            "ttl:d",
            "ttl:0d",
            "ttl:14",
            "ttl:14w",
            "ttl:-1d",
            "ttl:99999999999999999999d",
            "Pinned",
            "LRU",
        ] {
            assert!(text.parse::<Lifetime>().is_err(), "for {text:?}");
        }
    }

    #[test]
    fn the_default_template_joins_owner_and_task() {
        let name = NameTemplate::default()
            .render("claude", "fix-login")
            .unwrap();

        assert_eq!(name, "claude-fix-login");
    }

    #[test]
    fn a_custom_template_is_used() {
        let template = NameTemplate::try_from("{task}".to_owned()).unwrap();

        assert_eq!(template.render("claude", "fix-login").unwrap(), "fix-login");
    }

    #[test]
    fn templates_need_a_placeholder_and_safe_literals() {
        for template in [
            "lead",
            "{owner}/{task}",
            "{Owner}-{task}",
            "{owner}_{task}",
            "",
        ] {
            assert!(
                NameTemplate::try_from(template.to_owned()).is_err(),
                "for {template:?}"
            );
        }
    }

    #[test]
    fn a_task_cannot_escape_its_directory() {
        for task in ["../etc", "a/b", "Fix", "", "x--y", "trailing-"] {
            let error = NameTemplate::default().render("claude", task).unwrap_err();

            assert_eq!(error.code(), Code::InvalidArgument, "for {task:?}");
            assert_eq!(error.reason(), "TREE_NAME_INVALID");
        }
    }

    #[test]
    fn names_have_a_length_limit() {
        assert!(validate_name(&"a".repeat(MAX_NAME_LEN)).is_ok());
        assert!(validate_name(&"a".repeat(MAX_NAME_LEN + 1)).is_err());
    }

    #[test]
    fn the_role_comes_from_the_name() {
        assert_eq!(Role::of("default"), Role::Base);
        assert_eq!(Role::of("claude-fix-login"), Role::Task);
    }

    #[test]
    fn the_base_tree_is_always_pinned() {
        let policy = TreePolicy::default();

        assert_eq!(
            policy.lifetime(Role::Base, Some(Lifetime::TaskDone)),
            Lifetime::Pinned
        );
    }

    #[test]
    fn a_requested_lifetime_overrides_the_default() {
        let policy = TreePolicy::default();

        assert_eq!(policy.lifetime(Role::Task, None), Lifetime::TaskDone);
        assert_eq!(
            policy.lifetime(Role::Task, Some(Lifetime::Pinned)),
            Lifetime::Pinned
        );
    }
}

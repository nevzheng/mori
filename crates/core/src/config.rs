//! `config.toml`. It records only what differs from the built-in defaults: the root, and any
//! `[trees]` and `[vcs]` policy someone wrote by hand.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::disk::DiskPolicy;
use crate::error::ConfigError;
use crate::tree::TreePolicy;
use crate::vcs::VcsPolicy;

/// The config schema this version of mori reads and writes.
pub const SCHEMA: u32 = 1;

/// The parsed contents of `config.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The config schema version.
    pub schema: u32,
    /// The root mori was set up with.
    pub root: PathBuf,
    /// How trees are named and how long they live. Optional; every key has a default.
    #[serde(default)]
    pub trees: TreePolicy,
    /// Which VCS new clones use. Optional; jj by default.
    #[serde(default)]
    pub vcs: VcsPolicy,
    /// When to warn about disk space. Optional; below 10% free by default.
    #[serde(default)]
    pub disk: DiskPolicy,
}

impl Config {
    /// Parses `text`, read from `path` (used in error messages).
    ///
    /// # Errors
    ///
    /// [`ConfigError::ConfigInvalid`] if it doesn't parse, has unknown keys, a relative root, or
    /// an unsupported schema.
    pub fn parse(text: &str, path: &Path) -> Result<Self, ConfigError> {
        let invalid = |message: String| ConfigError::ConfigInvalid {
            path: path.to_path_buf(),
            message,
        };
        let config: Self = toml::from_str(text).map_err(|err| invalid(err.message().to_owned()))?;
        if config.schema != SCHEMA {
            return Err(invalid(format!(
                "unsupported schema {}; this mori reads schema {SCHEMA}",
                config.schema
            )));
        }
        if !config.root.is_absolute() {
            return Err(invalid(format!(
                "root must be absolute, got \"{}\"",
                config.root.display()
            )));
        }
        Ok(config)
    }

    /// Renders a fresh `config.toml` for `root`.
    ///
    /// # Errors
    ///
    /// [`ConfigError::NotUtf8`] if `root` isn't valid UTF-8.
    pub fn render(root: &Path) -> Result<String, ConfigError> {
        let root = toml::Value::String(crate::paths::utf8(root)?.to_owned());
        Ok(format!(
            "# mori configuration, written by `mori init`.\n\
             schema = {SCHEMA}\n\
             # Changing the root is a misconfiguration until root migrations exist.\n\
             root = {root}\n\
             \n\
             # What `mori clone` makes without --vcs: \"jj\" (the default) or \"git\".\n\
             # [vcs]\n\
             # default = \"jj\"\n\
             \n\
             # Warn when free space on the disk under the root drops below this: a share of the\n\
             # disk or a size such as \"200G\". \"0%\" turns the warning off.\n\
             # [disk]\n\
             # warn_below = \"10%\"\n\
             # Where Bazel keeps output bases, if you moved it with --output_user_root. `mori gc`\n\
             # removes the ones left by deleted trees.\n\
             # bazel_output_user_root = \"/fast/bazel\"\n"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorDetails;

    const PATH: &str = "/home/acme/.config/mori/config.toml";

    #[test]
    fn render_then_parse_round_trips() {
        let text = Config::render(Path::new("/home/acme/mori")).unwrap();

        let config = Config::parse(&text, Path::new(PATH)).unwrap();

        assert_eq!(
            config,
            Config {
                schema: SCHEMA,
                root: PathBuf::from("/home/acme/mori"),
                trees: TreePolicy::default(),
                vcs: VcsPolicy::default(),
                disk: DiskPolicy::default(),
            }
        );
    }

    #[test]
    fn render_escapes_the_root() {
        let text = Config::render(Path::new("/home/acme/odd \"name\"")).unwrap();

        let config = Config::parse(&text, Path::new(PATH)).unwrap();

        assert_eq!(config.root, PathBuf::from("/home/acme/odd \"name\""));
    }

    #[test]
    fn invalid_configs_are_refused() {
        for text in [
            "schema = 1",                                  // no root
            "schema = 2\nroot = \"/home/acme/mori\"",      // future schema
            "schema = 1\nroot = \"mori\"",                 // relative root
            "schema = 1\nroot = \"/r\"\ncolour = \"red\"", // unknown key
            "not toml",
            "schema = 1\nroot = \"/r\"\n[trees]\nname = \"{owner}/{task}\"",
            "schema = 1\nroot = \"/r\"\n[trees.lifetime]\ntask = \"forever\"",
            "schema = 1\nroot = \"/r\"\n[trees.lifetime]\nbase = \"lru\"",
            "schema = 1\nroot = \"/r\"\n[trees.landed]\nwhen = [\"pr-merged\"]",
            "schema = 1\nroot = \"/r\"\n[trees.lifetime]\ntask = \"forever\"",
        ] {
            let error = Config::parse(text, Path::new(PATH)).unwrap_err();

            assert_eq!(error.reason(), "CONFIG_INVALID", "for {text:?}");
        }
    }

    #[test]
    fn a_trees_section_sets_the_policy() {
        let text = r#"
            schema = 1
            root = "/home/acme/mori"

            [trees]
            name = "{task}"

            [trees.lifetime]
            task = "ttl:14d"

            [trees.lru]
            max = 5

        "#;

        let trees = Config::parse(text, Path::new(PATH)).unwrap().trees;

        assert_eq!(
            trees.name.render("claude", "fix-login").unwrap(),
            "fix-login"
        );
        assert_eq!(trees.lifetime.task.to_string(), "ttl:14d");
        assert_eq!(trees.lru.max, Some(5));
    }

    #[test]
    fn the_disk_floor_defaults_to_ten_percent() {
        let path = Path::new(PATH);
        let base = "schema = 1\nroot = \"/home/acme/mori\"\n";

        let unset = Config::parse(base, path).unwrap();
        let set = Config::parse(&format!("{base}[disk]\nwarn_below = \"200G\"\n"), path).unwrap();

        assert_eq!(unset.disk.warn_below, crate::disk::Floor::Percent(10));
        assert_eq!(
            set.disk.warn_below,
            crate::disk::Floor::Bytes(200 * 1024 * 1024 * 1024)
        );
        assert!(Config::parse(&format!("{base}[disk]\nwarn_below = \"lots\"\n"), path).is_err());
        assert!(Config::parse(&format!("{base}[disk]\nmax = 1\n"), path).is_err());
    }

    #[test]
    fn the_vcs_default_is_jj_and_can_be_git() {
        let path = Path::new(PATH);
        let base = "schema = 1\nroot = \"/home/acme/mori\"\n";

        let unset = Config::parse(base, path).unwrap();
        let git = Config::parse(&format!("{base}[vcs]\ndefault = \"git\"\n"), path).unwrap();

        assert_eq!(unset.vcs.default, crate::vcs::VcsKind::Jj);
        assert_eq!(git.vcs.default, crate::vcs::VcsKind::Git);
        assert!(Config::parse(&format!("{base}[vcs]\ndefault = \"hg\"\n"), path).is_err());
    }
}

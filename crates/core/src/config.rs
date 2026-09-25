//! `config.toml`. It records only what differs from the built-in defaults; today that's the root.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::ConfigError;

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
             root = {root}\n"
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
                root: PathBuf::from("/home/acme/mori")
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
        ] {
            let error = Config::parse(text, Path::new(PATH)).unwrap_err();

            assert_eq!(error.reason(), "CONFIG_INVALID", "for {text:?}");
        }
    }
}

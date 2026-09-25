//! Where mori keeps things. The root holds repos and trees (default `~/mori`); config, state and
//! cache follow the XDG Base Directory spec on every platform.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::error::ConfigError;

/// The environment variables mori reads, captured once so path resolution stays pure.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Env {
    /// `HOME`.
    pub home: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`.
    pub xdg_config_home: Option<PathBuf>,
    /// `XDG_STATE_HOME`.
    pub xdg_state_home: Option<PathBuf>,
    /// `XDG_CACHE_HOME`.
    pub xdg_cache_home: Option<PathBuf>,
    /// `MORI_ROOT`.
    pub mori_root: Option<PathBuf>,
    /// `MORI_CONFIG_DIR`.
    pub mori_config_dir: Option<PathBuf>,
    /// `MORI_STATE_DIR`.
    pub mori_state_dir: Option<PathBuf>,
    /// `MORI_CACHE_DIR`.
    pub mori_cache_dir: Option<PathBuf>,
}

impl Env {
    /// Captures the variables through `var`, e.g. `Env::from_vars(std::env::var_os)`.
    /// Empty values count as unset.
    pub fn from_vars(var: impl Fn(&str) -> Option<OsString>) -> Self {
        let get = |name: &str| {
            var(name)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        };
        Self {
            home: get("HOME"),
            xdg_config_home: get("XDG_CONFIG_HOME"),
            xdg_state_home: get("XDG_STATE_HOME"),
            xdg_cache_home: get("XDG_CACHE_HOME"),
            mori_root: get("MORI_ROOT"),
            mori_config_dir: get("MORI_CONFIG_DIR"),
            mori_state_dir: get("MORI_STATE_DIR"),
            mori_cache_dir: get("MORI_CACHE_DIR"),
        }
    }
}

/// Every path mori uses. All absolute and valid UTF-8.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    /// The root: `$MORI_ROOT`, else `~/mori`.
    pub root: PathBuf,
    /// The config directory.
    pub config_dir: PathBuf,
    /// `config.toml` inside the config directory.
    pub config_file: PathBuf,
    /// The state directory: the database, journal and reports.
    pub state_dir: PathBuf,
    /// `mori.db` inside the state directory.
    pub database: PathBuf,
    /// The cache directory: data that is safe to delete.
    pub cache_dir: PathBuf,
}

impl Paths {
    /// Resolves every path from `env`.
    ///
    /// Precedence for each directory: the `MORI_*` override, then the XDG variable, then the
    /// default under `HOME`. Relative XDG values are ignored, as the XDG spec requires; relative
    /// `MORI_*` values are errors.
    ///
    /// # Errors
    ///
    /// [`ConfigError::NotAbsolute`] for a relative `MORI_*` value, [`ConfigError::HomeNotSet`]
    /// when a default is needed without an absolute `HOME`, and [`ConfigError::NotUtf8`].
    pub fn resolve(env: &Env) -> Result<Self, ConfigError> {
        let home = || {
            env.home
                .clone()
                .filter(|home| home.is_absolute())
                .ok_or(ConfigError::HomeNotSet)
        };
        let dir = |over: (&'static str, &Option<PathBuf>), xdg: &Option<PathBuf>, default: &str| {
            if let Some(path) = absolute_override(over.0, over.1.as_ref())? {
                return Ok(path);
            }
            match xdg.as_ref().filter(|path| path.is_absolute()) {
                Some(base) => Ok(base.join("mori")),
                None => Ok(home()?.join(default).join("mori")),
            }
        };

        let root = match absolute_override("MORI_ROOT", env.mori_root.as_ref())? {
            Some(root) => root,
            None => home()?.join("mori"),
        };
        let config_dir = dir(
            ("MORI_CONFIG_DIR", &env.mori_config_dir),
            &env.xdg_config_home,
            ".config",
        )?;
        let state_dir = dir(
            ("MORI_STATE_DIR", &env.mori_state_dir),
            &env.xdg_state_home,
            ".local/state",
        )?;
        let cache_dir = dir(
            ("MORI_CACHE_DIR", &env.mori_cache_dir),
            &env.xdg_cache_home,
            ".cache",
        )?;

        let paths = Self {
            config_file: config_dir.join("config.toml"),
            database: state_dir.join("mori.db"),
            root,
            config_dir,
            state_dir,
            cache_dir,
        };
        for path in [
            &paths.root,
            &paths.config_file,
            &paths.database,
            &paths.cache_dir,
        ] {
            utf8(path)?;
        }
        Ok(paths)
    }

    /// `<root>/repos`, where clones live.
    #[must_use]
    pub fn repos(&self) -> PathBuf {
        self.root.join("repos")
    }

    /// `<root>/trees`, where agent trees live.
    #[must_use]
    pub fn trees(&self) -> PathBuf {
        self.root.join("trees")
    }
}

fn absolute_override(
    var: &'static str,
    value: Option<&PathBuf>,
) -> Result<Option<PathBuf>, ConfigError> {
    match value {
        Some(path) if path.is_absolute() => Ok(Some(path.clone())),
        Some(path) => Err(ConfigError::NotAbsolute {
            var,
            value: path.clone(),
        }),
        None => Ok(None),
    }
}

/// `path` as `&str`.
///
/// # Errors
///
/// [`ConfigError::NotUtf8`] if it isn't valid UTF-8.
pub fn utf8(path: &Path) -> Result<&str, ConfigError> {
    path.to_str().ok_or_else(|| ConfigError::NotUtf8 {
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorDetails;

    fn env(vars: &[(&str, &str)]) -> Env {
        Env::from_vars(|name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        })
    }

    #[test]
    fn defaults_come_from_home() {
        let paths = Paths::resolve(&env(&[("HOME", "/home/acme")])).unwrap();

        assert_eq!(paths.root, PathBuf::from("/home/acme/mori"));
        assert_eq!(
            paths.config_file,
            PathBuf::from("/home/acme/.config/mori/config.toml")
        );
        assert_eq!(
            paths.database,
            PathBuf::from("/home/acme/.local/state/mori/mori.db")
        );
        assert_eq!(paths.cache_dir, PathBuf::from("/home/acme/.cache/mori"));
    }

    #[test]
    fn xdg_variables_move_config_state_and_cache() {
        let paths = Paths::resolve(&env(&[
            ("HOME", "/home/acme"),
            ("XDG_CONFIG_HOME", "/xdg/config"),
            ("XDG_STATE_HOME", "/xdg/state"),
            ("XDG_CACHE_HOME", "/xdg/cache"),
        ]))
        .unwrap();

        assert_eq!(paths.config_dir, PathBuf::from("/xdg/config/mori"));
        assert_eq!(paths.state_dir, PathBuf::from("/xdg/state/mori"));
        assert_eq!(paths.cache_dir, PathBuf::from("/xdg/cache/mori"));
        assert_eq!(paths.root, PathBuf::from("/home/acme/mori"));
    }

    #[test]
    fn relative_xdg_values_are_ignored() {
        let paths =
            Paths::resolve(&env(&[("HOME", "/home/acme"), ("XDG_STATE_HOME", "state")])).unwrap();

        assert_eq!(
            paths.state_dir,
            PathBuf::from("/home/acme/.local/state/mori")
        );
    }

    #[test]
    fn mori_overrides_win() {
        let paths = Paths::resolve(&env(&[
            ("HOME", "/home/acme"),
            ("XDG_STATE_HOME", "/xdg/state"),
            ("MORI_ROOT", "/work/mori"),
            ("MORI_STATE_DIR", "/work/state"),
        ]))
        .unwrap();

        assert_eq!(paths.root, PathBuf::from("/work/mori"));
        assert_eq!(paths.state_dir, PathBuf::from("/work/state"));
    }

    #[test]
    fn relative_mori_root_is_an_error() {
        let error =
            Paths::resolve(&env(&[("HOME", "/home/acme"), ("MORI_ROOT", "mori")])).unwrap_err();

        assert_eq!(error.reason(), "PATH_NOT_ABSOLUTE");
    }

    #[test]
    fn missing_home_is_an_error_only_when_needed() {
        assert_eq!(
            Paths::resolve(&env(&[])).unwrap_err(),
            ConfigError::HomeNotSet
        );

        let without_home = Paths::resolve(&env(&[
            ("MORI_ROOT", "/r"),
            ("MORI_CONFIG_DIR", "/c"),
            ("MORI_STATE_DIR", "/s"),
            ("MORI_CACHE_DIR", "/k"),
        ]));
        assert!(without_home.is_ok());
    }

    #[test]
    fn empty_values_count_as_unset() {
        let paths = Paths::resolve(&env(&[("HOME", "/home/acme"), ("MORI_ROOT", "")])).unwrap();

        assert_eq!(paths.root, PathBuf::from("/home/acme/mori"));
    }
}

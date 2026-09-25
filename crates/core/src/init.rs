//! `mori init`: set up the root layout, the config and the state store, once per machine.
//!
//! The adapter observes the paths from [`paths_to_observe`], [`plan`] turns what it saw into an
//! [`InitPlan`] (or a refusal), and the adapter carries the plan out. An empty plan means mori is
//! already set up, which is what makes `init` safe to run twice. A dry run shows the plan and
//! stops.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{ConfigError, RootSource};
use crate::paths::Paths;

/// Mode for ordinary directories mori creates.
pub const DIR_MODE: u32 = 0o755;
/// Mode for the state directory: only the owner may read it.
pub const PRIVATE_DIR_MODE: u32 = 0o700;
/// Mode for `config.toml`.
pub const CONFIG_MODE: u32 = 0o644;
/// Mode for the state database: only the owner may read it.
pub const DATABASE_MODE: u32 = 0o600;

/// What exists before `init` runs, as seen by an adapter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observed {
    /// Which of [`paths_to_observe`] exist as directories.
    pub existing_dirs: BTreeSet<PathBuf>,
    /// The contents of `config.toml`, if it exists.
    pub config_text: Option<String>,
    /// The root recorded in the database, if the database exists.
    pub database_root: Option<PathBuf>,
    /// Clone directories found at `<root>/repos/<host>/<org>/<repo>`.
    pub clones: Vec<PathBuf>,
}

/// One thing `init` creates, in the order it must be created.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Create a directory with `mode`. Its parent exists by the time this runs.
    CreateDir {
        /// The directory.
        path: PathBuf,
        /// Its permission bits.
        mode: u32,
    },
    /// Write a new `config.toml`.
    WriteConfig {
        /// The file.
        path: PathBuf,
        /// Its contents.
        contents: String,
        /// Its permission bits.
        mode: u32,
    },
    /// Create the state database and record `root` in it.
    CreateDatabase {
        /// The database file.
        path: PathBuf,
        /// The root to record, as text: every path in [`Paths`] is valid UTF-8.
        root: String,
        /// Its permission bits.
        mode: u32,
    },
}

impl Step {
    /// The path this step creates.
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::CreateDir { path, .. }
            | Self::WriteConfig { path, .. }
            | Self::CreateDatabase { path, .. } => path,
        }
    }
}

/// A clone under the root that mori didn't make. Left untouched.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnmanagedRepo {
    /// The repo's identity, e.g. `github.com/acme/widget`.
    pub repo: String,
    /// The clone's directory.
    pub path: PathBuf,
}

/// What `init` will do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitPlan {
    /// The root being set up.
    pub root: PathBuf,
    /// What to create, parents first. Empty if mori is already set up.
    pub steps: Vec<Step>,
    /// Clones found under the root that mori didn't make, sorted by repo.
    pub unmanaged: Vec<UnmanagedRepo>,
}

impl InitPlan {
    /// True if there is nothing to create.
    #[must_use]
    pub fn already_initialized(&self) -> bool {
        self.steps.is_empty()
    }
}

/// The directories `init` ensures, with the mode each gets if created.
fn required_dirs(paths: &Paths) -> [(PathBuf, u32); 5] {
    [
        (paths.root.clone(), DIR_MODE),
        (paths.repos(), DIR_MODE),
        (paths.trees(), DIR_MODE),
        (paths.config_dir.clone(), DIR_MODE),
        (paths.state_dir.clone(), PRIVATE_DIR_MODE),
    ]
}

/// Every directory whose existence [`plan`] needs to know: the required directories and all
/// of their ancestors.
#[must_use]
pub fn paths_to_observe(paths: &Paths) -> BTreeSet<PathBuf> {
    required_dirs(paths)
        .iter()
        .flat_map(|(dir, _)| dir.ancestors().map(Path::to_path_buf))
        .collect()
}

/// Plans `init` from what the adapter observed.
///
/// # Errors
///
/// [`ConfigError::RootMismatch`] if the config or the database records a different root, and
/// [`ConfigError::ConfigInvalid`] if `config.toml` doesn't parse. Either way nothing should be
/// created.
pub fn plan(paths: &Paths, observed: &Observed) -> Result<InitPlan, ConfigError> {
    // Refusals come first, so a refused init creates nothing.
    if let Some(text) = &observed.config_text {
        let config = Config::parse(text, &paths.config_file)?;
        check_root(&config.root, &paths.root, RootSource::Config)?;
    }
    if let Some(recorded) = &observed.database_root {
        check_root(recorded, &paths.root, RootSource::Database)?;
    }

    let mut steps = Vec::new();
    let mut planned = BTreeSet::new();
    for (dir, mode) in required_dirs(paths) {
        for missing in missing_ancestors(&dir, &observed.existing_dirs, &planned) {
            let mode = if missing == dir { mode } else { DIR_MODE };
            planned.insert(missing.clone());
            steps.push(Step::CreateDir {
                path: missing,
                mode,
            });
        }
    }
    if observed.config_text.is_none() {
        steps.push(Step::WriteConfig {
            path: paths.config_file.clone(),
            contents: Config::render(&paths.root)?,
            mode: CONFIG_MODE,
        });
    }
    if observed.database_root.is_none() {
        steps.push(Step::CreateDatabase {
            path: paths.database.clone(),
            root: crate::paths::utf8(&paths.root)?.to_owned(),
            mode: DATABASE_MODE,
        });
    }

    Ok(InitPlan {
        root: paths.root.clone(),
        steps,
        unmanaged: unmanaged(paths, &observed.clones),
    })
}

fn check_root(
    recorded: &Path,
    effective: &Path,
    recorded_in: RootSource,
) -> Result<(), ConfigError> {
    if recorded == effective {
        Ok(())
    } else {
        Err(ConfigError::RootMismatch {
            recorded: recorded.to_path_buf(),
            effective: effective.to_path_buf(),
            recorded_in,
        })
    }
}

/// `dir` and its ancestors that neither exist nor are already planned, outermost first.
fn missing_ancestors(
    dir: &Path,
    existing: &BTreeSet<PathBuf>,
    planned: &BTreeSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut missing: Vec<PathBuf> = dir
        .ancestors()
        .take_while(|path| !existing.contains(*path) && !planned.contains(*path))
        .map(Path::to_path_buf)
        .collect();
    missing.reverse();
    missing
}

/// Every observed clone is unmanaged for now: mori doesn't manage any repos yet.
fn unmanaged(paths: &Paths, clones: &[PathBuf]) -> Vec<UnmanagedRepo> {
    let repos = paths.repos();
    let mut unmanaged: Vec<UnmanagedRepo> = clones
        .iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(&repos).ok()?;
            let parts: Vec<&str> = relative
                .iter()
                .map(|part| part.to_str())
                .collect::<Option<_>>()?;
            (parts.len() == 3).then(|| UnmanagedRepo {
                repo: parts.join("/"),
                path: path.clone(),
            })
        })
        .collect();
    unmanaged.sort();
    unmanaged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Env;

    fn paths() -> Paths {
        Paths::resolve(&Env {
            home: Some("/home/acme".into()),
            ..Env::default()
        })
        .unwrap()
    }

    /// A machine where only `/`, `/home` and `/home/acme` exist.
    fn fresh() -> Observed {
        Observed {
            existing_dirs: ["/", "/home", "/home/acme"]
                .into_iter()
                .map(PathBuf::from)
                .collect(),
            ..Observed::default()
        }
    }

    /// A machine where `init` has already run.
    fn initialized() -> Observed {
        let paths = paths();
        Observed {
            existing_dirs: paths_to_observe(&paths),
            config_text: Some(Config::render(&paths.root).unwrap()),
            database_root: Some(paths.root.clone()),
            clones: vec![],
        }
    }

    fn created(plan: &InitPlan) -> Vec<(&str, Option<u32>)> {
        plan.steps
            .iter()
            .map(|step| {
                let mode = match step {
                    Step::CreateDir { mode, .. } => Some(*mode),
                    _ => None,
                };
                (step.path().to_str().unwrap(), mode)
            })
            .collect()
    }

    #[test]
    fn a_fresh_machine_gets_everything_parents_first() {
        let plan = plan(&paths(), &fresh()).unwrap();

        assert_eq!(
            created(&plan),
            [
                ("/home/acme/mori", Some(DIR_MODE)),
                ("/home/acme/mori/repos", Some(DIR_MODE)),
                ("/home/acme/mori/trees", Some(DIR_MODE)),
                ("/home/acme/.config", Some(DIR_MODE)),
                ("/home/acme/.config/mori", Some(DIR_MODE)),
                ("/home/acme/.local", Some(DIR_MODE)),
                ("/home/acme/.local/state", Some(DIR_MODE)),
                ("/home/acme/.local/state/mori", Some(PRIVATE_DIR_MODE)),
                ("/home/acme/.config/mori/config.toml", None),
                ("/home/acme/.local/state/mori/mori.db", None),
            ]
        );
        assert!(!plan.already_initialized());
    }

    #[test]
    fn a_second_run_plans_nothing() {
        let plan = plan(&paths(), &initialized()).unwrap();

        assert!(plan.already_initialized());
    }

    #[test]
    fn missing_pieces_are_filled_in() {
        let observed = Observed {
            database_root: None,
            ..initialized()
        };

        let plan = plan(&paths(), &observed).unwrap();

        assert_eq!(
            created(&plan),
            [("/home/acme/.local/state/mori/mori.db", None)]
        );
    }

    #[test]
    fn a_different_root_in_the_config_is_refused() {
        let observed = Observed {
            config_text: Some(Config::render(Path::new("/home/acme/other")).unwrap()),
            ..initialized()
        };

        let error = plan(&paths(), &observed).unwrap_err();

        assert_eq!(
            error,
            ConfigError::RootMismatch {
                recorded: "/home/acme/other".into(),
                effective: "/home/acme/mori".into(),
                recorded_in: RootSource::Config,
            }
        );
        assert_eq!(error.reason(), "ROOT_MISMATCH");
    }

    #[test]
    fn a_different_root_in_the_database_is_refused() {
        let observed = Observed {
            database_root: Some("/home/acme/other".into()),
            ..initialized()
        };

        let error = plan(&paths(), &observed).unwrap_err();

        assert_eq!(error.reason(), "ROOT_MISMATCH");
    }

    #[test]
    fn an_invalid_config_is_refused() {
        let observed = Observed {
            config_text: Some("not toml".into()),
            ..initialized()
        };

        assert_eq!(
            plan(&paths(), &observed).unwrap_err().reason(),
            "CONFIG_INVALID"
        );
    }

    #[test]
    fn clones_are_listed_as_unmanaged_and_sorted() {
        let repos = paths().repos();
        let observed = Observed {
            clones: vec![
                repos.join("github.com/acme/widget"),
                repos.join("github.com/acme/anvil"),
                repos.join("too/shallow"),
            ],
            ..initialized()
        };

        let plan = plan(&paths(), &observed).unwrap();

        let names: Vec<&str> = plan
            .unmanaged
            .iter()
            .map(|repo| repo.repo.as_str())
            .collect();
        assert_eq!(names, ["github.com/acme/anvil", "github.com/acme/widget"]);
    }
}

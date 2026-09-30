//! mori's jj backend, driving the `jj` CLI.
//!
//! Reads pass `--ignore-working-copy`, so jj doesn't even snapshot a working copy while mori looks.
//! The only write so far is [`JjCli::clone_repo`], which makes a new directory and nothing else.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use mori_core::error::{Code, ErrorDetails};
use mori_core::forest::{Workspace, Workspaces};

/// Prints each workspace as two JSON strings, name then root, so any path parses exactly.
const WORKSPACE_TEMPLATE: &str = r#"json(name) ++ " " ++ json(root) ++ "\n""#;

/// Runs the `jj` program.
#[derive(Clone, Debug)]
pub struct JjCli {
    program: PathBuf,
    envs: Vec<(OsString, OsString)>,
}

impl JjCli {
    /// Uses the `jj` found on `PATH`.
    #[must_use]
    pub fn from_path() -> Self {
        Self::new("jj")
    }

    /// Uses `program` as `jj`.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            envs: Vec::new(),
        }
    }

    /// Sets an environment variable for every jj this runs, e.g. `JJ_CONFIG` in tests.
    #[must_use]
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }

    /// Clones `url` into `path`, which must not exist, as a jj repo that is also a git repo when
    /// `colocate` is set. The flag is always passed, so a user's `git.colocate` setting can't
    /// change what mori records.
    ///
    /// If the clone fails, whatever it left at `path` is removed: this call created it, so no one
    /// else's work is there. That is why an existing `path` is refused before jj runs.
    ///
    /// # Errors
    ///
    /// [`JjError::DestinationExists`] if anything is at `path` (it is left alone),
    /// [`JjError::Failed`] with jj's message if the clone fails, [`JjError::NotFound`] if jj isn't
    /// installed.
    pub fn clone_repo(&self, url: &str, path: &Path, colocate: bool) -> Result<(), JjError> {
        if path.symlink_metadata().is_ok() {
            return Err(JjError::DestinationExists {
                path: path.to_path_buf(),
            });
        }
        let colocate = if colocate {
            "--colocate"
        } else {
            "--no-colocate"
        };
        let output = self
            .command()
            .args([
                "--no-pager",
                "--color=never",
                "git",
                "clone",
                colocate,
                "--",
                url,
            ])
            .arg(path)
            .output()
            .map_err(|source| self.spawn_error(source))?;
        if output.status.success() {
            return Ok(());
        }
        // Best effort: the clone already failed, and that is the error worth reporting.
        let _ = std::fs::remove_dir_all(path);
        Err(JjError::Failed {
            clone: path.to_path_buf(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.envs(self.envs.iter().map(|(key, value)| (key, value)));
        command
    }

    fn spawn_error(&self, source: std::io::Error) -> JjError {
        match source.kind() {
            ErrorKind::NotFound => JjError::NotFound {
                program: self.program.clone(),
            },
            _ => JjError::Io {
                program: self.program.clone(),
                source,
            },
        }
    }

    /// Runs `jj` read-only against the repo at `clone` and returns its stdout.
    fn read(&self, clone: &Path, args: &[&str]) -> Result<String, JjError> {
        let output = self
            .command()
            .args(["--no-pager", "--color=never", "--ignore-working-copy"])
            .arg("--repository")
            .arg(clone)
            .args(args)
            .output()
            .map_err(|source| self.spawn_error(source))?;
        if !output.status.success() {
            return Err(JjError::Failed {
                clone: clone.to_path_buf(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        String::from_utf8(output.stdout).map_err(|_| JjError::OutputInvalid {
            output: "not UTF-8".to_owned(),
        })
    }
}

impl Workspaces for JjCli {
    type Error = JjError;

    fn list(&self, clone: &Path) -> Result<Vec<Workspace>, JjError> {
        let stdout = self.read(clone, &["workspace", "list", "-T", WORKSPACE_TEMPLATE])?;
        parse_workspace_list(&stdout)
    }
}

/// Parses the output of `jj workspace list` with [`WORKSPACE_TEMPLATE`].
fn parse_workspace_list(stdout: &str) -> Result<Vec<Workspace>, JjError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let invalid = || JjError::OutputInvalid {
                output: line.to_owned(),
            };
            let mut values = serde_json::Deserializer::from_str(line).into_iter::<String>();
            let (Some(Ok(name)), Some(Ok(root)), None) =
                (values.next(), values.next(), values.next())
            else {
                return Err(invalid());
            };
            Ok(Workspace {
                name,
                root: PathBuf::from(root),
            })
        })
        .collect()
}

/// Errors from the jj backend (domain `jj.mori`).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JjError {
    /// `jj` isn't installed, or isn't on `PATH`.
    #[error("can't find {}: install jj, or put it on PATH", program.display())]
    NotFound {
        /// The program mori tried to run.
        program: PathBuf,
    },

    /// `jj` couldn't be started.
    #[error("can't run {}: {source}", program.display())]
    Io {
        /// The program mori tried to run.
        program: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },

    /// Something is already where a clone would go.
    #[error("{} already exists; mori clones only into a new directory", path.display())]
    DestinationExists {
        /// The path.
        path: PathBuf,
    },

    /// `jj` ran and failed.
    #[error("jj failed in {}: {stderr}", clone.display())]
    Failed {
        /// The clone it ran against.
        clone: PathBuf,
        /// What jj said.
        stderr: String,
    },

    /// `jj` printed something mori can't read: probably a jj version it doesn't support.
    #[error("unexpected output from jj: {output:?}")]
    OutputInvalid {
        /// The part mori couldn't read.
        output: String,
    },
}

impl JjError {
    /// The AIP-193 `ErrorInfo.domain`.
    pub const DOMAIN: &'static str = "jj.mori";
}

impl ErrorDetails for JjError {
    fn code(&self) -> Code {
        match self {
            Self::NotFound { .. } => Code::FailedPrecondition,
            Self::DestinationExists { .. } => Code::AlreadyExists,
            Self::Io { .. } | Self::Failed { .. } | Self::OutputInvalid { .. } => Code::Internal,
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "JJ_NOT_FOUND",
            Self::DestinationExists { .. } => "PATH_EXISTS",
            Self::Io { .. } => "IO_ERROR",
            Self::Failed { .. } => "JJ_FAILED",
            Self::OutputInvalid { .. } => "JJ_OUTPUT_INVALID",
        }
    }

    fn domain(&self) -> &'static str {
        Self::DOMAIN
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::NotFound { program } | Self::Io { program, .. } => {
                vec![("program", program.display().to_string())]
            }
            Self::Failed { clone, .. } => vec![("clone", clone.display().to_string())],
            Self::DestinationExists { path } => vec![("path", path.display().to_string())],
            Self::OutputInvalid { .. } => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_lines_parse_exactly() {
        let stdout = concat!(
            "\"default\" \"/home/acme/mori/repos/github.com/acme/widget\"\n",
            "\"claude-fix-login\" \"/home/acme/mori/trees/widget/claude-fix-login\"\n",
            "\"odd\" \"/tmp/a \\\"quoted\\\" p\u{e4}th\"\n",
        );

        let workspaces = parse_workspace_list(stdout).unwrap();

        assert_eq!(
            workspaces,
            [
                Workspace {
                    name: "default".to_owned(),
                    root: PathBuf::from("/home/acme/mori/repos/github.com/acme/widget"),
                },
                Workspace {
                    name: "claude-fix-login".to_owned(),
                    root: PathBuf::from("/home/acme/mori/trees/widget/claude-fix-login"),
                },
                Workspace {
                    name: "odd".to_owned(),
                    root: PathBuf::from("/tmp/a \"quoted\" p\u{e4}th"),
                },
            ]
        );
    }

    #[test]
    fn no_output_means_no_workspaces() {
        assert_eq!(parse_workspace_list("").unwrap(), []);
    }

    #[test]
    fn unreadable_output_is_an_error() {
        for stdout in [
            "default: wskvyoyy 144bac1a (empty)\n",
            "\"only-a-name\"\n",
            "\"a\" \"/b\" \"extra\"\n",
            "\"a\" 42\n",
        ] {
            let error = parse_workspace_list(stdout).unwrap_err();

            assert_eq!(error.reason(), "JJ_OUTPUT_INVALID", "for {stdout:?}");
        }
    }

    #[test]
    fn an_existing_destination_is_refused_and_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let clone = dir.path().join("widget");
        std::fs::create_dir(&clone).unwrap();
        std::fs::write(clone.join("notes.txt"), "someone's work").unwrap();

        let error = JjCli::new("/nonexistent/jj")
            .clone_repo("https://github.com/acme/widget", &clone, true)
            .unwrap_err();

        assert_eq!(error.reason(), "PATH_EXISTS");
        assert_eq!(
            std::fs::read_to_string(clone.join("notes.txt")).unwrap(),
            "someone's work"
        );
    }

    #[test]
    fn a_missing_jj_is_a_failed_precondition() {
        let jj = JjCli::new("/nonexistent/jj");

        let error = jj.list(Path::new("/nonexistent/clone")).unwrap_err();

        assert_eq!(error.code(), Code::FailedPrecondition);
        assert_eq!(error.reason(), "JJ_NOT_FOUND");
    }

    /// The pinned jj under `bazel test` (which sets `MORI_TEST_JJ`), run with a throwaway
    /// config in `dir`. Plain `cargo test` has no pinned jj, so there the real-jj tests say so and
    /// pass without running.
    fn pinned_jj(dir: &Path) -> Result<Option<JjCli>, Box<dyn std::error::Error>> {
        let Some(program) = std::env::var_os("MORI_TEST_JJ") else {
            eprintln!("skipped: MORI_TEST_JJ isn't set; `bazel test` sets it to the pinned jj");
            return Ok(None);
        };
        let config = dir.join("jj.toml");
        std::fs::write(
            &config,
            "user.name = \"Test\"\nuser.email = \"test@example.com\"\n",
        )?;
        Ok(Some(
            JjCli::new(std::fs::canonicalize(program)?).env("JJ_CONFIG", config),
        ))
    }

    /// Runs `jj args` for test setup, failing the test if it fails.
    fn run(jj: &JjCli, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let output = jj.command().args(args).output()?;
        assert!(
            output.status.success(),
            "jj {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    /// A repo with one commit on `main`, to clone from.
    fn source_repo(jj: &JjCli, dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let source = dir.join("source");
        run(
            jj,
            &["git", "init", "--colocate", &source.display().to_string()],
        )?;
        std::fs::write(source.join("README.md"), "widget\n")?;
        let repo = source.display().to_string();
        run(jj, &["--repository", &repo, "commit", "--message", "first"])?;
        run(
            jj,
            &[
                "--repository",
                &repo,
                "bookmark",
                "create",
                "main",
                "--revision",
                "@-",
            ],
        )?;
        Ok(source)
    }

    #[test]
    fn lists_the_workspaces_of_a_real_repo() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = dir.path().join("widget");
        let tree = dir.path().join("claude-fix-login");
        run(&jj, &["git", "init", &clone.display().to_string()])?;
        run(
            &jj,
            &[
                "--repository",
                &clone.display().to_string(),
                "workspace",
                "add",
                "--name",
                "claude-fix-login",
                &tree.display().to_string(),
            ],
        )?;

        let mut workspaces = jj.list(&clone)?;
        workspaces.sort_by(|a, b| a.name.cmp(&b.name));

        let names: Vec<_> = workspaces.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, ["claude-fix-login", "default"]);
        assert_eq!(workspaces[0].root.canonicalize()?, tree.canonicalize()?);
        assert_eq!(workspaces[1].root.canonicalize()?, clone.canonicalize()?);
        Ok(())
    }

    #[test]
    fn clones_a_colocated_repo() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let source = source_repo(&jj, dir.path())?;
        let clone = dir.path().join("widget");

        jj.clone_repo(&source.display().to_string(), &clone, true)?;

        assert!(clone.join(".jj").is_dir());
        assert!(
            clone.join(".git").exists(),
            "colocated means a git repo too"
        );
        assert_eq!(
            std::fs::read_to_string(clone.join("README.md"))?,
            "widget\n"
        );
        let names: Vec<_> = jj.list(&clone)?.into_iter().map(|w| w.name).collect();
        assert_eq!(names, ["default"]);
        Ok(())
    }

    #[test]
    fn clones_a_jj_only_repo() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let source = source_repo(&jj, dir.path())?;
        let clone = dir.path().join("widget");

        jj.clone_repo(&source.display().to_string(), &clone, false)?;

        assert!(clone.join(".jj").is_dir());
        assert!(!clone.join(".git").exists());
        Ok(())
    }

    #[test]
    fn a_failed_clone_leaves_nothing_behind() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = dir.path().join("widget");

        let missing = dir.path().join("no-such-repo").display().to_string();
        let error = jj.clone_repo(&missing, &clone, true).unwrap_err();

        assert_eq!(error.reason(), "JJ_FAILED");
        assert!(!clone.exists());
        Ok(())
    }
}

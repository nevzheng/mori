//! mori's jj backend, driving the `jj` CLI.
//!
//! Every call here is read-only: it passes `--ignore-working-copy`, so jj doesn't even snapshot a
//! working copy while mori looks.

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
        }
    }

    /// Runs `jj` read-only against the repo at `clone` and returns its stdout.
    fn read(&self, clone: &Path, args: &[&str]) -> Result<String, JjError> {
        let output = Command::new(&self.program)
            .args(["--no-pager", "--color=never", "--ignore-working-copy"])
            .arg("--repository")
            .arg(clone)
            .args(args)
            .output()
            .map_err(|source| match source.kind() {
                ErrorKind::NotFound => JjError::NotFound {
                    program: self.program.clone(),
                },
                _ => JjError::Io {
                    program: self.program.clone(),
                    source,
                },
            })?;
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
            Self::Io { .. } | Self::Failed { .. } | Self::OutputInvalid { .. } => Code::Internal,
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "JJ_NOT_FOUND",
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
    fn a_missing_jj_is_a_failed_precondition() {
        let jj = JjCli::new("/nonexistent/jj");

        let error = jj.list(Path::new("/nonexistent/clone")).unwrap_err();

        assert_eq!(error.code(), Code::FailedPrecondition);
        assert_eq!(error.reason(), "JJ_NOT_FOUND");
    }

    /// Runs the real `jj`. Ignored by default because CI doesn't install jj yet; run it with
    /// `cargo test -p mori-jj -- --include-ignored`.
    #[test]
    #[ignore = "needs jj on PATH"]
    fn lists_the_workspaces_of_a_real_repo() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let clone = dir.path().join("widget");
        let tree = dir.path().join("claude-fix-login");
        let config = dir.path().join("jj.toml");
        std::fs::write(
            &config,
            "user.name = \"Test\"\nuser.email = \"test@example.com\"\n",
        )?;
        let jj = |args: &[&str]| {
            Command::new("jj")
                .env("JJ_CONFIG", &config)
                .args(args)
                .status()
                .map(|status| assert!(status.success(), "jj {args:?} failed"))
        };
        jj(&["git", "init", &clone.display().to_string()])?;
        jj(&[
            "--repository",
            &clone.display().to_string(),
            "workspace",
            "add",
            "--name",
            "claude-fix-login",
            &tree.display().to_string(),
        ])?;

        let mut workspaces = JjCli::from_path().list(&clone)?;
        workspaces.sort_by(|a, b| a.name.cmp(&b.name));

        let names: Vec<_> = workspaces.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, ["claude-fix-login", "default"]);
        assert_eq!(workspaces[0].root.canonicalize()?, tree.canonicalize()?);
        assert_eq!(workspaces[1].root.canonicalize()?, clone.canonicalize()?);
        Ok(())
    }
}

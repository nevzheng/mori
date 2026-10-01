//! mori's jj backend, driving the `jj` CLI.
//!
//! Reads pass `--ignore-working-copy`, so jj doesn't even snapshot a working copy while mori looks.
//! The writes, [`JjCli::clone_repo`] and [`JjCli::add_workspace`], each make one new directory and
//! clean up after themselves if jj fails. [`JjCli::snapshot`] and [`JjCli::forget_workspace`] are
//! for removing a tree, and change only jj's own records.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use mori_core::error::{Code, ErrorDetails};
use mori_core::forest::{TreeState, Workspace, Workspaces};
pub use mori_core::vcs::RemoteBookmark;
use mori_core::vcs::Vcs;

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

    /// Adds a workspace named `name` to the clone at `clone`, with its working copy at `path` (which
    /// must not exist) on a new change on top of `from`, a jj revset such as `trunk()`.
    ///
    /// If jj fails, whatever it left is undone: the directory is removed and a workspace it
    /// registered is forgotten. This call created both, so no one else's work is there.
    ///
    /// # Errors
    ///
    /// [`JjError::DestinationExists`] if anything is at `path` (it is left alone),
    /// [`JjError::Failed`] with jj's message if jj fails (for example, `from` matches nothing),
    /// [`JjError::NotFound`] if jj isn't installed.
    pub fn add_workspace(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        from: &str,
    ) -> Result<(), JjError> {
        if path.symlink_metadata().is_ok() {
            return Err(JjError::DestinationExists {
                path: path.to_path_buf(),
            });
        }
        let output = self
            .command()
            .args(["--no-pager", "--color=never", "--repository"])
            .arg(clone)
            .args(["workspace", "add", "--name", name, "--revision", from, "--"])
            .arg(path)
            .output()
            .map_err(|source| self.spawn_error(source))?;
        if output.status.success() {
            return Ok(());
        }
        // Best effort: jj already failed, and that is the error worth reporting.
        let _ = std::fs::remove_dir_all(path);
        let _ = self
            .command()
            .args(["--no-pager", "--color=never", "--repository"])
            .arg(clone)
            .args(["workspace", "forget", "--", name])
            .output();
        Err(JjError::Failed {
            clone: clone.to_path_buf(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    }

    /// Snapshots the working copy of the tree at `tree`, so jj records edits made since it last
    /// ran there. Files aren't changed; only jj's record of them is.
    ///
    /// # Errors
    ///
    /// [`JjError::Failed`] with jj's message, [`JjError::NotFound`] if jj isn't installed.
    pub fn snapshot(&self, tree: &Path) -> Result<(), JjError> {
        self.write(tree, &["util", "snapshot"])
    }

    /// Makes the clone at `clone` forget the workspace named `name`. The workspace's files stay;
    /// deleting them is the caller's next step.
    ///
    /// # Errors
    ///
    /// [`JjError::Failed`] with jj's message, [`JjError::NotFound`] if jj isn't installed.
    pub fn forget_workspace(&self, clone: &Path, name: &str) -> Result<(), JjError> {
        self.write(clone, &["workspace", "forget", "--", name])
    }

    /// Runs `jj args` in the repo at `repo`, letting it record what it does.
    fn write(&self, repo: &Path, args: &[&str]) -> Result<(), JjError> {
        let output = self
            .command()
            .args(["--no-pager", "--color=never", "--repository"])
            .arg(repo)
            .args(args)
            .output()
            .map_err(|source| self.spawn_error(source))?;
        if output.status.success() {
            return Ok(());
        }
        Err(JjError::Failed {
            clone: repo.to_path_buf(),
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

    fn state(&self, clone: &Path, name: &str) -> Result<TreeState, JjError> {
        self.state_covering(clone, name, &[])
    }
}

/// Lists remote bookmarks as tab-separated name, remote and commit ID. A conflicted bookmark
/// has no single target and is left out.
const REMOTE_BOOKMARK_TEMPLATE: &str = r#"if(remote && normal_target, name ++ "\t" ++ remote ++ "\t" ++ normal_target.commit_id() ++ "\n")"#;

impl JjCli {
    /// Like [`Workspaces::state`], but changes that are ancestors of any of `landed` (commit IDs
    /// of work that landed, e.g. a squash-merged bookmark's last target) count as saved too.
    ///
    /// # Errors
    ///
    /// As [`Workspaces::state`].
    pub fn state_covering(
        &self,
        clone: &Path,
        name: &str,
        landed: &[String],
    ) -> Result<TreeState, JjError> {
        let working_copy = format!("{}@", revset_string(name));
        let head = self.read(
            clone,
            &[
                "log",
                "--no-graph",
                "-r",
                &working_copy,
                "-T",
                STATE_TEMPLATE,
            ],
        )?;
        let saved = std::iter::once("remote_bookmarks() | trunk()".to_owned())
            .chain(landed.iter().map(|commit| revset_string(commit)))
            .collect::<Vec<_>>()
            .join(" | ");
        let unpushed = self.read(
            clone,
            &[
                "log",
                "--no-graph",
                "-r",
                &format!("(::{working_copy} ~ ::({saved})) ~ empty()"),
                "-T",
                UNPUSHED_TEMPLATE,
            ],
        )?;
        parse_state(&head, &unpushed)
    }

    /// The full ID of the commit the tree's working copy is on.
    ///
    /// # Errors
    ///
    /// When jj can't be run or has no such workspace.
    pub fn working_copy_commit(&self, clone: &Path, name: &str) -> Result<String, JjError> {
        let working_copy = format!("{}@", revset_string(name));
        Ok(self
            .read(
                clone,
                &["log", "--no-graph", "-r", &working_copy, "-T", "commit_id"],
            )?
            .trim()
            .to_owned())
    }

    /// The remote bookmarks pointing into the tree's own history (its changes not in trunk).
    /// The colocated `git` pseudo-remote is left out.
    ///
    /// # Errors
    ///
    /// When jj can't be run or its answer can't be read.
    pub fn pushed_bookmarks(
        &self,
        clone: &Path,
        name: &str,
    ) -> Result<Vec<RemoteBookmark>, JjError> {
        let own = format!("::{}@ ~ ::trunk()", revset_string(name));
        let stdout = self.read(
            clone,
            &[
                "bookmark",
                "list",
                "--all-remotes",
                "-r",
                &own,
                "-T",
                REMOTE_BOOKMARK_TEMPLATE,
            ],
        )?;
        parse_remote_bookmarks(&stdout)
    }

    /// When the tree last changed: the latest committer time among its own changes and its
    /// working copy, in seconds since the Unix epoch.
    ///
    /// # Errors
    ///
    /// When jj can't be run or its answer can't be read.
    pub fn last_change(&self, clone: &Path, name: &str) -> Result<u64, JjError> {
        let working_copy = format!("{}@", revset_string(name));
        let stdout = self.read(
            clone,
            &[
                "log",
                "--no-graph",
                "-r",
                &format!("latest((::{working_copy} ~ ::trunk()) | {working_copy})"),
                "-T",
                r#"committer.timestamp().utc().format("%s") ++ "\n""#,
            ],
        )?;
        stdout.trim().parse().map_err(|_| JjError::OutputInvalid {
            output: stdout.clone(),
        })
    }

    /// The git repository behind the clone: `.git` when colocated, else jj's own store.
    ///
    /// # Errors
    ///
    /// [`JjError::OutputInvalid`] if the clone's `.jj/repo/store/git_target` can't be read.
    pub fn git_dir(clone: &Path) -> Result<PathBuf, JjError> {
        let store = clone.join(".jj/repo/store");
        let target = std::fs::read_to_string(store.join("git_target")).map_err(|_| {
            JjError::OutputInvalid {
                output: format!("no git_target in {}", store.display()),
            }
        })?;
        Ok(store.join(target.trim()))
    }

    /// Points the git ref `name` (e.g. `refs/mori/removed/<entry>`) at `commit_id`, so git's and
    /// jj's garbage collection keep the commit. Refs outside `refs/heads` and `refs/tags` aren't
    /// jj bookmarks, so the pin is never shown or pushed.
    ///
    /// # Errors
    ///
    /// [`JjError::Failed`] if git refuses (for example, no such commit).
    pub fn pin(&self, clone: &Path, name: &str, commit_id: &str) -> Result<(), JjError> {
        self.git(clone, &["update-ref", name, commit_id])
            .map(|_| ())
    }

    /// Whether the commit exists in the clone's git repository.
    ///
    /// # Errors
    ///
    /// [`JjError::NotFound`] if git isn't installed.
    pub fn commit_exists(&self, clone: &Path, commit_id: &str) -> Result<bool, JjError> {
        match self.git(
            clone,
            &["cat-file", "-e", &format!("{commit_id}^{{commit}}")],
        ) {
            Ok(_) => Ok(true),
            Err(JjError::Failed { .. }) => Ok(false),
            Err(other) => Err(other),
        }
    }

    /// Adds a workspace named `name` at `path` on a new change on top of `commit_id`, even if jj
    /// has let go of that commit: a temporary bookmark imports it first, then goes away.
    ///
    /// # Errors
    ///
    /// As [`JjCli::add_workspace`], and [`JjError::Failed`] if the commit can't be imported.
    pub fn add_workspace_at(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        commit_id: &str,
    ) -> Result<(), JjError> {
        let temporary = format!("refs/heads/mori-restore-{name}");
        self.git(clone, &["update-ref", &temporary, commit_id])?;
        let imported = self.write(clone, &["git", "import"]);
        let added = imported
            .and_then(|()| self.add_workspace(clone, name, path, &revset_string(commit_id)));
        // Best effort: the workspace is what matters; the temporary bookmark only carried the
        // commit in.
        let _ = self.git(clone, &["update-ref", "-d", &temporary]);
        let _ = self.write(clone, &["git", "import"]);
        added
    }

    /// Runs `git` against the clone's git repository and returns its stdout.
    fn git(&self, clone: &Path, args: &[&str]) -> Result<String, JjError> {
        let git_dir = Self::git_dir(clone)?;
        let output = Command::new("git")
            .envs(self.envs.iter().map(|(key, value)| (key, value)))
            .arg("--git-dir")
            .arg(&git_dir)
            .args(args)
            .output()
            .map_err(|source| match source.kind() {
                ErrorKind::NotFound => JjError::NotFound {
                    program: PathBuf::from("git"),
                },
                _ => JjError::Io {
                    program: PathBuf::from("git"),
                    source,
                },
            })?;
        if !output.status.success() {
            return Err(JjError::Failed {
                clone: clone.to_path_buf(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Fetches the clone's remotes, so its remote bookmarks are current. Changes only jj's
    /// record of the remote.
    ///
    /// # Errors
    ///
    /// [`JjError::Failed`] with jj's message (for example, no network).
    pub fn fetch(&self, clone: &Path) -> Result<(), JjError> {
        self.write(clone, &["git", "fetch"])
    }
}

/// Every method is the inherent one of the same name; the trait lets flows take any backend.
impl Vcs for JjCli {
    fn clone_repo(&self, url: &str, path: &Path, colocate: bool) -> Result<(), JjError> {
        Self::clone_repo(self, url, path, colocate)
    }

    fn add_tree(&self, clone: &Path, name: &str, path: &Path, from: &str) -> Result<(), JjError> {
        self.add_workspace(clone, name, path, from)
    }

    fn add_tree_at(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        commit_id: &str,
    ) -> Result<(), JjError> {
        self.add_workspace_at(clone, name, path, commit_id)
    }

    fn snapshot(&self, tree: &Path) -> Result<(), JjError> {
        Self::snapshot(self, tree)
    }

    fn forget_tree(&self, clone: &Path, name: &str) -> Result<(), JjError> {
        self.forget_workspace(clone, name)
    }

    fn state_covering(
        &self,
        clone: &Path,
        name: &str,
        landed: &[String],
    ) -> Result<TreeState, JjError> {
        Self::state_covering(self, clone, name, landed)
    }

    fn working_copy_commit(&self, clone: &Path, name: &str) -> Result<String, JjError> {
        Self::working_copy_commit(self, clone, name)
    }

    fn pushed_bookmarks(&self, clone: &Path, name: &str) -> Result<Vec<RemoteBookmark>, JjError> {
        Self::pushed_bookmarks(self, clone, name)
    }

    fn last_change(&self, clone: &Path, name: &str) -> Result<u64, JjError> {
        Self::last_change(self, clone, name)
    }

    fn pin(&self, clone: &Path, name: &str, commit_id: &str) -> Result<(), JjError> {
        Self::pin(self, clone, name, commit_id)
    }

    fn commit_exists(&self, clone: &Path, commit_id: &str) -> Result<bool, JjError> {
        Self::commit_exists(self, clone, commit_id)
    }

    fn fetch(&self, clone: &Path) -> Result<(), JjError> {
        Self::fetch(self, clone)
    }
}

/// Parses [`REMOTE_BOOKMARK_TEMPLATE`] output, leaving out the `git` pseudo-remote.
fn parse_remote_bookmarks(stdout: &str) -> Result<Vec<RemoteBookmark>, JjError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            match parts[..] {
                [_, "git", _] => None,
                [name, remote, commit_id] => Some(Ok(RemoteBookmark {
                    name: name.to_owned(),
                    remote: remote.to_owned(),
                    commit_id: commit_id.to_owned(),
                })),
                _ => Some(Err(JjError::OutputInvalid {
                    output: line.to_owned(),
                })),
            }
        })
        .collect()
}

/// Prints the working-copy change's ID, its commit ID and whether it has edits.
const STATE_TEMPLATE: &str =
    r#"change_id.short(12) ++ " " ++ commit_id ++ " " ++ if(empty, "clean", "changed") ++ "\n""#;

/// Prints a commit ID per line: the unpushed changes in [`JjCli::state_covering`].
const UNPUSHED_TEMPLATE: &str = r#"commit_id ++ "\n""#;

/// Quotes `name` as a revset string, so any workspace name, even a foreign one, is taken
/// literally.
fn revset_string(name: &str) -> String {
    let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Parses [`STATE_TEMPLATE`] output and the commit IDs of the unpushed changes
/// ([`UNPUSHED_TEMPLATE`]). Edits in the working copy count as changed only while the working
/// copy is itself unpushed: once a remote bookmark, trunk or landed work covers it, they're saved.
fn parse_state(head: &str, unpushed: &str) -> Result<TreeState, JjError> {
    let invalid = || JjError::OutputInvalid {
        output: head.to_owned(),
    };
    let mut fields = head.split_whitespace();
    let (Some(change), Some(commit), Some(edits), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(invalid());
    };
    let edited = match edits {
        "changed" => true,
        "clean" => false,
        _ => return Err(invalid()),
    };
    let unpushed: Vec<&str> = unpushed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let changed = edited && unpushed.contains(&commit);
    let unpushed = u32::try_from(unpushed.len()).map_err(|_| invalid())?;
    Ok(TreeState {
        change: change.to_owned(),
        changed,
        unpushed,
    })
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

    /// A clone of [`source_repo`], so `trunk()` resolves to its `main`.
    fn cloned(jj: &JjCli, dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let source = source_repo(jj, dir)?;
        let clone = dir.join("widget");
        jj.clone_repo(&source.display().to_string(), &clone, true)?;
        Ok(clone)
    }

    #[test]
    fn adds_a_workspace_on_trunk() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = cloned(&jj, dir.path())?;
        let tree = dir.path().join("trees").join("claude-fix-login");
        std::fs::create_dir(dir.path().join("trees"))?;

        jj.add_workspace(&clone, "claude-fix-login", &tree, "trunk()")?;

        assert_eq!(std::fs::read_to_string(tree.join("README.md"))?, "widget\n");
        let mut names: Vec<_> = jj.list(&clone)?.into_iter().map(|w| w.name).collect();
        names.sort();
        assert_eq!(names, ["claude-fix-login", "default"]);
        Ok(())
    }

    #[test]
    fn a_failed_add_leaves_nothing_behind() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = cloned(&jj, dir.path())?;
        let tree = dir.path().join("claude-fix-login");

        let error = jj
            .add_workspace(&clone, "claude-fix-login", &tree, "no_such_bookmark")
            .unwrap_err();

        assert_eq!(error.reason(), "JJ_FAILED");
        assert!(!tree.exists());
        let names: Vec<_> = jj.list(&clone)?.into_iter().map(|w| w.name).collect();
        assert_eq!(names, ["default"]);
        Ok(())
    }

    #[test]
    fn an_existing_tree_path_is_refused_and_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("claude-fix-login");
        std::fs::create_dir(&tree).unwrap();
        std::fs::write(tree.join("notes.txt"), "someone's work").unwrap();

        let error = JjCli::new("/nonexistent/jj")
            .add_workspace(dir.path(), "claude-fix-login", &tree, "trunk()")
            .unwrap_err();

        assert_eq!(error.reason(), "PATH_EXISTS");
        assert_eq!(
            std::fs::read_to_string(tree.join("notes.txt")).unwrap(),
            "someone's work"
        );
    }

    #[test]
    fn state_lines_parse() {
        let state = parse_state("vmvywosutlnw c0ffee changed\n", "abc\nc0ffee\n").unwrap();

        assert_eq!(
            state,
            TreeState {
                change: "vmvywosutlnw".to_owned(),
                changed: true,
                unpushed: 2,
            }
        );
        assert!(
            !parse_state("vmvywosutlnw c0ffee clean\n", "")
                .unwrap()
                .changed
        );
        assert!(parse_state("vmvywosutlnw c0ffee dirty\n", "").is_err());
        assert!(parse_state("vmvywosutlnw changed\n", "").is_err());
        assert!(parse_state("", "").is_err());
    }

    #[test]
    fn a_working_copy_already_saved_is_not_changed() {
        // Edits in the working copy that is itself on a remote bookmark (so not unpushed) are
        // saved.
        let state = parse_state("vmvywosutlnw c0ffee changed\n", "abc\n").unwrap();

        assert!(!state.changed);
        assert_eq!(state.unpushed, 1);
    }

    #[test]
    fn revset_strings_are_quoted() {
        assert_eq!(revset_string("claude-fix-login"), r#""claude-fix-login""#);
        assert_eq!(revset_string(r#"odd "one""#), r#""odd \"one\"""#);
    }

    #[test]
    fn reads_a_trees_state() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = cloned(&jj, dir.path())?;
        let tree = dir.path().join("claude-fix-login");
        jj.add_workspace(&clone, "claude-fix-login", &tree, "trunk()")?;

        let fresh = jj.state(&clone, "claude-fix-login")?;
        assert!(!fresh.changed);
        assert_eq!(fresh.unpushed, 0);

        // Edit, then let jj snapshot the edit (as it does whenever someone runs jj in the tree).
        std::fs::write(tree.join("login.rs"), "fn login() {}\n")?;
        run(
            &jj,
            &["--repository", &tree.display().to_string(), "status"],
        )?;
        let edited = jj.state(&clone, "claude-fix-login")?;
        assert!(edited.changed);
        assert_eq!(edited.unpushed, 1);
        assert_ne!(edited.change, "");

        // Commit it: the working copy is clean again, but the work is still only local.
        let tree_arg = tree.display().to_string();
        run(
            &jj,
            &["--repository", &tree_arg, "commit", "--message", "login"],
        )?;
        let committed = jj.state(&clone, "claude-fix-login")?;
        assert!(!committed.changed);
        assert_eq!(committed.unpushed, 1);
        Ok(())
    }

    #[test]
    fn a_snapshot_makes_fresh_edits_visible() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = cloned(&jj, dir.path())?;
        let tree = dir.path().join("claude-fix-login");
        jj.add_workspace(&clone, "claude-fix-login", &tree, "trunk()")?;
        std::fs::write(tree.join("login.rs"), "fn login() {}\n")?;
        assert!(
            !jj.state(&clone, "claude-fix-login")?.changed,
            "not yet snapshotted"
        );

        jj.snapshot(&tree)?;

        assert!(jj.state(&clone, "claude-fix-login")?.changed);
        Ok(())
    }

    #[test]
    fn forgetting_a_workspace_keeps_its_files() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = cloned(&jj, dir.path())?;
        let tree = dir.path().join("claude-fix-login");
        jj.add_workspace(&clone, "claude-fix-login", &tree, "trunk()")?;

        jj.forget_workspace(&clone, "claude-fix-login")?;

        let names: Vec<_> = jj.list(&clone)?.into_iter().map(|w| w.name).collect();
        assert_eq!(names, ["default"]);
        assert!(tree.join("README.md").exists());
        Ok(())
    }

    #[test]
    fn remote_bookmark_lines_parse_without_the_git_pseudo_remote() {
        let stdout = "main\tgit\taaaa\nclaude/fix-login\torigin\tbbbb\n";

        assert_eq!(
            parse_remote_bookmarks(stdout).unwrap(),
            [RemoteBookmark {
                name: "claude/fix-login".to_owned(),
                remote: "origin".to_owned(),
                commit_id: "bbbb".to_owned(),
            }]
        );
        assert!(parse_remote_bookmarks("just-a-name\n").is_err());
    }

    /// A clone of [`source_repo`] with a tree `claude-fix-login` holding one committed change.
    fn tree_with_a_commit(
        jj: &JjCli,
        dir: &Path,
    ) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
        let clone = cloned(jj, dir)?;
        let tree = dir.join("claude-fix-login");
        jj.add_workspace(&clone, "claude-fix-login", &tree, "trunk()")?;
        std::fs::write(tree.join("login.rs"), "fn login() {}\n")?;
        let tree_arg = tree.display().to_string();
        run(
            jj,
            &["--repository", &tree_arg, "commit", "--message", "login"],
        )?;
        Ok((clone, tree))
    }

    #[test]
    fn pushed_bookmarks_and_landed_work() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let (clone, tree) = tree_with_a_commit(&jj, dir.path())?;
        let tree_arg = tree.display().to_string();
        assert_eq!(jj.pushed_bookmarks(&clone, "claude-fix-login")?, []);
        assert_eq!(jj.state(&clone, "claude-fix-login")?.unpushed, 1);

        run(
            &jj,
            &[
                "--repository",
                &tree_arg,
                "bookmark",
                "create",
                "claude/fix-login",
                "-r",
                "@-",
            ],
        )?;
        run(
            &jj,
            &[
                "--repository",
                &tree_arg,
                "git",
                "push",
                "--bookmark",
                "claude/fix-login",
            ],
        )?;
        let pushed = jj.pushed_bookmarks(&clone, "claude-fix-login")?;
        assert_eq!(pushed.len(), 1);
        assert_eq!(pushed[0].name, "claude/fix-login");
        assert_eq!(pushed[0].remote, "origin");
        assert_eq!(jj.state(&clone, "claude-fix-login")?.unpushed, 0);

        // The remote deletes the bookmark (as after a squash merge): the work is only local again,
        // unless the commit the bookmark last pointed to counts as landed.
        run(
            &jj,
            &[
                "--repository",
                &tree_arg,
                "bookmark",
                "delete",
                "claude/fix-login",
            ],
        )?;
        run(
            &jj,
            &["--repository", &tree_arg, "git", "push", "--deleted"],
        )?;
        jj.fetch(&clone)?;
        assert_eq!(jj.pushed_bookmarks(&clone, "claude-fix-login")?, []);
        assert_eq!(jj.state(&clone, "claude-fix-login")?.unpushed, 1);
        let landed = [pushed[0].commit_id.clone()];
        assert_eq!(
            jj.state_covering(&clone, "claude-fix-login", &landed)?
                .unpushed,
            0
        );
        Ok(())
    }

    #[test]
    fn a_pushed_working_copy_is_saved() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let clone = cloned(&jj, dir.path())?;
        let tree = dir.path().join("claude-fix-login");
        jj.add_workspace(&clone, "claude-fix-login", &tree, "trunk()")?;
        let tree_arg = tree.display().to_string();

        // Describe the edit instead of committing it: it stays the working-copy change.
        std::fs::write(tree.join("login.rs"), "fn login() {}\n")?;
        run(
            &jj,
            &["--repository", &tree_arg, "describe", "--message", "login"],
        )?;
        let described = jj.state(&clone, "claude-fix-login")?;
        assert!(described.changed);
        assert_eq!(described.unpushed, 1);

        // Push the working-copy change itself: a remote bookmark now points at it.
        run(
            &jj,
            &[
                "--repository",
                &tree_arg,
                "bookmark",
                "create",
                "claude/fix-login",
                "-r",
                "@",
            ],
        )?;
        run(
            &jj,
            &[
                "--repository",
                &tree_arg,
                "git",
                "push",
                "--bookmark",
                "claude/fix-login",
            ],
        )?;
        let pushed = jj.state(&clone, "claude-fix-login")?;
        assert!(!pushed.changed, "the pushed working copy is saved");
        assert_eq!(pushed.unpushed, 0);

        // A further edit to the working copy isn't on the remote, so it counts again.
        std::fs::write(tree.join("login.rs"), "fn login() { check() }\n")?;
        jj.snapshot(&tree)?;
        assert!(jj.state(&clone, "claude-fix-login")?.changed);
        Ok(())
    }

    #[test]
    fn last_change_is_a_recent_epoch_time() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        let (clone, _) = tree_with_a_commit(&jj, dir.path())?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let changed = jj.last_change(&clone, "claude-fix-login")?;

        assert!(now.abs_diff(changed) < 3600, "{changed} vs {now}");
        Ok(())
    }

    #[test]
    fn a_pinned_commit_brings_a_removed_tree_back() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let Some(jj) = pinned_jj(dir.path())? else {
            return Ok(());
        };
        for colocate in [true, false] {
            let source = source_repo(&jj, dir.path())?;
            let clone = dir.path().join(format!("widget-{colocate}"));
            jj.clone_repo(&source.display().to_string(), &clone, colocate)?;
            let tree = dir.path().join(format!("claude-fix-login-{colocate}"));
            jj.add_workspace(&clone, "claude-fix-login", &tree, "trunk()")?;
            std::fs::write(tree.join("login.rs"), "fn login() {}\n")?;
            jj.snapshot(&tree)?;
            let commit = jj
                .read(
                    &clone,
                    &[
                        "log",
                        "--no-graph",
                        "-r",
                        "\"claude-fix-login\"@",
                        "-T",
                        "commit_id",
                    ],
                )?
                .trim()
                .to_owned();

            // Remove it as mori does: pin, forget, delete.
            jj.pin(&clone, "refs/mori/removed/j-test", &commit)?;
            jj.forget_workspace(&clone, "claude-fix-login")?;
            std::fs::remove_dir_all(&tree)?;
            assert!(jj.commit_exists(&clone, &commit)?);
            assert!(!jj.commit_exists(&clone, "0123456789abcdef0123456789abcdef01234567")?);

            jj.add_workspace_at(&clone, "claude-fix-login", &tree, &commit)?;

            assert_eq!(
                std::fs::read_to_string(tree.join("login.rs"))?,
                "fn login() {}\n",
                "colocated: {colocate}"
            );
            let bookmarks = jj.read(&clone, &["bookmark", "list", "-T", "name ++ \"\\n\""])?;
            assert!(!bookmarks.contains("mori-restore"), "{bookmarks}");
            std::fs::remove_dir_all(&source)?;
        }
        Ok(())
    }
}

//! mori's git backend: clones are plain git repos and trees are detached `git worktree`s, driven
//! through the `git` CLI.
//!
//! A tree's name is its worktree directory's name, which is also git's own name for it under
//! `.git/worktrees/`; the clone's main worktree is the tree named `default`.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use mori_core::clone::BASE_TREE_NAME;
use mori_core::error::{Code, ErrorDetails};
use mori_core::forest::{TreeState, Workspace, Workspaces};
use mori_core::vcs::{RemoteBookmark, Vcs};

/// Where trunk may be, in order, as jj's `trunk()` looks: the remote's default branch as
/// `git clone` recorded it, then the usual names.
const TRUNKS: [&str; 4] = [
    "refs/remotes/origin/HEAD",
    "refs/remotes/origin/main",
    "refs/remotes/origin/master",
    "refs/remotes/origin/trunk",
];

/// mori's default starting revision, jj's name for trunk; here it means the first of [`TRUNKS`].
const JJ_TRUNK: &str = "trunk()";

/// Runs the `git` program.
#[derive(Clone, Debug)]
pub struct GitCli {
    program: PathBuf,
    envs: Vec<(OsString, OsString)>,
}

impl GitCli {
    /// Uses the `git` found on `PATH`.
    #[must_use]
    pub fn from_path() -> Self {
        Self::new("git")
    }

    /// Uses `program` as `git`.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            envs: Vec::new(),
        }
    }

    /// Sets an environment variable for every git this runs, e.g. `GIT_CONFIG_GLOBAL` in tests.
    #[must_use]
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }

    /// Runs `git -C dir args` and returns its stdout.
    fn run(&self, dir: &Path, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new(&self.program)
            .envs(self.envs.iter().map(|(key, value)| (key, value)))
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .map_err(|source| self.spawn_error(source))?;
        if !output.status.success() {
            return Err(GitError::Failed {
                dir: dir.to_path_buf(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        String::from_utf8(output.stdout).map_err(|_| GitError::OutputInvalid {
            output: "not UTF-8".to_owned(),
        })
    }

    fn spawn_error(&self, source: std::io::Error) -> GitError {
        match source.kind() {
            ErrorKind::NotFound => GitError::NotFound {
                program: self.program.clone(),
            },
            _ => GitError::Io {
                program: self.program.clone(),
                source,
            },
        }
    }

    /// The first of [`TRUNKS`] the repo at `dir` has.
    fn trunk(&self, dir: &Path) -> Option<&'static str> {
        TRUNKS.into_iter().find(|trunk| {
            self.run(dir, &["rev-parse", "--verify", "--quiet", trunk])
                .is_ok()
        })
    }

    /// The working directory of the tree `name`.
    fn root(&self, clone: &Path, name: &str) -> Result<PathBuf, GitError> {
        self.list(clone)?
            .into_iter()
            .find(|workspace| workspace.name == name)
            .map(|workspace| workspace.root)
            .ok_or_else(|| GitError::NoSuchTree {
                clone: clone.to_path_buf(),
                name: name.to_owned(),
            })
    }

    /// Adds a detached worktree at `path` on `revision`. If git fails, whatever it left is
    /// removed: this call created it, so no one else's work is there.
    fn add_worktree(&self, clone: &Path, path: &Path, revision: &str) -> Result<(), GitError> {
        if path.symlink_metadata().is_ok() {
            return Err(GitError::DestinationExists {
                path: path.to_path_buf(),
            });
        }
        let path_arg = path.to_string_lossy();
        let added = self.run(
            clone,
            &[
                "worktree",
                "add",
                "--detach",
                "--quiet",
                "--end-of-options",
                &path_arg,
                revision,
            ],
        );
        if added.is_err() {
            let _ = std::fs::remove_dir_all(path);
            let _ = self.remove_worktree(clone, path);
        }
        added.map(|_| ())
    }

    /// Removes the worktree at `path` and its directory. With the directory already gone, git
    /// still clears that one entry; a bare `git worktree prune` would clear every stale entry,
    /// including worktrees mori didn't make.
    fn remove_worktree(&self, clone: &Path, path: &Path) -> Result<(), GitError> {
        let path_arg = path.to_string_lossy();
        self.run(
            clone,
            &[
                "worktree",
                "remove",
                "--force",
                "--end-of-options",
                &path_arg,
            ],
        )
        .map(|_| ())
    }
}

impl Workspaces for GitCli {
    type Error = GitError;

    fn list(&self, clone: &Path) -> Result<Vec<Workspace>, GitError> {
        let stdout = self.run(clone, &["worktree", "list", "--porcelain", "-z"])?;
        parse_worktree_list(&stdout)
    }

    fn state(&self, clone: &Path, name: &str) -> Result<TreeState, GitError> {
        self.state_covering(clone, name, &[])
    }
}

impl Vcs for GitCli {
    /// `colocate` doesn't apply: a git clone is always a git repo.
    fn clone_repo(&self, url: &str, path: &Path, _colocate: bool) -> Result<(), GitError> {
        if path.symlink_metadata().is_ok() {
            return Err(GitError::DestinationExists {
                path: path.to_path_buf(),
            });
        }
        let parent = path.parent().unwrap_or(Path::new("."));
        let path_arg = path.to_string_lossy();
        let cloned = self.run(
            parent,
            &["clone", "--quiet", "--end-of-options", url, &path_arg],
        );
        if cloned.is_err() {
            let _ = std::fs::remove_dir_all(path);
        }
        cloned.map(|_| ())
    }

    /// `from` is a git revision; mori's default, `trunk()`, means the remote's default branch.
    fn add_tree(&self, clone: &Path, _name: &str, path: &Path, from: &str) -> Result<(), GitError> {
        let from = if from == JJ_TRUNK {
            self.trunk(clone).unwrap_or(TRUNKS[0])
        } else {
            from
        };
        self.add_worktree(clone, path, from)
    }

    /// git checks out any commit still in the object store, reachable or not.
    fn add_tree_at(
        &self,
        clone: &Path,
        _name: &str,
        path: &Path,
        commit_id: &str,
    ) -> Result<(), GitError> {
        self.add_worktree(clone, path, commit_id)
    }

    /// Nothing to do: git reads the working tree live.
    fn snapshot(&self, _tree: &Path) -> Result<(), GitError> {
        Ok(())
    }

    /// Removes the worktree, which also deletes its directory if it is still there.
    fn forget_tree(&self, clone: &Path, name: &str) -> Result<(), GitError> {
        let root = self.root(clone, name)?;
        self.remove_worktree(clone, &root)
    }

    fn state_covering(
        &self,
        clone: &Path,
        name: &str,
        landed: &[String],
    ) -> Result<TreeState, GitError> {
        let root = self.root(clone, name)?;
        let change = self.run(&root, &["rev-parse", "--short=12", "HEAD"])?;
        let status = self.run(&root, &["status", "--porcelain", "-z"])?;
        let mut args = vec!["rev-list", "--count", "HEAD", "--not", "--remotes"];
        args.extend(landed.iter().map(String::as_str));
        let unpushed = self.run(&root, &args)?;
        Ok(TreeState {
            change: change.trim().to_owned(),
            changed: !status.is_empty(),
            unpushed: unpushed
                .trim()
                .parse()
                .map_err(|_| GitError::OutputInvalid { output: unpushed })?,
        })
    }

    /// `HEAD`: mori only asks for trees with no edits, so it holds all of the tree's work.
    /// When the directory is gone, git still records the worktree's `HEAD`; that is the commit.
    fn working_copy_commit(&self, clone: &Path, name: &str) -> Result<String, GitError> {
        let root = self.root(clone, name)?;
        if root.exists() {
            return Ok(self.run(&root, &["rev-parse", "HEAD"])?.trim().to_owned());
        }
        let stdout = self.run(clone, &["worktree", "list", "--porcelain", "-z"])?;
        match worktree_head(&stdout, &root) {
            Some(head) => Ok(head),
            None => Err(GitError::OutputInvalid { output: stdout }),
        }
    }

    /// Remote branches that hold the tree's own work (its commits not in trunk): those whose tip
    /// is in the tree's history, and those stacked on top of it, as when a lead merges a worker's
    /// commit into a branch and pushes that. A tree with no work of its own has none.
    fn pushed_bookmarks(&self, clone: &Path, name: &str) -> Result<Vec<RemoteBookmark>, GitError> {
        let root = self.root(clone, name)?;
        let trunk = self.trunk(&root);
        let query = |relation: &str| -> Result<Vec<RemoteBookmark>, GitError> {
            let mut args = vec![
                "for-each-ref",
                "--format=%(refname:lstrip=2)%09%(objectname)%09%(symref)",
                relation,
                "HEAD",
            ];
            if let Some(trunk) = trunk {
                args.extend(["--no-merged", trunk]);
            }
            args.push("refs/remotes");
            parse_remote_branches(&self.run(&root, &args)?)
        };
        let mut branches = query("--merged")?;
        // Without trunk there is no telling the tree's own work from everyone's.
        let has_own_work = trunk.is_some_and(|trunk| {
            self.run(&root, &["merge-base", "--is-ancestor", "HEAD", trunk])
                .is_err()
        });
        if has_own_work {
            for branch in query("--contains")? {
                if !branches.contains(&branch) {
                    branches.push(branch);
                }
            }
        }
        Ok(branches)
    }

    /// Every remote-tracking branch, wherever it points; `origin/HEAD` is skipped.
    fn remote_bookmarks(&self, clone: &Path) -> Result<Vec<RemoteBookmark>, GitError> {
        parse_remote_branches(&self.run(
            clone,
            &[
                "for-each-ref",
                "--format=%(refname:lstrip=2)%09%(objectname)%09%(symref)",
                "refs/remotes",
            ],
        )?)
    }

    /// The newer of `HEAD`'s commit time and the newest edited file's modification time.
    fn last_change(&self, clone: &Path, name: &str) -> Result<u64, GitError> {
        let root = self.root(clone, name)?;
        let committed = self.run(&root, &["log", "-1", "--format=%ct", "HEAD"])?;
        let committed: u64 = committed
            .trim()
            .parse()
            .map_err(|_| GitError::OutputInvalid { output: committed })?;
        let status = self.run(&root, &["status", "--porcelain", "-z"])?;
        let edited = edited_paths(&status)
            .filter_map(|path| std::fs::metadata(root.join(path)).ok()?.modified().ok())
            .filter_map(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|since| since.as_secs())
            .max()
            .unwrap_or(0);
        Ok(committed.max(edited))
    }

    fn pin(&self, clone: &Path, name: &str, commit_id: &str) -> Result<(), GitError> {
        self.run(clone, &["update-ref", name, commit_id])
            .map(|_| ())
    }

    fn commit_exists(&self, clone: &Path, commit_id: &str) -> Result<bool, GitError> {
        match self.run(
            clone,
            &["cat-file", "-e", &format!("{commit_id}^{{commit}}")],
        ) {
            Ok(_) => Ok(true),
            Err(GitError::Failed { .. }) => Ok(false),
            Err(other) => Err(other),
        }
    }

    fn fetch(&self, clone: &Path) -> Result<(), GitError> {
        self.run(clone, &["fetch", "--prune", "--quiet"])
            .map(|_| ())
    }

    fn available(&self) -> bool {
        Command::new(&self.program)
            .envs(self.envs.iter().map(|(key, value)| (key, value)))
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    }
}

/// Parses `git worktree list --porcelain -z`: records of NUL-terminated `key value` lines, each
/// record ended by an empty line. The first record is the main worktree.
fn parse_worktree_list(stdout: &str) -> Result<Vec<Workspace>, GitError> {
    let mut workspaces = Vec::new();
    for line in stdout.split('\0') {
        let Some(path) = line.strip_prefix("worktree ") else {
            continue;
        };
        let root = PathBuf::from(path);
        let name = if workspaces.is_empty() {
            BASE_TREE_NAME.to_owned()
        } else {
            root.file_name()
                .ok_or_else(|| GitError::OutputInvalid {
                    output: line.to_owned(),
                })?
                .to_string_lossy()
                .into_owned()
        };
        workspaces.push(Workspace { name, root });
    }
    Ok(workspaces)
}

/// The `HEAD` that `git worktree list --porcelain -z` records for the worktree at `root`.
fn worktree_head(stdout: &str, root: &Path) -> Option<String> {
    let mut lines = stdout.split('\0');
    while let Some(line) = lines.next() {
        if line.strip_prefix("worktree ").map(Path::new) == Some(root) {
            return lines
                .next()
                .and_then(|head| head.strip_prefix("HEAD "))
                .map(str::to_owned);
        }
    }
    None
}

/// Parses `for-each-ref` lines of `<remote>/<branch>`, the commit, and the symref target (set
/// only for `origin/HEAD`, which is skipped).
fn parse_remote_branches(stdout: &str) -> Result<Vec<RemoteBookmark>, GitError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            match fields.as_slice() {
                [_, _, symref] if !symref.is_empty() => None,
                [full, commit_id, _] => Some(full.split_once('/').map_or_else(
                    || {
                        Err(GitError::OutputInvalid {
                            output: line.to_owned(),
                        })
                    },
                    |(remote, name)| {
                        Ok(RemoteBookmark {
                            name: name.to_owned(),
                            remote: remote.to_owned(),
                            commit_id: (*commit_id).to_owned(),
                        })
                    },
                )),
                _ => Some(Err(GitError::OutputInvalid {
                    output: line.to_owned(),
                })),
            }
        })
        .collect()
}

/// The paths in `git status --porcelain -z` output. A rename's entry is followed by its old path,
/// which is skipped.
fn edited_paths(status: &str) -> impl Iterator<Item = &str> {
    let mut entries = status.split('\0').filter(|entry| !entry.is_empty());
    std::iter::from_fn(move || {
        let entry = entries.next()?;
        let (code, path) = (entry.get(..2)?, entry.get(3..)?);
        if code.starts_with('R') || code.starts_with('C') {
            entries.next();
        }
        Some(path)
    })
}

/// Errors from the git backend (domain `git.mori`).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GitError {
    /// `git` isn't installed, or isn't on `PATH`.
    #[error("can't find {}: install git, or put it on PATH", program.display())]
    NotFound {
        /// The program mori tried to run.
        program: PathBuf,
    },

    /// `git` couldn't be started.
    #[error("can't run {}: {source}", program.display())]
    Io {
        /// The program mori tried to run.
        program: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },

    /// Something is already where a clone or tree would go.
    #[error("{} already exists; mori creates only new directories", path.display())]
    DestinationExists {
        /// The path.
        path: PathBuf,
    },

    /// The clone has no worktree of that name.
    #[error("{} has no worktree named {name}", clone.display())]
    NoSuchTree {
        /// The clone.
        clone: PathBuf,
        /// The tree's name.
        name: String,
    },

    /// `git` ran and failed.
    #[error("git failed in {}: {stderr}", dir.display())]
    Failed {
        /// The directory it ran in.
        dir: PathBuf,
        /// What git said.
        stderr: String,
    },

    /// `git` printed something mori can't read.
    #[error("unexpected output from git: {output:?}")]
    OutputInvalid {
        /// The part mori couldn't read.
        output: String,
    },
}

impl GitError {
    /// The AIP-193 `ErrorInfo.domain`.
    pub const DOMAIN: &'static str = "git.mori";
}

impl ErrorDetails for GitError {
    fn code(&self) -> Code {
        match self {
            Self::NotFound { .. } => Code::FailedPrecondition,
            Self::DestinationExists { .. } => Code::AlreadyExists,
            Self::NoSuchTree { .. } => Code::NotFound,
            Self::Io { .. } | Self::Failed { .. } | Self::OutputInvalid { .. } => Code::Internal,
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "GIT_NOT_FOUND",
            Self::DestinationExists { .. } => "PATH_EXISTS",
            Self::NoSuchTree { .. } => "TREE_NOT_FOUND",
            Self::Io { .. } => "IO_ERROR",
            Self::Failed { .. } => "GIT_FAILED",
            Self::OutputInvalid { .. } => "GIT_OUTPUT_INVALID",
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
            Self::Failed { dir, .. } => vec![("dir", dir.display().to_string())],
            Self::DestinationExists { path } => vec![("path", path.display().to_string())],
            Self::NoSuchTree { clone, name } => vec![
                ("clone", clone.display().to_string()),
                ("name", name.clone()),
            ],
            Self::OutputInvalid { .. } => vec![],
        }
    }
}

#[cfg(test)]
mod tests;

//! mori's GitHub host adapter: pull request state, through the `gh` CLI.
//!
//! mori stores no token: `gh` already holds the person's login. Every answer is a fact or
//! unknown; nothing here ever guesses "merged".

use std::path::PathBuf;
use std::process::Command;

use mori_core::clone::RepoId;
use mori_core::vcs::Forge;
pub use mori_core::vcs::Merged;

/// Runs the `gh` program.
#[derive(Clone, Debug)]
pub struct GhCli {
    program: PathBuf,
}

impl GhCli {
    /// Uses the `gh` found on `PATH`.
    #[must_use]
    pub fn from_path() -> Self {
        Self::new("gh")
    }

    /// Uses `program` as `gh`.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    /// Whether the pull request whose head is `bookmark` in `repo` merged.
    #[must_use]
    pub fn pr_merged(&self, repo: &RepoId, bookmark: &str) -> Merged {
        let output = Command::new(&self.program)
            .args(["pr", "view", "--json", "state", "--jq", ".state", "--repo"])
            .arg(repo.to_string())
            .arg("--")
            .arg(bookmark)
            .output();
        let Ok(output) = output else {
            return Merged::Unknown;
        };
        if output.status.success() {
            return match String::from_utf8_lossy(&output.stdout).trim() {
                "MERGED" => Merged::Yes,
                "OPEN" | "CLOSED" => Merged::No,
                _ => Merged::Unknown,
            };
        }
        if String::from_utf8_lossy(&output.stderr).contains("no pull requests found") {
            Merged::No
        } else {
            Merged::Unknown
        }
    }
}

impl Forge for GhCli {
    fn pr_merged(&self, repo: &RepoId, bookmark: &str) -> Merged {
        Self::pr_merged(self, repo, bookmark)
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use mori_core::clone::CloneUrl;

    use super::*;

    /// A stand-in `gh` that prints `stdout` and `stderr` and exits with `status`.
    fn fake_gh(
        dir: &std::path::Path,
        stdout: &str,
        stderr: &str,
        status: i32,
    ) -> std::io::Result<GhCli> {
        let path = dir.join("gh");
        std::fs::write(
            &path,
            format!(
                "#!/bin/sh\nprintf '%s' '{stdout}'\nprintf '%s' '{stderr}' >&2\nexit {status}\n"
            ),
        )?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        Ok(GhCli::new(path))
    }

    fn widget() -> RepoId {
        CloneUrl::parse("github.com/acme/widget").unwrap().repo
    }

    #[test]
    fn gh_answers_become_facts() {
        let dir = tempfile::tempdir().unwrap();
        for (stdout, stderr, status, expected) in [
            ("MERGED\n", "", 0, Merged::Yes),
            ("OPEN\n", "", 0, Merged::No),
            ("CLOSED\n", "", 0, Merged::No),
            (
                "",
                "no pull requests found for branch \"claude/x\"",
                1,
                Merged::No,
            ),
            ("", "error connecting to api.github.com", 1, Merged::Unknown),
            (
                "",
                "To get started with GitHub CLI, please run:  gh auth login",
                4,
                Merged::Unknown,
            ),
            ("SOMETHING NEW\n", "", 0, Merged::Unknown),
        ] {
            let gh = fake_gh(dir.path(), stdout, stderr, status).unwrap();

            assert_eq!(
                gh.pr_merged(&widget(), "claude/x"),
                expected,
                "for {stdout:?} {stderr:?}"
            );
        }
    }

    #[test]
    fn a_missing_gh_is_unknown() {
        let gh = GhCli::new("/nonexistent/gh");

        assert_eq!(gh.pr_merged(&widget(), "claude/x"), Merged::Unknown);
    }
}

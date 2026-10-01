use std::path::Path;

use mori_core::forest::Workspaces;
use mori_core::vcs::Vcs;
use tempfile::TempDir;

use super::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// git with no user or system config, and a fixed identity.
fn git() -> GitCli {
    GitCli::from_path()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
}

/// A bare "remote" with one commit on `main`, and a clone of it made through the adapter.
struct Repo {
    dir: TempDir,
    git: GitCli,
}

impl Repo {
    fn new() -> Result<Self> {
        let dir = TempDir::new()?;
        let git = git();
        let seed = dir.path().join("seed");
        std::fs::create_dir(&seed)?;
        git.run(&seed, &["init", "--quiet", "-b", "main"])?;
        std::fs::write(seed.join("README.md"), "hello\n")?;
        git.run(&seed, &["add", "README.md"])?;
        git.run(&seed, &["commit", "--quiet", "-m", "first"])?;
        git.run(
            dir.path(),
            &["clone", "--quiet", "--bare", "seed", "remote.git"],
        )?;
        let remote = dir.path().join("remote.git");
        git.clone_repo(&remote.to_string_lossy(), &dir.path().join("clone"), true)?;
        Ok(Self { dir, git })
    }

    fn clone(&self) -> PathBuf {
        self.dir.path().join("clone")
    }

    fn tree_path(&self, name: &str) -> PathBuf {
        self.dir.path().join("trees").join(name)
    }

    fn add(&self, name: &str) -> Result<PathBuf> {
        let path = self.tree_path(name);
        std::fs::create_dir_all(self.dir.path().join("trees"))?;
        self.git
            .add_tree(&self.clone(), name, &path, "origin/HEAD")?;
        Ok(path)
    }

    fn commit(&self, tree: &Path, file: &str) -> Result<String> {
        std::fs::write(tree.join(file), "work\n")?;
        self.git.run(tree, &["add", file])?;
        self.git.run(tree, &["commit", "--quiet", "-m", file])?;
        Ok(self
            .git
            .run(tree, &["rev-parse", "HEAD"])?
            .trim()
            .to_owned())
    }
}

#[test]
fn a_new_tree_is_listed_detached_and_clean() -> Result<()> {
    let repo = Repo::new()?;

    let path = repo.add("claude-fix-login")?;

    let names: Vec<String> = repo
        .git
        .list(&repo.clone())?
        .into_iter()
        .map(|workspace| workspace.name)
        .collect();
    assert_eq!(names, ["default", "claude-fix-login"]);
    assert!(path.join(".git").is_file(), "a worktree has a .git file");
    let head = repo.git.run(&path, &["symbolic-ref", "--quiet", "HEAD"]);
    assert!(head.is_err(), "the tree is detached");
    let state = repo.git.state(&repo.clone(), "claude-fix-login")?;
    assert!(!state.changed);
    assert_eq!(state.unpushed, 0);
    Ok(())
}

#[test]
fn edits_and_local_commits_are_unsaved() -> Result<()> {
    let repo = Repo::new()?;
    let path = repo.add("t")?;

    std::fs::write(path.join("new.txt"), "draft\n")?;
    assert!(
        repo.git.state(&repo.clone(), "t")?.changed,
        "untracked counts"
    );

    repo.commit(&path, "new.txt")?;
    let state = repo.git.state(&repo.clone(), "t")?;
    assert!(!state.changed);
    assert_eq!(state.unpushed, 1);
    Ok(())
}

#[test]
fn a_pushed_branch_is_seen_and_a_landed_one_covers_its_work() -> Result<()> {
    let repo = Repo::new()?;
    let path = repo.add("t")?;
    let commit = repo.commit(&path, "fix.txt")?;

    repo.git.run(
        &path,
        &["push", "--quiet", "origin", "HEAD:refs/heads/claude/fix"],
    )?;
    repo.git.fetch(&repo.clone())?;

    let pushed = repo.git.pushed_bookmarks(&repo.clone(), "t")?;
    assert_eq!(
        pushed,
        [RemoteBookmark {
            name: "claude/fix".to_owned(),
            remote: "origin".to_owned(),
            commit_id: commit.clone(),
        }]
    );
    assert_eq!(repo.git.state(&repo.clone(), "t")?.unpushed, 0);

    // The remote deletes the branch after a squash merge.
    repo.git.run(
        &path,
        &["push", "--quiet", "origin", ":refs/heads/claude/fix"],
    )?;
    repo.git.fetch(&repo.clone())?;
    assert!(repo.git.pushed_bookmarks(&repo.clone(), "t")?.is_empty());
    assert_eq!(repo.git.state(&repo.clone(), "t")?.unpushed, 1);
    assert_eq!(
        repo.git
            .state_covering(&repo.clone(), "t", &[commit])?
            .unpushed,
        0
    );
    Ok(())
}

#[test]
fn a_pinned_tree_can_be_removed_and_brought_back() -> Result<()> {
    let repo = Repo::new()?;
    let path = repo.add("t")?;
    repo.commit(&path, "work.txt")?;
    let commit = repo.git.working_copy_commit(&repo.clone(), "t")?;

    repo.git
        .pin(&repo.clone(), "refs/mori/removed/j-1", &commit)?;
    repo.git.forget_tree(&repo.clone(), "t")?;

    assert!(!path.exists(), "removing a worktree deletes its directory");
    assert_eq!(repo.git.list(&repo.clone())?.len(), 1);
    assert!(repo.git.commit_exists(&repo.clone(), &commit)?);

    repo.git.add_tree_at(&repo.clone(), "t", &path, &commit)?;

    assert_eq!(repo.git.working_copy_commit(&repo.clone(), "t")?, commit);
    Ok(())
}

#[test]
fn an_unknown_commit_does_not_exist() -> Result<()> {
    let repo = Repo::new()?;

    assert!(
        !repo
            .git
            .commit_exists(&repo.clone(), "0123456789abcdef0123456789abcdef01234567")?
    );
    Ok(())
}

#[test]
fn a_failed_add_leaves_nothing() -> Result<()> {
    let repo = Repo::new()?;
    let path = repo.tree_path("t");

    let error = repo
        .git
        .add_tree(&repo.clone(), "t", &path, "no-such-revision")
        .err()
        .ok_or("added a tree on a missing revision")?;

    assert_eq!(error.reason(), "GIT_FAILED");
    assert!(!path.exists());
    assert_eq!(repo.git.list(&repo.clone())?.len(), 1);
    Ok(())
}

#[test]
fn an_existing_destination_is_refused_and_left_alone() -> Result<()> {
    let repo = Repo::new()?;
    let path = repo.tree_path("t");
    std::fs::create_dir_all(&path)?;
    std::fs::write(path.join("mine.txt"), "keep\n")?;

    let error = repo
        .git
        .add_tree(&repo.clone(), "t", &path, "origin/HEAD")
        .err()
        .ok_or("added over an existing directory")?;

    assert_eq!(error.reason(), "PATH_EXISTS");
    assert!(path.join("mine.txt").exists());
    Ok(())
}

#[test]
fn a_missing_git_is_a_failed_precondition() {
    let git = GitCli::new("/nonexistent/git");

    let error = git.list(Path::new("/tmp")).unwrap_err();

    assert_eq!(error.code(), Code::FailedPrecondition);
    assert_eq!(error.reason(), "GIT_NOT_FOUND");
}

#[test]
fn worktree_lists_parse_exactly() {
    let stdout = concat!(
        "worktree /home/acme/mori/repos/github.com/acme/widget\0HEAD 1111\0branch refs/heads/main\0\0",
        "worktree /home/acme/mori/trees/widget/claude-fix-login\0HEAD 2222\0detached\0\0",
        "worktree /home/acme/mori/trees/widget/gone\0HEAD 3333\0detached\0prunable gitdir file points to non-existent location\0\0",
    );

    let workspaces = parse_worktree_list(stdout).unwrap();

    let names: Vec<&str> = workspaces.iter().map(|w| w.name.as_str()).collect();
    assert_eq!(names, ["default", "claude-fix-login", "gone"]);
    assert_eq!(
        workspaces[1].root,
        PathBuf::from("/home/acme/mori/trees/widget/claude-fix-login")
    );
}

#[test]
fn remote_branches_skip_origin_head() {
    let stdout = "origin/HEAD\taaaa\trefs/remotes/origin/main\norigin/claude/fix\tbbbb\t\n";

    let branches = parse_remote_branches(stdout).unwrap();

    assert_eq!(
        branches,
        [RemoteBookmark {
            name: "claude/fix".to_owned(),
            remote: "origin".to_owned(),
            commit_id: "bbbb".to_owned(),
        }]
    );
}

#[test]
fn status_paths_skip_rename_sources() {
    let status = " M src/lib.rs\0R  new.rs\0old.rs\0?? notes.txt\0";

    let paths: Vec<&str> = edited_paths(status).collect();

    assert_eq!(paths, ["src/lib.rs", "new.rs", "notes.txt"]);
}

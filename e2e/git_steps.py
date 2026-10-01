"""Steps for CUJ 8, trees as git worktrees (spec/cuj/08-git-backend.feature), and the clone
scenarios that pick a backend."""

import subprocess
from pathlib import Path

import pytest
from harness import Placeholders
from precisely import assert_that, includes
from pytest_bdd import given, parsers, then

CLONE = "<home>/mori/repos/github.com/acme/widget"
TREES = "<home>/mori/trees/widget"


def git(env: dict[str, str], cwd: Path, *args: str) -> str:
    """Runs git with a fixed identity, as a person working in the tree would."""
    result = subprocess.run(
        ["git", "-C", str(cwd), *args],
        env={
            **env,
            "GIT_AUTHOR_NAME": "Test",
            "GIT_AUTHOR_EMAIL": "test@example.com",
            "GIT_COMMITTER_NAME": "Test",
            "GIT_COMMITTER_EMAIL": "test@example.com",
        },
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if result.returncode != 0:
        pytest.fail(f"git {args} exited {result.returncode}\nstderr:\n{result.stderr}")
    return result.stdout


def worktrees(env: dict[str, str], clone: Path) -> list[str]:
    """The names of the clone's worktrees other than the main one."""
    lines = git(env, clone, "worktree", "list", "--porcelain").splitlines()
    roots = [line.removeprefix("worktree ") for line in lines if line.startswith("worktree ")]
    return [Path(root).name for root in roots[1:]]


# Setup


@given(parsers.parse('config.toml sets the default vcs to "{vcs}"'))
def default_vcs(placeholders: Placeholders, vcs: str) -> None:
    config = placeholders.path("$XDG_CONFIG_HOME/mori/config.toml")
    config.write_text(config.read_text() + f'\n[vcs]\ndefault = "{vcs}"\n')


@given(parsers.parse('someone commits work with git in "{name}"'))
def git_commit(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    tree = placeholders.path(f"{TREES}/{name}")
    (tree / "login.rs").write_text("fn login() {}\n")
    git(env, tree, "add", "login.rs")
    git(env, tree, "commit", "--quiet", "--message", "login")


@given(parsers.parse('pushes it with git to the remote as "{branch}"'))
def git_push(env: dict[str, str], placeholders: Placeholders, branch: str) -> None:
    tree = placeholders.path(f"{TREES}/claude-fix-login")
    git(env, tree, "push", "--quiet", "origin", f"HEAD:refs/heads/{branch}")


@given(parsers.parse('the git tree "{lead}" merges "{worker}" and pushes it as "{branch}"'))
def git_lead_merges(
    env: dict[str, str], placeholders: Placeholders, lead: str, worker: str, branch: str
) -> None:
    worker_head = git(env, placeholders.path(f"{TREES}/{worker}"), "rev-parse", "HEAD").strip()
    tree = placeholders.path(f"{TREES}/{lead}")
    git(env, tree, "merge", "--quiet", "--no-edit", worker_head)
    git(env, tree, "push", "--quiet", "origin", f"HEAD:refs/heads/{branch}")


@given(parsers.parse('the remote deletes the git branch "{branch}" after a squash merge'))
def git_remote_deletes(env: dict[str, str], placeholders: Placeholders, branch: str) -> None:
    # The squashed commit lands on trunk elsewhere; what the tree sees is its branch vanishing.
    tree = placeholders.path(CLONE)
    git(env, tree, "push", "--quiet", "origin", f":refs/heads/{branch}")


# The clone and its trees


@then("the clone is a git repo without jj")
def git_only(placeholders: Placeholders) -> None:
    clone = placeholders.path(CLONE)
    assert (clone / ".git").is_dir(), "no .git"
    assert not (clone / ".jj").exists(), ".jj exists"


@then(parsers.parse('the clone has a git worktree "{name}"'))
def has_worktree(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    assert_that(worktrees(env, placeholders.path(CLONE)), includes(name))


@then(parsers.parse('the clone has no git worktree "{name}"'))
def has_no_worktree(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    assert name not in worktrees(env, placeholders.path(CLONE))


@then(parsers.parse('git works in "{name}" on a detached HEAD'))
def detached(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    tree = placeholders.path(f"{TREES}/{name}")
    assert (tree / ".git").is_file(), "a worktree has a .git file"
    git(env, tree, "status")
    head = subprocess.run(
        ["git", "-C", str(tree), "symbolic-ref", "--quiet", "HEAD"],
        env=env,
        capture_output=True,
        check=False,
    )
    assert head.returncode != 0, "HEAD is on a branch"

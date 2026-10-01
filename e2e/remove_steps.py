"""Steps for CUJ 4, removing a tree (spec/cuj/04-tree-remove.feature)."""

import sqlite3
import subprocess

import pytest
from harness import Placeholders
from pytest_bdd import given, parsers, then

TREES = "<home>/mori/trees/widget"
CLONE = "<home>/mori/repos/github.com/acme/widget"


def jj(env: dict[str, str], *args: str) -> None:
    result = subprocess.run(
        ["jj", *args], env=env, capture_output=True, text=True, timeout=60, check=False
    )
    if result.returncode != 0:
        pytest.fail(f"setup: jj {args} exited {result.returncode}\nstderr:\n{result.stderr}")


@given(parsers.parse('someone edits "{name}" without running jj'))
def edit_without_jj(placeholders: Placeholders, name: str) -> None:
    # Different from anything committed in the scenarios, so it is always a real edit.
    (placeholders.path(f"{TREES}/{name}") / "login.rs").write_text("fn login() { todo() }\n")


@given(parsers.parse('someone commits work in "{name}"'))
def commit_work(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    tree = placeholders.path(f"{TREES}/{name}")
    (tree / "login.rs").write_text("fn login() {}\n")
    jj(env, "--repository", str(tree), "commit", "--message", "login")


@given(parsers.parse('pushes it to the remote as "{bookmark}"'))
def push_work(env: dict[str, str], placeholders: Placeholders, bookmark: str) -> None:
    tree = str(placeholders.path(f"{TREES}/claude-fix-login"))
    jj(env, "--repository", tree, "bookmark", "create", bookmark, "--revision", "@-")
    jj(env, "--repository", tree, "git", "push", "--bookmark", bookmark)


@given(parsers.parse('the remote deletes "{bookmark}" after a squash merge'))
def remote_deletes(env: dict[str, str], placeholders: Placeholders, bookmark: str) -> None:
    # The squashed commit lands on trunk elsewhere; what the tree sees is its bookmark vanishing.
    tree = str(placeholders.path(f"{TREES}/claude-fix-login"))
    jj(env, "--repository", tree, "bookmark", "delete", bookmark)
    jj(env, "--repository", tree, "git", "push", "--deleted")


@given(parsers.parse('someone commits more work in "{name}"'))
def commit_more(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    tree = placeholders.path(f"{TREES}/{name}")
    (tree / "logout.rs").write_text("fn logout() {}\n")
    jj(env, "--repository", str(tree), "commit", "--message", "logout")


@given(
    parsers.parse('trunk moves and another workspace rebases the working copy of "{name}" onto it')
)
def rebase_from_elsewhere(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    # As a lead agent rebasing a worker's change does, this leaves the tree's working copy stale.
    # The new trunk commit is pushed, so it is no one's unsaved work.
    clone = placeholders.path(CLONE)
    (clone / "NOTES.md").write_text("trunk moved\n")
    jj(env, "--repository", str(clone), "commit", "--message", "notes")
    jj(env, "--repository", str(clone), "bookmark", "set", "main", "--revision", "@-")
    jj(env, "--repository", str(clone), "git", "push", "--bookmark", "main")
    jj(
        env,
        "--repository",
        str(clone),
        "rebase",
        "--revisions",
        f"{name}@",
        "--destination",
        "main",
    )


@then(parsers.parse('mori has no record of the tree "{name}"'))
def no_record(placeholders: Placeholders, name: str) -> None:
    database = placeholders.path("$XDG_STATE_HOME/mori/mori.db")
    with sqlite3.connect(f"{database.as_uri()}?mode=ro", uri=True) as db:
        (count,) = db.execute("SELECT count(*) FROM trees WHERE name = ?", (name,)).fetchone()
    assert count == 0, f"{name} is still recorded"

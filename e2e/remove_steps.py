"""Steps for CUJ 4, removing a tree (spec/cuj/04-tree-remove.feature)."""

import sqlite3
import subprocess

import pytest
from harness import Placeholders
from pytest_bdd import given, parsers, then

TREES = "<home>/mori/trees/widget"


def jj(env: dict[str, str], *args: str) -> None:
    result = subprocess.run(
        ["jj", *args], env=env, capture_output=True, text=True, timeout=60, check=False
    )
    if result.returncode != 0:
        pytest.fail(f"setup: jj {args} exited {result.returncode}\nstderr:\n{result.stderr}")


@given(parsers.parse('someone edits "{name}" without running jj'))
def edit_without_jj(placeholders: Placeholders, name: str) -> None:
    (placeholders.path(f"{TREES}/{name}") / "login.rs").write_text("fn login() {}\n")


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


@then(parsers.parse('mori has no record of the tree "{name}"'))
def no_record(placeholders: Placeholders, name: str) -> None:
    database = placeholders.path("$XDG_STATE_HOME/mori/mori.db")
    with sqlite3.connect(f"{database.as_uri()}?mode=ro", uri=True) as db:
        (count,) = db.execute("SELECT count(*) FROM trees WHERE name = ?", (name,)).fetchone()
    assert count == 0, f"{name} is still recorded"

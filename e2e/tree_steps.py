"""Steps for CUJ 3, an agent starts a task in its own tree (spec/cuj/03-tree-create.feature)."""

import json
import sqlite3
import subprocess
from pathlib import Path

import pytest
from harness import Mori, Placeholders
from precisely import assert_that, contains_string, equal_to, includes
from pytest_bdd import given, parsers, then

CLONE = "<home>/mori/repos/github.com/acme/widget"


def workspaces(env: dict[str, str], clone: Path) -> list[str]:
    result = subprocess.run(
        [
            "jj",
            "--ignore-working-copy",
            "--repository",
            str(clone),
            "workspace",
            "list",
            "-T",
            'name ++ "\\n"',
        ],
        env=env,
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if result.returncode != 0:
        pytest.fail(f"jj workspace list exited {result.returncode}\nstderr:\n{result.stderr}")
    return result.stdout.split()


# Setup


@given(parsers.parse('USER is "{user}"'))
def user_is(env: dict[str, str], user: str) -> None:
    env["USER"] = user


@given(parsers.parse('someone added a workspace "{name}" to the clone outside mori'))
def foreign_workspace(
    env: dict[str, str], placeholders: Placeholders, tmp_path: Path, name: str
) -> None:
    (tmp_path / "elsewhere").mkdir(exist_ok=True)
    clone = placeholders.path(CLONE)
    result = subprocess.run(
        [
            "jj",
            "--repository",
            str(clone),
            "workspace",
            "add",
            "--name",
            name,
            str(tmp_path / "elsewhere" / name),
        ],
        env=env,
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if result.returncode != 0:
        pytest.fail(f"setup: jj workspace add exited {result.returncode}\n{result.stderr}")


# The clone's workspaces


@then(parsers.parse('the clone has a workspace "{name}"'))
def has_workspace(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    assert_that(workspaces(env, placeholders.path(CLONE)), includes(name))


@then(parsers.parse('the clone has no workspace "{name}"'))
def has_no_workspace(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    assert name not in workspaces(env, placeholders.path(CLONE))


# The records


@then(
    parsers.parse(
        'mori records the tree "{name}" for {owner}\'s task "{task}", lifetime "{lifetime}"'
    )
)
def records_tree(
    placeholders: Placeholders, name: str, owner: str, task: str, lifetime: str
) -> None:
    database = placeholders.path("$XDG_STATE_HOME/mori/mori.db")
    with sqlite3.connect(f"{database.as_uri()}?mode=ro", uri=True) as db:
        row = db.execute(
            "SELECT role, owner, task, lifetime FROM trees WHERE name = ?", (name,)
        ).fetchone()
    assert_that(row, equal_to(("task", owner, task, lifetime)))


# What mori says


@then(parsers.parse('the output says it created "{name}"'))
def says_created(mori: Mori, name: str) -> None:
    assert_that(mori.last.stdout, contains_string(f"Created tree {name}:"))


@then(parsers.parse('it names the tree "{name}", its path and its lifetime "{lifetime}"'))
def json_names_the_tree(mori: Mori, placeholders: Placeholders, name: str, lifetime: str) -> None:
    tree = json.loads(mori.last.stdout)["tree"]
    assert_that(tree["name"], equal_to(name))
    assert_that(tree["path"], equal_to(str(placeholders.path(f"<home>/mori/trees/widget/{name}"))))
    assert_that(tree["lifetime"], equal_to(lifetime))
    assert tree["id"].startswith("tree_"), tree["id"]

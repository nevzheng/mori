"""Steps for CUJ 2, seeing the forest (spec/cuj/02-ls.feature).

Rows are read from `mori ls --json`. proto3 JSON leaves out fields at their defaults (false, 0,
empty), so every read here says what the default is.
"""

import json
import subprocess

import pytest
from harness import Mori, Placeholders
from precisely import assert_that, contains_string, equal_to
from pytest_bdd import given, parsers, then

CLONE = "<home>/mori/repos/github.com/acme/widget"
TREES = "<home>/mori/trees/widget"


def jj(env: dict[str, str], *args: str) -> None:
    result = subprocess.run(
        ["jj", *args], env=env, capture_output=True, text=True, timeout=60, check=False
    )
    if result.returncode != 0:
        pytest.fail(f"setup: jj {args} exited {result.returncode}\nstderr:\n{result.stderr}")


def rows(mori: Mori) -> dict[str, dict]:
    """Every tree row in the forest, by tree name."""
    response = json.loads(mori.last.stdout)
    return {row["tree"]["name"]: row for repo in response.get("repos", []) for row in repo["trees"]}


def row(mori: Mori, name: str) -> dict:
    found = rows(mori)
    assert name in found, f"no row for {name!r} in {sorted(found)}"
    return found[name]


# Setup


@given(parsers.parse('someone edits "{name}" and runs jj there'))
def edit_tree(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    tree = placeholders.path(f"{TREES}/{name}")
    (tree / "login.rs").write_text("fn login() {}\n")
    jj(env, "--repository", str(tree), "status")


@given(parsers.parse('someone forgot the workspace "{name}" outside mori'))
def forget_workspace(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    jj(env, "--repository", str(placeholders.path(CLONE)), "workspace", "forget", name)


# The rows


@then(parsers.parse('the forest shows "{name}" in "{repo}" as a clean base tree, {lifetime}'))
def shows_base_tree(mori: Mori, name: str, repo: str, lifetime: str) -> None:
    found = row(mori, name)
    assert_that(found["status"], equal_to("STATUS_TREE"))
    assert_that(found["tree"]["repo"], equal_to(repo))
    assert_that(found["tree"]["name"], equal_to("default"))
    assert_that(found["tree"]["lifetime"], equal_to(lifetime))
    assert_that(found["state"].get("changed", False), equal_to(False))
    assert_that(found["state"].get("unpushed", 0), equal_to(0))


@then(parsers.parse('the forest shows "{name}" as {owner}\'s task "{task}", lifetime "{lifetime}"'))
def shows_task_tree(mori: Mori, name: str, owner: str, task: str, lifetime: str) -> None:
    tree = row(mori, name)["tree"]
    assert_that(
        (tree["owner"], tree["task"], tree["lifetime"]),
        equal_to((owner, task, lifetime)),
    )


@then(parsers.parse('the forest shows "{name}" as edited with {count:d} unpushed change'))
def shows_work(mori: Mori, name: str, count: int) -> None:
    state = row(mori, name)["state"]
    assert_that(state.get("changed", False), equal_to(True))
    assert_that(state.get("unpushed", 0), equal_to(count))


@then(parsers.parse('the forest shows "{name}" as foreign'))
def shows_foreign(mori: Mori, name: str) -> None:
    assert_that(row(mori, name)["status"], equal_to("STATUS_FOREIGN"))


@then(parsers.parse('the forest shows "{name}" as missing'))
def shows_missing(mori: Mori, name: str) -> None:
    found = row(mori, name)
    assert_that(found["status"], equal_to("STATUS_MISSING"))
    assert "state" not in found, found


@then(parsers.parse('the forest lists "{repo}" as not managed by mori'))
def lists_unmanaged(mori: Mori, repo: str) -> None:
    unmanaged = json.loads(mori.last.stdout).get("unmanagedRepos", [])
    assert_that([entry["repo"] for entry in unmanaged], equal_to([repo]))


@then(parsers.parse('the forest shows only "{repo}"'))
def shows_only(mori: Mori, repo: str) -> None:
    repos = json.loads(mori.last.stdout)["repos"]
    assert_that([entry["repo"] for entry in repos], equal_to([repo]))


# The text view


@then(parsers.parse('the output has a row for "{name}" with "{owner}", "{task}" and "{lifetime}"'))
def text_row(mori: Mori, name: str, owner: str, task: str, lifetime: str) -> None:
    assert_that(mori.last.stdout, contains_string("NAME"))
    lines = [line.split() for line in mori.last.stdout.splitlines()]
    matching = [cells for cells in lines if cells and cells[0] == name]
    assert matching, f"no row for {name!r} in:\n{mori.last.stdout}"
    # The STATUS column is left out when every row is a plain tree.
    assert_that(matching[0][1:4], equal_to([owner, task, lifetime]))


@then(parsers.parse('the forest shows "{name}" pushed "{bookmark}", on the remote, not landed'))
def pushed_not_landed(mori: Mori, name: str, bookmark: str) -> None:
    seen = {entry["name"]: entry for entry in row(mori, name).get("bookmarks", [])}
    assert bookmark in seen, seen
    assert_that(
        (seen[bookmark].get("onRemote", False), seen[bookmark].get("landed", False)),
        equal_to((True, False)),
    )


@then(parsers.parse('the forest shows "{name}" pushed "{bookmark}", gone, landed'))
def pushed_landed(mori: Mori, name: str, bookmark: str) -> None:
    seen = {entry["name"]: entry for entry in row(mori, name).get("bookmarks", [])}
    assert bookmark in seen, seen
    assert_that(
        (seen[bookmark].get("onRemote", False), seen[bookmark].get("landed", False)),
        equal_to((False, True)),
    )

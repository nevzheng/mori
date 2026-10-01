"""Steps for CUJ 6, the cleanup report (spec/cuj/06-gc.feature)."""

import json
import sqlite3
import subprocess

import pytest
from harness import Mori, Placeholders
from precisely import assert_that, contains_string, equal_to
from pytest_bdd import given, parsers, then, when

CLASSES = {
    "remove": "CLASS_REMOVE",
    "blocked": "CLASS_BLOCKED",
    "keep": "CLASS_KEEP",
    "never": "CLASS_NEVER",
}

TREES = "<home>/mori/trees/widget"


def jj(env: dict[str, str], *args: str) -> None:
    result = subprocess.run(
        ["jj", *args], env=env, capture_output=True, text=True, timeout=60, check=False
    )
    if result.returncode != 0:
        pytest.fail(f"setup: jj {args} exited {result.returncode}\nstderr:\n{result.stderr}")


@then(parsers.parse('the report has "{name}" as "{cls}" because "{reason}"'))
def report_has(mori: Mori, name: str, cls: str, reason: str) -> None:
    items = {item["name"]: item for item in json.loads(mori.last.stdout).get("items", [])}
    assert name in items, f"no item {name!r} in {sorted(items)}"
    assert_that((items[name]["class"], items[name]["reason"]), equal_to((CLASSES[cls], reason)))


@given(parsers.parse('someone stacks work in "{upper}" on top of "{lower}"'))
def stack_work(env: dict[str, str], placeholders: Placeholders, upper: str, lower: str) -> None:
    tree = placeholders.path(f"{TREES}/{upper}")
    jj(env, "--repository", str(tree), "new", f"{lower}@-")
    (tree / "logout.rs").write_text("fn logout() {}\n")
    jj(env, "--repository", str(tree), "commit", "--message", "logout")


@given(parsers.parse('pushes the top of "{name}" to the remote as "{bookmark}"'))
def push_top(env: dict[str, str], placeholders: Placeholders, name: str, bookmark: str) -> None:
    tree = str(placeholders.path(f"{TREES}/{name}"))
    jj(env, "--repository", tree, "bookmark", "create", bookmark, "--revision", "@-")
    jj(env, "--repository", tree, "git", "push", "--bookmark", bookmark)


@then("the output says how to apply the report")
def says_how_to_apply(mori: Mori) -> None:
    assert_that(mori.last.stdout, contains_string("`mori gc --apply --yes`"))


# Applying a report


@pytest.fixture
def cleanup() -> dict[str, str]:
    """The journal entry of the scenario's removal, once made."""
    return {}


@then(parsers.parse('the journal has an entry for "{name}" whose commit is pinned'))
def journal_pins(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    journal = placeholders.path("$XDG_STATE_HOME/mori/journal.jsonl").read_text().splitlines()
    entries = [json.loads(line) for line in journal]
    entry = next(entry for entry in entries if entry["name"] == name)
    assert entry["pin"].startswith("refs/mori/removed/"), entry
    clone = placeholders.path("<home>/mori/repos/github.com/acme/widget")
    pinned = subprocess.run(
        ["git", "--git-dir", str(clone / ".git"), "rev-parse", entry["pin"]],
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    assert_that(pinned.stdout.strip(), equal_to(entry["commit_id"]))


def line_for(mori: Mori, name: str) -> str:
    lines = [line for line in mori.last.stdout.splitlines() if f" {name} " in f"{line} "]
    assert lines, f"no line for {name!r} in:\n{mori.last.stdout}"
    return lines[0]


@then(parsers.parse('the output says "{name}" would be removed'))
def says_would_remove(mori: Mori, name: str) -> None:
    assert_that(line_for(mori, name), contains_string("would remove"))


@then(parsers.parse('the output says "{name}" was kept because it has unsaved work'))
def says_kept_unsaved(mori: Mori, name: str) -> None:
    line = line_for(mori, name)
    assert_that(line, contains_string("kept: it has work only this machine has"))
    assert line.strip().startswith("blocked"), line


# Restoring


@given(parsers.parse('I have removed it with "{command}"'))
def removed_with(mori: Mori, cleanup: dict[str, str], command: str) -> None:
    result = mori.run(f"{command} --json")
    if result.returncode != 0:
        pytest.fail(f"setup: {command} exited {result.returncode}\n{result.stderr}")
    removed = [
        item
        for item in json.loads(result.stdout)["items"]
        if item.get("outcome") == "OUTCOME_REMOVED"
    ]
    assert removed, result.stdout
    cleanup["entry"] = removed[0]["entryId"]


@given("I have restored the removed tree")
def restored(mori: Mori, cleanup: dict[str, str]) -> None:
    result = mori.run(f"mori restore {cleanup['entry']}")
    if result.returncode != 0:
        pytest.fail(f"setup: mori restore exited {result.returncode}\n{result.stderr}")


@given("the removed tree's commit is gone from the clone")
def commit_gone(placeholders: Placeholders, cleanup: dict[str, str]) -> None:
    # Point the entry at a commit the clone never had: the same as its commit being collected.
    journal = placeholders.path("$XDG_STATE_HOME/mori/journal.jsonl")
    entries = [json.loads(line) for line in journal.read_text().splitlines()]
    for entry in entries:
        if entry["id"] == cleanup["entry"]:
            entry["commit_id"] = "0123456789abcdef0123456789abcdef01234567"
    journal.write_text("".join(json.dumps(entry) + "\n" for entry in entries))


@when("I restore the removed tree")
def restore(mori: Mori, cleanup: dict[str, str]) -> None:
    mori.run(f"mori restore {cleanup['entry']}")


@then(parsers.parse('mori records the tree "{name}" under its old ID'))
def old_id(placeholders: Placeholders, name: str) -> None:
    journal = placeholders.path("$XDG_STATE_HOME/mori/journal.jsonl").read_text().splitlines()
    old = next(json.loads(line) for line in journal if json.loads(line)["name"] == name)
    database = placeholders.path("$XDG_STATE_HOME/mori/mori.db")
    with sqlite3.connect(f"{database.as_uri()}?mode=ro", uri=True) as db:
        (tree_id,) = db.execute("SELECT id FROM trees WHERE name = ?", (name,)).fetchone()
    assert_that(tree_id, equal_to(old["tree_id"]))

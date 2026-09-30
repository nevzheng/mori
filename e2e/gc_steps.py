"""Steps for CUJ 6, the cleanup report (spec/cuj/06-gc.feature)."""

import json
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


@then(parsers.parse('the report has "{name}" as "{cls}" because "{reason}"'))
def report_has(mori: Mori, name: str, cls: str, reason: str) -> None:
    items = {item["name"]: item for item in json.loads(mori.last.stdout).get("items", [])}
    assert name in items, f"no item {name!r} in {sorted(items)}"
    assert_that((items[name]["class"], items[name]["reason"]), equal_to((CLASSES[cls], reason)))


@then("the report is saved")
def report_saved(mori: Mori, placeholders: Placeholders) -> None:
    report_id = json.loads(mori.last.stdout)["reportId"]
    saved = placeholders.path(f"$XDG_STATE_HOME/mori/reports/{report_id}.json")
    assert_that(json.loads(saved.read_text())["id"], equal_to(report_id))


@then("the output says how to apply the report")
def says_how_to_apply(mori: Mori) -> None:
    assert_that(mori.last.stdout, contains_string("mori gc apply gc-"))


# Applying a report


@pytest.fixture
def cleanup() -> dict[str, str]:
    """The scenario's cleanup report ID, once made."""
    return {}


@given("I have made a cleanup report")
def made_report(mori: Mori, cleanup: dict[str, str]) -> None:
    result = mori.run("mori gc --json")
    if result.returncode != 0:
        pytest.fail(f"setup: mori gc exited {result.returncode}\n{result.stderr}")
    cleanup["id"] = json.loads(result.stdout)["reportId"]


@when("I apply the report without confirming")
def apply_unconfirmed(mori: Mori, cleanup: dict[str, str]) -> None:
    mori.run(f"mori gc apply {cleanup['id']}")


@when(parsers.parse('I apply the report with "{flags}"'))
def apply_report(mori: Mori, cleanup: dict[str, str], flags: str) -> None:
    mori.run(f"mori gc apply {cleanup['id']} {flags}".strip())


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


@then(parsers.parse('the output says "{name}" would be removed'))
def says_would_remove(mori: Mori, name: str) -> None:
    assert_that(mori.last.stdout, contains_string(f"{name}: would remove"))


@then(parsers.parse('the output says "{name}" was kept because it has unsaved work'))
def says_kept_unsaved(mori: Mori, name: str) -> None:
    assert_that(
        mori.last.stdout, contains_string(f"{name}: kept: it has work only this machine has")
    )

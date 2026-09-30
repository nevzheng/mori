"""Steps for CUJ 6, the cleanup report (spec/cuj/06-gc.feature)."""

import json

from harness import Mori, Placeholders
from precisely import assert_that, contains_string, equal_to
from pytest_bdd import parsers, then

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

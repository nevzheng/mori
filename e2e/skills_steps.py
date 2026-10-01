"""Steps for skills in the root (spec/cuj/skills.feature)."""

import hashlib
import json

from harness import DiskWatch, Mori, Placeholders
from precisely import assert_that, contains_string
from pytest_bdd import given, parsers, then

SKILLS = "<home>/mori/context/skills"
MANIFEST = "$XDG_STATE_HOME/mori/skills.json"
OLD = "---\nname: using-mori\ndescription: An older version.\n---\n"
EDIT = "\nMy own note.\n"


def skill(placeholders: Placeholders, name: str):
    return placeholders.path(f"{SKILLS}/{name}/SKILL.md")


# Setup


@given(parsers.parse('an older mori wrote "{name}"'))
def older_mori_wrote(placeholders: Placeholders, disk: DiskWatch, name: str) -> None:
    skill(placeholders, name).write_text(OLD)
    manifest_path = placeholders.path(MANIFEST)
    manifest = json.loads(manifest_path.read_text())
    manifest["files"][f"context/skills/{name}/SKILL.md"] = hashlib.sha256(OLD.encode()).hexdigest()
    manifest_path.write_text(json.dumps(manifest))
    disk.remember()


@given(parsers.parse('someone edited the skill "{name}"'))
def edited(placeholders: Placeholders, name: str) -> None:
    path = skill(placeholders, name)
    path.write_text(path.read_text() + EDIT)


@given(parsers.parse('someone added their own skill "{name}"'))
def own_skill(placeholders: Placeholders, disk: DiskWatch, name: str) -> None:
    path = skill(placeholders, name)
    path.parent.mkdir()
    path.write_text(f"---\nname: {name}\ndescription: My own flow.\n---\n")
    disk.remember()


@given(parsers.parse('the skills were installed by mori "{version}"'))
def installed_by(placeholders: Placeholders, version: str) -> None:
    manifest_path = placeholders.path(MANIFEST)
    manifest = json.loads(manifest_path.read_text())
    manifest["version"] = version
    manifest_path.write_text(json.dumps(manifest))


# The files


@then(parsers.parse('"{path}" lists the skills "{first}" and "{second}"'))
def index_lists(placeholders: Placeholders, path: str, first: str, second: str) -> None:
    index = placeholders.path(path).read_text()
    for name in (first, second):
        assert_that(index, contains_string(f"- [{name}](skills/{name}/SKILL.md): "))


@then(parsers.parse('"{path}" points to "{target}"'))
def points_to(placeholders: Placeholders, path: str, target: str) -> None:
    assert_that(placeholders.path(path).read_text(), contains_string(f"({target})"))


@then(parsers.parse('"{name}" is this mori\'s version'))
def current_version(placeholders: Placeholders, name: str) -> None:
    assert skill(placeholders, name).read_text() != OLD


@then(parsers.parse('the skill "{name}" still has the edit'))
def still_edited(placeholders: Placeholders, name: str) -> None:
    assert skill(placeholders, name).read_text().endswith(EDIT)


# What mori says


@then(parsers.parse('the output says "{name}" was updated'))
def says_updated(mori: Mori, name: str) -> None:
    lines = [line for line in mori.last.stdout.splitlines() if f"skills/{name}/SKILL.md" in line]
    assert lines and lines[0].strip().startswith("updated:"), mori.last.stdout


@then(parsers.parse('the output says "{name}" would be updated'))
def says_would_update(mori: Mori, name: str) -> None:
    lines = [line for line in mori.last.stdout.splitlines() if f"{name}/SKILL.md" in line]
    assert lines and "would update" in lines[0], mori.last.stdout


@then(parsers.parse('the output says "{name}" was kept'))
def says_kept(mori: Mori, name: str) -> None:
    lines = [line for line in mori.last.stdout.splitlines() if f"{name}/SKILL.md" in line]
    assert lines and "kept" in lines[0], mori.last.stdout


@then(parsers.parse('it says the skills are from mori "{version}"'))
def says_stale(mori: Mori, version: str) -> None:
    assert_that(
        mori.last.stderr, contains_string(f"are from mori {version}; run `mori skills sync`")
    )


@given("the skills are where an older mori put them, one of them edited")
def old_layout(placeholders: Placeholders, disk: DiskWatch) -> None:
    manifest_path = placeholders.path(MANIFEST)
    manifest = json.loads(manifest_path.read_text())
    for name, text in [("lead-tree", "old lead\n"), ("using-mori", "old using\n")]:
        old = placeholders.path(f"<home>/mori/skills/{name}/SKILL.md")
        old.parent.mkdir(parents=True)
        old.write_text(text)
        manifest["files"][f"skills/{name}/SKILL.md"] = hashlib.sha256(text.encode()).hexdigest()
    manifest_path.write_text(json.dumps(manifest))
    placeholders.path("<home>/mori/skills/using-mori/SKILL.md").write_text("my edit\n")
    disk.remember()


@then("the edited old skill is still there and the output says it was kept")
def old_edit_kept(mori: Mori, placeholders: Placeholders) -> None:
    old = placeholders.path("<home>/mori/skills/using-mori/SKILL.md")
    assert old.read_text() == "my edit\n"
    lines = [line for line in mori.last.stdout.splitlines() if str(old) in line]
    assert lines and "kept" in lines[0], mori.last.stdout

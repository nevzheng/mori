"""Steps for CUJ 0, setting up mori (spec/cuj/00-init.feature)."""

import json
import sqlite3
import tomllib
from pathlib import Path

from harness import DiskWatch, Mori, Placeholders, listed_paths, mode
from precisely import (
    assert_that,
    contains_exactly,
    contains_string,
    equal_to,
    includes,
)
from pytest_bdd import given, parsers, then

# Everything a first run creates in an empty HOME with the default XDG directories.
FIRST_RUN_CREATES = [
    "<home>/mori",
    "<home>/mori/repos",
    "<home>/mori/trees",
    "$XDG_CONFIG_HOME",
    "$XDG_CONFIG_HOME/mori",
    "$XDG_CONFIG_HOME/mori/config.toml",
    "<home>/.local",
    "$XDG_STATE_HOME",
    "$XDG_STATE_HOME/mori",
    "$XDG_STATE_HOME/mori/mori.db",
    "<home>/mori/skills",
    "<home>/mori/skills/lead-tree",
    "<home>/mori/skills/lead-tree/SKILL.md",
    "<home>/mori/skills/using-mori",
    "<home>/mori/skills/using-mori/SKILL.md",
    "<home>/mori/skills/llms.txt",
    "<home>/mori/llms.txt",
    "$XDG_STATE_HOME/mori/skills.json",
]
FILES = {"config.toml", "mori.db", "SKILL.md", "llms.txt", "skills.json"}


def first_run_creates(placeholders: Placeholders) -> list[Path]:
    return [placeholders.path(path) for path in FIRST_RUN_CREATES]


# Setup


@given(parsers.parse('"{path}" exists'))
def directory_exists(placeholders: Placeholders, disk: DiskWatch, path: str) -> None:
    placeholders.path(path).mkdir(parents=True)
    disk.remember()


# The disk


@then(parsers.parse('"{first}" and "{second}" exist'))
def both_exist(placeholders: Placeholders, first: str, second: str) -> None:
    for path in (first, second):
        assert placeholders.path(path).is_dir(), f"{path} is not a directory"


@then(parsers.parse('"{path}" exists'))
def exists(placeholders: Placeholders, path: str) -> None:
    assert placeholders.path(path).exists(), f"{path} doesn't exist"


@then(parsers.parse('"{path}" exists with mode {bits}'))
def exists_with_mode(placeholders: Placeholders, path: str, bits: str) -> None:
    assert_that(oct(mode(placeholders.path(path))), equal_to(oct(int(bits, 8))))


@then(parsers.parse('nothing exists at "{path}"'))
def nothing_at(placeholders: Placeholders, path: str) -> None:
    assert not placeholders.path(path).exists(), f"{path} exists"


@then("nothing new exists under <home>")
def home_is_empty(home: Path) -> None:
    assert_that(list(home.iterdir()), equal_to([]))


@then("no file or directory was created or modified")
def nothing_changed(home: Path, disk: DiskWatch) -> None:
    assert_that(disk.changed_under(home), equal_to([]))


@then(parsers.parse('nothing under "{path}" changed'))
def nothing_changed_under(placeholders: Placeholders, disk: DiskWatch, path: str) -> None:
    assert_that(disk.changed_under(placeholders.path(path)), equal_to([]))


@then(parsers.parse('"{path}" records the root "{root}"'))
def config_records_root(placeholders: Placeholders, path: str, root: str) -> None:
    config = tomllib.loads(placeholders.path(path).read_text())
    assert_that(config["root"], equal_to(str(placeholders.path(root))))


@then(parsers.parse('the database exists with mode {bits} and records the root "{root}"'))
def database_records_root(placeholders: Placeholders, bits: str, root: str) -> None:
    database = placeholders.path("$XDG_STATE_HOME/mori/mori.db")
    assert_that(oct(mode(database)), equal_to(oct(int(bits, 8))))
    with sqlite3.connect(f"{database.as_uri()}?mode=ro", uri=True) as connection:
        (recorded,) = connection.execute("SELECT value FROM meta WHERE key = 'root'").fetchone()
    assert_that(recorded, equal_to(str(placeholders.path(root))))


# What mori says


@then("the output lists every path it created")
def lists_created(mori: Mori, home: Path, placeholders: Placeholders) -> None:
    listed = listed_paths(mori.last.stdout, "created")
    on_disk = set(home.rglob("*"))
    assert_that(listed, equal_to(on_disk))
    assert_that(listed, equal_to(set(first_run_creates(placeholders))))


@then("the output lists the paths it would create")
def lists_would_create(mori: Mori, placeholders: Placeholders) -> None:
    listed = listed_paths(mori.last.stdout, "would create")
    assert_that(listed, equal_to(set(first_run_creates(placeholders))))


@then("the output says mori is already set up")
def says_already_set_up(mori: Mori) -> None:
    assert_that(mori.last.stdout, contains_string("mori is already set up at "))


@then("the output lists no unmanaged repos")
def lists_no_unmanaged(mori: Mori) -> None:
    assert "Clones mori didn't make" not in mori.last.stdout, mori.last.stdout


@then(parsers.parse('the output lists "{repo}" as unmanaged'))
def lists_unmanaged(mori: Mori, repo: str) -> None:
    _, found, unmanaged = mori.last.stdout.partition("Clones mori didn't make (left alone):\n")
    assert found, f"no unmanaged section in:\n{mori.last.stdout}"
    assert_that([line.split()[0] for line in unmanaged.splitlines()], includes(repo))


@then("the message says moving a root needs a migration that isn't defined yet")
def says_migration_needed(mori: Mori) -> None:
    assert_that(mori.last.stderr, contains_string("needs a migration, which isn't defined yet"))


# JSON


@then("stdout is exactly one JSON object")
def one_json_object(mori: Mori) -> None:
    lines = mori.last.stdout.splitlines()
    assert_that(len(lines), equal_to(1))
    assert isinstance(json.loads(lines[0]), dict), f"not a JSON object: {lines[0]}"


@then("it lists the root, the created paths and the unmanaged repos")
def json_lists_everything(mori: Mori, placeholders: Placeholders) -> None:
    response = json.loads(mori.last.stdout)
    assert_that(response["root"], equal_to(str(placeholders.path("<home>/mori"))))
    expected = [
        {"path": str(path), "kind": "KIND_FILE" if path.name in FILES else "KIND_DIRECTORY"}
        for path in first_run_creates(placeholders)
    ]
    assert_that(response["created"], contains_exactly(*expected))
    # proto3 JSON leaves out empty lists: there are no clones yet.
    assert_that(response.get("unmanagedRepos", []), equal_to([]))

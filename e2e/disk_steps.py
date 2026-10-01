"""Steps for CUJ 10, seeing what the forest costs (spec/cuj/10-disk-usage.feature)."""

import json
import subprocess

import pytest
from harness import Mori, Placeholders
from precisely import (
    assert_that,
    contains_string,
    equal_to,
    greater_than_or_equal_to,
    less_than,
)
from pytest_bdd import given, parsers, then

TREES = "<home>/mori/trees/widget"
MB = 1024 * 1024


def rows(mori: Mori) -> dict[str, dict]:
    response = json.loads(mori.last.stdout)
    return {row["tree"]["name"]: row for repo in response.get("repos", []) for row in repo["trees"]}


# Setup


@given(parsers.parse('"{name}" holds a {size:d} MB file'))
def big_file(placeholders: Placeholders, name: str, size: int) -> None:
    # Random-ish bytes so no filesystem can compress or dedupe them away.
    block = bytes(range(256)) * 4096
    with (placeholders.path(f"{TREES}/{name}") / "big.bin").open("wb") as file:
        for _ in range(size):
            file.write(block)


def jj(env: dict[str, str], tree: str, *args: str) -> None:
    result = subprocess.run(
        ["jj", "--repository", tree, *args],
        env=env,
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if result.returncode != 0:
        pytest.fail(f"setup: jj {args} exited {result.returncode}\nstderr:\n{result.stderr}")


@given(parsers.parse('"{name}" landed a {size:d} MB file'))
def landed_big_file(
    env: dict[str, str], mori: Mori, placeholders: Placeholders, name: str, size: int
) -> None:
    # Commit a big file, push it, let mori see the bookmark, then delete it as a squash merge does.
    big_file(placeholders, name, size)
    tree = str(placeholders.path(f"{TREES}/{name}"))
    bookmark = f"claude/{name}"
    jj(env, tree, "commit", "--message", "big")
    jj(env, tree, "bookmark", "create", bookmark, "--revision", "@-")
    jj(env, tree, "git", "push", "--bookmark", bookmark)
    if mori.run("mori ls").returncode != 0:
        pytest.fail(f"setup: mori ls failed\n{mori.last.stderr}")
    jj(env, tree, "bookmark", "delete", bookmark)
    jj(env, tree, "git", "push", "--deleted")


@given(parsers.parse('config.toml sets the free-space floor to "{floor}"'))
def free_space_floor(placeholders: Placeholders, floor: str) -> None:
    config = placeholders.path("$XDG_CONFIG_HOME/mori/config.toml")
    config.write_text(config.read_text() + f'\n[disk]\nwarn_below = "{floor}"\n')


# Sizes


@then(parsers.parse('the forest shows "{name}" using at least {size:d} MB'))
def using_at_least(mori: Mori, name: str, size: int) -> None:
    assert_that(int(rows(mori)[name].get("sizeBytes", "0")), greater_than_or_equal_to(size * MB))


@then(parsers.parse('the forest shows "{name}" using less than {size:d} MB'))
def using_less_than(mori: Mori, name: str, size: int) -> None:
    found = rows(mori)[name]
    assert found.get("sizeMeasuredAt"), found
    assert_that(int(found.get("sizeBytes", "0")), less_than(size * MB))


@then("the forest shows no sizes")
def no_sizes(mori: Mori) -> None:
    for name, found in rows(mori).items():
        assert "sizeMeasuredAt" not in found, (name, found)
        assert "sizeBytes" not in found, (name, found)


@then("the forest shows the disk's free and total space")
def shows_disk(mori: Mori) -> None:
    disk = json.loads(mori.last.stdout)["disk"]
    total = int(disk["totalBytes"])
    assert 0 < int(disk.get("freeBytes", "0")) <= total, disk


# gc


def gc_items(mori: Mori) -> dict[str, dict]:
    return {item["name"]: item for item in json.loads(mori.last.stdout).get("items", [])}


@then(parsers.parse('gc shows "{name}" freeing at least {size:d} MB'))
def gc_frees(mori: Mori, name: str, size: int) -> None:
    item = gc_items(mori)[name]
    assert_that(item.get("class"), equal_to("CLASS_REMOVE"))
    assert_that(int(item.get("sizeBytes", "0")), greater_than_or_equal_to(size * MB))


@then(parsers.parse("gc would remove {count:d} tree"))
def gc_would_remove(mori: Mori, count: int) -> None:
    picked = [
        name
        for name, item in gc_items(mori).items()
        if item.get("outcome") == "OUTCOME_WOULD_REMOVE"
    ]
    assert_that(len(picked), equal_to(count))


@then(parsers.parse('only one of "{first}" and "{second}" is left'))
def one_left(placeholders: Placeholders, first: str, second: str) -> None:
    left = [name for name in (first, second) if placeholders.path(f"{TREES}/{name}").exists()]
    assert_that(len(left), equal_to(1))


# Text


@then("the output has a SIZE column")
def size_column(mori: Mori) -> None:
    header = [line.split() for line in mori.last.stdout.splitlines() if "NAME" in line]
    assert header, mori.last.stdout
    assert_that(header[0][-1], contains_string("SIZE"))


@then("the output says how much the trees use and how much is free")
def says_summary(mori: Mori) -> None:
    assert_that(mori.last.stdout, contains_string("Trees use "))
    assert_that(mori.last.stdout, contains_string(" free of "))


@then(parsers.parse('the output warns that free space is below the "{floor}" floor'))
def warns_low(mori: Mori, floor: str) -> None:
    assert_that(mori.last.stdout, contains_string(f"below the {floor} floor"))
    assert_that(mori.last.stdout, contains_string("`mori gc`"))

"""Steps for CUJ 10, seeing what the forest costs (spec/cuj/10-disk-usage.feature)."""

import hashlib
import json
import subprocess

import pytest
from harness import Mori, Placeholders
from precisely import (
    all_of,
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


@given(parsers.parse('"{path}" says "{text}"'))
def file_says(placeholders: Placeholders, path: str, text: str) -> None:
    file = placeholders.path(path)
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_text(f"{text}\n")


@given(parsers.parse('config.toml sets the free-space floor to "{floor}"'))
def free_space_floor(placeholders: Placeholders, floor: str) -> None:
    config = placeholders.path("$XDG_CONFIG_HOME/mori/config.toml")
    config.write_text(config.read_text() + f'\n[disk]\nwarn_below = "{floor}"\n')


@given(parsers.parse('config.toml points the Bazel output user root at "{root}"'))
def bazel_root(placeholders: Placeholders, root: str) -> None:
    config = placeholders.path("$XDG_CONFIG_HOME/mori/config.toml")
    path = placeholders.path(root)
    config.write_text(config.read_text() + f'\n[disk]\nbazel_output_user_root = "{path}"\n')


def output_base(placeholders: Placeholders, workspace: str):
    root = placeholders.path("<home>/bazel-root")
    path = str(placeholders.path(workspace))
    return root / hashlib.md5(path.encode()).hexdigest()


@given(parsers.parse('Bazel has an output base for "{workspace}"'))
def bazel_output_base(placeholders: Placeholders, workspace: str) -> None:
    base = output_base(placeholders, workspace)
    (base / "execroot" / "_main").mkdir(parents=True)
    (base / "execroot" / "_main" / "out.o").write_bytes(b"\0" * 65536)
    (base / "DO_NOT_BUILD_HERE").write_text(f"{placeholders.path(workspace)}\n")


@given(parsers.parse('config.toml sets the disk limit "{key}" to "{value}"'))
def disk_limit(placeholders: Placeholders, key: str, value: str) -> None:
    config = placeholders.path("$XDG_CONFIG_HOME/mori/config.toml")
    config.write_text(config.read_text() + f"\n[disk]\n{key} = {value}\n")


@given(parsers.parse('config.toml sets the disk limit "{key}" to the size "{size}"'))
def disk_size_limit(placeholders: Placeholders, key: str, size: str) -> None:
    disk_limit(placeholders, key, f'"{size}"')


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


def leftovers(mori: Mori) -> dict[str, dict]:
    return {
        item["name"]: item
        for item in json.loads(mori.last.stdout).get("items", [])
        if item.get("kind") == "KIND_BAZEL_LEFTOVER"
    }


@then(parsers.parse('gc lists the Bazel output of "{name}" as safe to remove'))
def lists_leftover(mori: Mori, name: str) -> None:
    found = leftovers(mori)
    assert name in found, found
    assert_that(found[name]["class"], equal_to("CLASS_REMOVE"))
    assert_that(found[name]["reason"], equal_to("ORPHANED"))
    assert_that(int(found[name].get("sizeBytes", "0")), greater_than_or_equal_to(65536))


@then(parsers.parse('gc lists no Bazel output of "{name}"'))
def lists_no_leftover(mori: Mori, name: str) -> None:
    assert name not in leftovers(mori), leftovers(mori)


@then(parsers.parse('the Bazel output base for "{workspace}" is gone'))
def base_gone(placeholders: Placeholders, workspace: str) -> None:
    assert not output_base(placeholders, workspace).exists()


@then(parsers.parse('the Bazel output base for "{workspace}" is still there'))
def base_kept(placeholders: Placeholders, workspace: str) -> None:
    assert (output_base(placeholders, workspace) / "DO_NOT_BUILD_HERE").exists()


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


@then(parsers.parse('the output warns that "{what}" is over "{key}"'))
def warns_over(mori: Mori, what: str, key: str) -> None:
    lines = [line for line in mori.last.stdout.splitlines() if line.startswith("warning: ")]
    assert any(what in line and f"over [disk] {key}" in line for line in lines), mori.last.stdout


@then(parsers.parse("the output warns that no {tool} cache is shared across trees"))
def warns_no_cache(mori: Mori, tool: str) -> None:
    assert_that(
        mori.last.stdout,
        all_of(
            contains_string("warning: agents make many trees"),
            contains_string(f"no {tool}"),
            contains_string("https://nevzheng.github.io/mori/shared-caches/"),
        ),
    )


@then("the output has no cache warning")
def no_cache_warning(mori: Mori) -> None:
    assert "agents make many trees" not in mori.last.stdout, mori.last.stdout


@then(parsers.parse('the output warns that free space is below the "{floor}" floor'))
def warns_low(mori: Mori, floor: str) -> None:
    assert_that(mori.last.stdout, contains_string(f"below the {floor} floor"))
    assert_that(mori.last.stdout, contains_string("`mori gc`"))

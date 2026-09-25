"""Shared steps for mori's end-to-end features.

Every scenario runs the real `mori` binary (built by Bazel; its path is in MORI_BIN) with a
temporary HOME and XDG directories, so nothing touches the machine running the tests.
Scenarios tagged @wip describe features that aren't built yet and are skipped.
"""

import os
import shlex
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

import pytest
from matchers import matches_regex
from precisely import assert_that, equal_to
from pytest_bdd import given, parsers, then, when

MORI_BIN = Path(os.environ["MORI_BIN"]).resolve()


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line("markers", "wip: the feature isn't built yet; the scenario is skipped")


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    for item in items:
        if item.get_closest_marker("wip"):
            item.add_marker(pytest.mark.skip(reason="@wip: the feature isn't built yet"))


@dataclass
class World:
    """What one scenario has set up and observed."""

    home: Path
    env: dict[str, str] = field(default_factory=dict)
    result: subprocess.CompletedProcess[str] | None = None

    def last(self) -> subprocess.CompletedProcess[str]:
        assert self.result is not None, "no command has been run yet"
        return self.result


@pytest.fixture
def world(tmp_path: Path) -> World:
    home = tmp_path / "home"
    home.mkdir()
    return World(home=home)


@given("a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it")
def temporary_home(world: World) -> None:
    world.env = {
        "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
        "HOME": str(world.home),
        "XDG_CONFIG_HOME": str(world.home / ".config"),
        "XDG_STATE_HOME": str(world.home / ".local" / "state"),
        "XDG_CACHE_HOME": str(world.home / ".cache"),
    }


@given("MORI_ROOT is not set")
def mori_root_unset(world: World) -> None:
    world.env.pop("MORI_ROOT", None)


@when(parsers.parse('I run "{command}"'))
def run(world: World, command: str) -> None:
    program, *args = shlex.split(command)
    assert program == "mori", f"steps only run mori, not {program!r}"
    world.result = subprocess.run(
        [str(MORI_BIN), *args],
        env=world.env,
        cwd=world.home,
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )


@then("it succeeds")
def succeeds(world: World) -> None:
    result = world.last()
    if result.returncode != 0:
        pytest.fail(f"mori exited {result.returncode}\nstderr:\n{result.stderr}")


@then(parsers.parse("it fails with exit code {code:d}"))
def fails_with_exit_code(world: World, code: int) -> None:
    assert_that(world.last().returncode, equal_to(code))


@then(parsers.parse('stdout matches "{pattern}"'))
def stdout_matches(world: World, pattern: str) -> None:
    assert_that(world.last().stdout.strip(), matches_regex(pattern))

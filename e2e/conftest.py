"""Fixtures and the steps every feature shares.

Every scenario runs the real `mori` binary (built by Bazel; its path is in MORI_BIN) with a
temporary HOME, so nothing touches the machine running the tests. Steps get what they need from
the fixtures below. Scenarios tagged @wip describe features that aren't built yet and are skipped.
"""

import os
from pathlib import Path

import pytest
from harness import CANONICAL_CODES, DiskWatch, Mori, Placeholders
from matchers import matches_regex
from precisely import all_of, assert_that, contains_string, equal_to
from pytest_bdd import given, parsers, then, when

# Steps for one journey each.
pytest_plugins = ["init_steps"]


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line("markers", "wip: the feature isn't built yet; the scenario is skipped")


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    for item in items:
        if item.get_closest_marker("wip"):
            item.add_marker(pytest.mark.skip(reason="@wip: the feature isn't built yet"))


# Fixtures


@pytest.fixture(scope="session")
def mori_binary() -> Path:
    return Path(os.environ["MORI_BIN"]).resolve()


@pytest.fixture
def home(tmp_path: Path) -> Path:
    home = tmp_path / "home"
    home.mkdir()
    return home


@pytest.fixture
def env() -> dict[str, str]:
    """The scenario's environment. Starts with only PATH; steps add the rest."""
    return {"PATH": os.environ.get("PATH", "/usr/bin:/bin")}


@pytest.fixture
def mori(mori_binary: Path, home: Path, env: dict[str, str]) -> Mori:
    return Mori(binary=mori_binary, home=home, env=env)


@pytest.fixture
def placeholders(home: Path, env: dict[str, str]) -> Placeholders:
    return Placeholders(home=home, env=env)


@pytest.fixture
def disk(home: Path) -> DiskWatch:
    return DiskWatch(root=home)


# The environment


@given("a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it")
def temporary_home(home: Path, env: dict[str, str]) -> None:
    env["HOME"] = str(home)
    env["XDG_CONFIG_HOME"] = str(home / ".config")
    env["XDG_STATE_HOME"] = str(home / ".local" / "state")
    env["XDG_CACHE_HOME"] = str(home / ".cache")


@given("MORI_ROOT is not set")
def mori_root_unset(env: dict[str, str]) -> None:
    env.pop("MORI_ROOT", None)


@given(parsers.parse('MORI_ROOT is "{root}"'))
def mori_root_is(env: dict[str, str], placeholders: Placeholders, root: str) -> None:
    env["MORI_ROOT"] = str(placeholders.path(root))


# Running mori


@given(parsers.parse('I have run "{command}"'))
def have_run(mori: Mori, disk: DiskWatch, command: str) -> None:
    result = mori.run(command)
    if result.returncode != 0:
        pytest.fail(f"setup: {command!r} exited {result.returncode}\nstderr:\n{result.stderr}")
    disk.remember()


@when(parsers.parse('I run "{command}"'))
def run(mori: Mori, command: str) -> None:
    mori.run(command)


# Outcomes


@then("it succeeds")
def succeeds(mori: Mori) -> None:
    result = mori.last
    if result.returncode != 0:
        pytest.fail(f"mori exited {result.returncode}\nstderr:\n{result.stderr}")


@then(parsers.parse("it fails with exit code {code:d}"))
def fails_with_exit_code(mori: Mori, code: int) -> None:
    assert_that(mori.last.returncode, equal_to(code))


@then(parsers.parse('it fails with status {status} and reason "{reason}"'))
def fails_with_status(mori: Mori, status: str, reason: str) -> None:
    assert_that(mori.last.returncode, equal_to(CANONICAL_CODES[status]))
    assert_that(
        mori.last.stderr,
        all_of(contains_string(f"status: {status}"), contains_string(f"reason: {reason} (")),
    )


@then(parsers.parse('stdout matches "{pattern}"'))
def stdout_matches(mori: Mori, pattern: str) -> None:
    assert_that(mori.last.stdout.strip(), matches_regex(pattern))

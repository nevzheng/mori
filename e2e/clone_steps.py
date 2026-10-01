"""Steps for CUJ 1, adding a repo (spec/cuj/01-clone.feature).

The remote is local: repos made with jj in a temp directory, reached through git's standard
`url.<base>.insteadOf` rewrite, so `mori clone github.com/acme/widget` clones for real without
the network and without mori knowing it's a test.
"""

import json
import sqlite3
import subprocess
from pathlib import Path

import pytest
from harness import Mori, Placeholders
from precisely import assert_that, contains_string, equal_to
from pytest_bdd import given, parsers, then

CLONE = "<home>/mori/repos/github.com/acme/widget"


@pytest.fixture
def remotes(tmp_path: Path, env: dict[str, str]) -> Path:
    """Where the fake remote's repos live, and the git config that sends GitHub URLs there."""
    remotes = tmp_path / "remotes"
    remotes.mkdir()
    gitconfig = tmp_path / "gitconfig"
    gitconfig.write_text(
        f'[url "{remotes}/github.com/"]\n'
        "\tinsteadOf = https://github.com/\n"
        "\tinsteadOf = git@github.com:\n"
    )
    env["GIT_CONFIG_GLOBAL"] = str(gitconfig)
    env["GIT_CONFIG_NOSYSTEM"] = "1"
    return remotes


def jj(env: dict[str, str], *args: str) -> None:
    result = subprocess.run(
        ["jj", *args], env=env, capture_output=True, text=True, timeout=60, check=False
    )
    if result.returncode != 0:
        pytest.fail(f"setup: jj {args} exited {result.returncode}\nstderr:\n{result.stderr}")


def database(placeholders: Placeholders) -> sqlite3.Connection:
    path = placeholders.path("$XDG_STATE_HOME/mori/mori.db")
    return sqlite3.connect(f"{path.as_uri()}?mode=ro", uri=True)


# Setup


@given(parsers.parse('"{repo}" is a repo on the remote'))
def repo_on_remote(remotes: Path, env: dict[str, str], repo: str) -> None:
    # The short form fetches https://<repo>.git, so that is the directory's name.
    source = remotes / f"{repo}.git"
    source.parent.mkdir(parents=True, exist_ok=True)
    jj(env, "git", "init", "--colocate", str(source))
    (source / "README.md").write_text("widget\n")
    jj(env, "--repository", str(source), "commit", "--message", "first")
    jj(env, "--repository", str(source), "bookmark", "create", "main", "--revision", "@-")


@given("mori's database is read-only")
def database_read_only(placeholders: Placeholders) -> None:
    placeholders.path("$XDG_STATE_HOME/mori/mori.db").chmod(0o400)


@given("mori's database is gone")
def database_gone(placeholders: Placeholders) -> None:
    placeholders.path("$XDG_STATE_HOME/mori/mori.db").unlink()


# The clone


@then("the clone is a jj repo colocated with git")
def colocated(placeholders: Placeholders) -> None:
    clone = placeholders.path(CLONE)
    assert (clone / ".jj").is_dir(), "no .jj"
    assert (clone / ".git").exists(), "no .git"


@then("the clone is a jj repo without git")
def jj_only(placeholders: Placeholders) -> None:
    clone = placeholders.path(CLONE)
    assert (clone / ".jj").is_dir(), "no .jj"
    assert not (clone / ".git").exists(), ".git exists"


# The records


@then(parsers.parse('mori records "{repo}" with a pinned base tree'))
def records_repo(placeholders: Placeholders, repo: str) -> None:
    with database(placeholders) as db:
        rows = db.execute(
            "SELECT repos.remote, trees.name, trees.lifetime"
            " FROM repos JOIN trees ON trees.repo_id = repos.id"
        ).fetchall()
    assert_that(rows, equal_to([(repo, "default", "pinned")]))


@then(parsers.parse('the base tree of "{repo}" belongs to "{owner}"'))
def base_owner(placeholders: Placeholders, repo: str, owner: str) -> None:
    with database(placeholders) as db:
        (recorded,) = db.execute(
            "SELECT trees.owner FROM trees JOIN repos ON repos.id = trees.repo_id"
            " WHERE repos.remote = ? AND trees.name = 'default'",
            (repo,),
        ).fetchone()
    assert_that(recorded, equal_to(owner))


@then("mori records no repos")
def records_nothing(placeholders: Placeholders) -> None:
    with database(placeholders) as db:
        (count,) = db.execute("SELECT count(*) FROM repos").fetchone()
    assert_that(count, equal_to(0))


# What mori says


@then(parsers.parse('the output says it cloned "{repo}"'))
def says_cloned(mori: Mori, repo: str) -> None:
    assert_that(mori.last.stdout, contains_string(f"Cloned {repo} into "))


@then("the message says the clone is kept")
def says_clone_kept(mori: Mori) -> None:
    assert_that(mori.last.stderr, contains_string("The clone is kept"))


@then(parsers.parse('it names the repo "{repo}", its path and that it is colocated'))
def json_names_the_clone(mori: Mori, placeholders: Placeholders, repo: str) -> None:
    response = json.loads(mori.last.stdout)
    assert_that(response["repo"], equal_to(repo))
    assert_that(response["path"], equal_to(str(placeholders.path(CLONE))))
    assert_that(response["colocated"], equal_to(True))
    assert_that(response["fetchUrl"], equal_to(f"https://{repo}.git"))


@then(parsers.parse('"{path}" lists the repo context "{folder}" for "{repo}"'))
def index_lists_repo(placeholders: Placeholders, path: str, folder: str, repo: str) -> None:
    index = placeholders.path(path).read_text()
    assert_that(index, contains_string(f"- [{folder}](projects/{folder}/): {repo}"))

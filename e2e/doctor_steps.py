"""Steps for CUJ 9, checking the root (spec/cuj/09-doctor.feature)."""

import json
import shutil
import subprocess

import pytest
from harness import Mori, Placeholders
from pytest_bdd import given, parsers, then

CLONE = "<home>/mori/repos/github.com/acme/widget"
TREES = "<home>/mori/trees/widget"


def run(env: dict[str, str], *args: str) -> None:
    result = subprocess.run(
        list(args), env=env, capture_output=True, text=True, timeout=60, check=False
    )
    if result.returncode != 0:
        pytest.fail(f"setup: {args} exited {result.returncode}\nstderr:\n{result.stderr}")


@given(parsers.parse('the directory of "{name}" was deleted by hand'))
def tree_dir_deleted(placeholders: Placeholders, name: str) -> None:
    shutil.rmtree(placeholders.path(f"{TREES}/{name}"))


@given(parsers.parse('the directory "{path}" was deleted by hand'))
def dir_deleted(placeholders: Placeholders, path: str) -> None:
    shutil.rmtree(placeholders.path(path))


@given(parsers.parse('the bookmark "{name}" in the clone has two targets'))
def conflicted_bookmark(env: dict[str, str], placeholders: Placeholders, name: str) -> None:
    # Two operations move the bookmark to sibling commits from the same starting point: jj keeps
    # both targets.
    jj = ["jj", "--repository", str(placeholders.path(CLONE))]
    run(env, *jj, "new", f"{name}@origin", "--message", "left")
    run(env, *jj, "new", f"{name}@origin", "--message", "right")
    op = subprocess.run(
        [*jj, "op", "log", "--no-graph", "--limit", "1", "-T", 'id.short() ++ "\\n"'],
        env=env,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    run(env, *jj, "bookmark", "set", name, "-r", 'description(exact:"left\\n")')
    run(env, *jj, "--at-op", op, "bookmark", "set", name, "-r", 'description(exact:"right\\n")')


@given(parsers.parse('someone runs "jj git init" in the clone'))
def jj_git_init(env: dict[str, str], placeholders: Placeholders) -> None:
    run(env, "jj", "git", "init", "--colocate", str(placeholders.path(CLONE)))


@then(
    parsers.re(
        r'doctor reports "(?P<code>[A-Z_]+)" for "(?P<subject>[^"]+)", (?P<fixable>fixable|not fixable)'
    )
)
def reports(mori: Mori, placeholders: Placeholders, code: str, subject: str, fixable: str) -> None:
    if subject.startswith("<home>"):
        subject = str(placeholders.path(subject))
    findings = json.loads(mori.last.stdout).get("findings", [])
    matching = [f for f in findings if f["code"] == code and f["subject"] == subject]
    assert matching, f"no {code} for {subject} in {findings}"
    assert matching[0].get("autoFixable", False) == (fixable == "fixable"), matching[0]
    assert matching[0]["fix"], matching[0]

"""Steps for CUJ 11, agents asking mori over MCP (spec/cuj/11-mcp.feature)."""

import json
import subprocess
from pathlib import Path

import pytest
from harness import Placeholders
from precisely import assert_that, equal_to
from pytest_bdd import parsers, then, when


@pytest.fixture
def mcp() -> dict[str, dict]:
    """The answers of the scenario's MCP calls, by tool name."""
    return {}


def exchange(binary: Path, home: Path, env: dict[str, str], requests: list[dict]) -> list[dict]:
    """Runs `mori mcp` with `requests` on stdin and returns its replies, in order."""
    lines = [
        {"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        *requests,
    ]
    result = subprocess.run(
        [str(binary), "mcp"],
        input="".join(json.dumps(line) + "\n" for line in lines),
        env=env,
        cwd=home,
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    return replies[1:]


@when("an agent calls over MCP:")
def calls(
    mori_binary: Path,
    home: Path,
    env: dict[str, str],
    placeholders: Placeholders,
    mcp: dict[str, dict],
    datatable: list[list[str]],
) -> None:
    rows = datatable[1:]
    requests = [
        {
            "jsonrpc": "2.0",
            "id": index + 1,
            "method": "tools/call",
            "params": {
                "name": tool,
                "arguments": json.loads(arguments.replace("<home>", str(placeholders.home))),
            },
        }
        for index, (tool, arguments) in enumerate(rows)
    ]
    for (tool, _), reply in zip(rows, exchange(mori_binary, home, env, requests), strict=True):
        mcp[tool] = reply


@when("an agent lists the MCP tools")
def lists_tools(mori_binary: Path, home: Path, env: dict[str, str], mcp: dict[str, dict]) -> None:
    (reply,) = exchange(mori_binary, home, env, [{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}])
    mcp["tools/list"] = reply


@then("every call succeeds")
def all_succeed(mcp: dict[str, dict]) -> None:
    for tool, reply in mcp.items():
        assert "result" in reply and not reply["result"].get("isError"), (tool, reply)


@then(parsers.parse('the "mori_where" answer names the tree "{name}" with purpose "{purpose}"'))
def where_names(mcp: dict[str, dict], name: str, purpose: str) -> None:
    answer = mcp["mori_where"]["result"]["structuredContent"]
    assert_that(answer["tree"]["name"], equal_to(name))
    assert_that(answer["tree"]["purpose"], equal_to(purpose))


@then(parsers.parse('the "mori_trees" answer lists only "{name}"'))
def trees_list(mcp: dict[str, dict], name: str) -> None:
    answer = mcp["mori_trees"]["result"]["structuredContent"]
    names = [row["tree"]["name"] for repo in answer.get("repos", []) for row in repo.get("trees", [])]
    assert_that(names, equal_to([name]))


@then(parsers.parse('they are "{names}"'))
def tool_names(mcp: dict[str, dict], names: str) -> None:
    tools = mcp["tools/list"]["result"]["tools"]
    assert_that([tool["name"] for tool in tools], equal_to(names.split(", ")))

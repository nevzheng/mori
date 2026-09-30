"""What the steps are built from: running mori, naming paths, and watching the disk.

Plain classes and functions with their dependencies passed in; conftest.py wires them into
fixtures. Nothing here knows about Gherkin.
"""

import re
import shlex
import stat
import subprocess
from collections.abc import Mapping
from dataclasses import dataclass, field
from pathlib import Path

# The google.rpc.Code numbers mori exits with.
CANONICAL_CODES = {
    "OK": 0,
    "INVALID_ARGUMENT": 3,
    "NOT_FOUND": 5,
    "ALREADY_EXISTS": 6,
    "FAILED_PRECONDITION": 9,
    "INTERNAL": 13,
}


@dataclass
class Mori:
    """Runs the real `mori` binary the way a user would: from `home`, with exactly `env`."""

    binary: Path
    home: Path
    env: Mapping[str, str]
    _last: subprocess.CompletedProcess[str] | None = field(default=None, init=False)

    def run(self, command: str) -> subprocess.CompletedProcess[str]:
        program, *args = shlex.split(command)
        assert program == "mori", f"steps only run mori, not {program!r}"
        self._last = subprocess.run(
            [str(self.binary), *args],
            env=dict(self.env),
            cwd=self.home,
            capture_output=True,
            text=True,
            timeout=60,
            check=False,
        )
        return self._last

    @property
    def last(self) -> subprocess.CompletedProcess[str]:
        assert self._last is not None, "no command has been run yet"
        return self._last


@dataclass(frozen=True)
class Placeholders:
    """Turns `<home>` and `$VARIABLE` in step text into real paths, using the scenario's env."""

    home: Path
    env: Mapping[str, str]

    def path(self, text: str) -> Path:
        text = text.replace("<home>", str(self.home))
        return Path(re.sub(r"\$([A-Z_]+)", lambda m: self.env[m.group(1)], text))


# What would differ if a path were created, removed or modified.
Entry = tuple[int, int, int, int]  # file type, permission bits, size, mtime in ns


def snapshot(root: Path) -> dict[Path, Entry]:
    """Every path under `root` (not `root` itself; symlinks not followed)."""
    entries = {}
    for path in root.rglob("*"):
        info = path.lstat()
        entries[path] = (
            stat.S_IFMT(info.st_mode),
            stat.S_IMODE(info.st_mode),
            info.st_size,
            info.st_mtime_ns,
        )
    return entries


@dataclass
class DiskWatch:
    """Remembers the disk under `root`, so a later step can ask what changed since."""

    root: Path
    _before: dict[Path, Entry] | None = field(default=None, init=False)

    def remember(self) -> None:
        self._before = snapshot(self.root)

    def changed_under(self, path: Path) -> list[Path]:
        """Paths at or under `path` that were created, removed or modified since `remember`."""
        assert self._before is not None, "nothing was remembered to compare with"
        now = snapshot(self.root)
        candidates = self._before.keys() | now.keys()
        inside = (p for p in candidates if p == path or path in p.parents)
        return sorted(p for p in inside if self._before.get(p) != now.get(p))


def mode(path: Path) -> int:
    """The permission bits of `path`."""
    return stat.S_IMODE(path.stat().st_mode)


def listed_paths(output: str, verb: str) -> set[Path]:
    """The paths on lines like `  created /x` (for `verb` "created") in mori's text output."""
    pattern = re.compile(rf"^\s+{re.escape(verb)} (.+)$")
    return {Path(m.group(1)) for line in output.splitlines() if (m := pattern.match(line))}

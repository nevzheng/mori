"""Extra precisely matchers."""

import re

from precisely.base import Matcher
from precisely.results import matched, unmatched


class _MatchesRegex(Matcher):
    def __init__(self, pattern: str) -> None:
        self._pattern = re.compile(pattern)

    def match(self, actual: object):
        if isinstance(actual, str) and self._pattern.fullmatch(actual):
            return matched()
        return unmatched(f"was {actual!r}")

    def describe(self) -> str:
        return f"a string fully matching /{self._pattern.pattern}/"


def matches_regex(pattern: str) -> Matcher:
    """Matches a string that fully matches `pattern`."""
    return _MatchesRegex(pattern)

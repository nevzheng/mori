"""Entry point that runs pytest inside Bazel's py_test."""

import sys

import pytest

if __name__ == "__main__":
    sys.exit(
        pytest.main(
            [
                "-p",
                "no:cacheprovider",  # runfiles are read-only
                "--strict-markers",
                "e2e",
                *sys.argv[1:],
            ]
        )
    )

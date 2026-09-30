"""Binds the feature files to pytest. Paths are relative to this file."""

from pytest_bdd import scenarios

scenarios(
    "../spec/cuj/00-init.feature",
    "../spec/cuj/01-clone.feature",
    "features/smoke.feature",
)

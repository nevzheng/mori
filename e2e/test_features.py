"""Binds the feature files to pytest. Paths are relative to this file."""

from pytest_bdd import scenarios

scenarios(
    "../spec/cuj/00-init.feature",
    "../spec/cuj/01-clone.feature",
    "../spec/cuj/02-ls.feature",
    "../spec/cuj/03-tree-create.feature",
    "../spec/cuj/04-tree-remove.feature",
    "../spec/cuj/06-gc.feature",
    "../spec/cuj/06-gc-apply.feature",
    "../spec/cuj/07-restore.feature",
    "../spec/cuj/skills.feature",
    "features/smoke.feature",
)

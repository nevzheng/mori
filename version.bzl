"""mori's version for Bazel builds. Keep it equal to `version` in the root Cargo.toml.

On `main` the version is the next release with `-dev` (the head channel); a release PR drops the
suffix, and the release workflow refuses a tag that doesn't match both."""

MORI_VERSION = "0.1.0-dev"

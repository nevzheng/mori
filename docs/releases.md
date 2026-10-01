# Release policy

When mori is released, how it is versioned, and what it promises before 1.0. The channels
themselves (head and release) are on [Install and release channels](install.md).

## Versions

mori is pre-alpha: versions are `0.MINOR.PATCH`.

- **Minor** (`0.2.0`): new commands or flags, changed behaviour or output, a database schema
  step.
- **Patch** (`0.2.1`): fixes only, no new behaviour.
- **Pre-release** (`0.1.0-alpha.1`): a tag ahead of a release, for early users to try. `alpha.N`
  counts up; the release itself follows once an alpha has had real use. mori's first tag is
  `v0.1.0-alpha.1`.
- **`main`** is always the next release with `-dev` (`0.2.0-dev`), so a head build says what it
  is heading toward. SemVer sorts `0.1.0-alpha.1` before `0.1.0-dev` (`alpha` before `dev`), so
  head is always newer than the alphas of the release it is heading toward.

## When a release is cut

A release is cut when all of these hold:

1. `main` is green, on every platform CI builds.
2. No open issue is labelled `priority/P0`, and none reports lost work.
3. The changes since the last release have run on head, in daily use, for at least three days.
4. The CHANGELOG entry is written: what changed, and anything a user must do.

There is no fixed schedule. A release happens when there is something worth shipping, at most every
two weeks; a fix for lost work ships as soon as it is ready.

## How a release is cut

1. A release PR sets the version in `Cargo.toml` and `version.bzl`: `-dev` becomes `-alpha.N` for a
   pre-release, or is dropped for a release. It dates the CHANGELOG entry.
   **Its review is the gate**: what it says is what ships, and the CHANGELOG entry becomes the
   release notes.
2. Merging it releases it. The release workflow checks both version files and the CHANGELOG
   entry, builds `mori` for macOS (Apple silicon and Intel) and Linux (x86-64 and arm64), attaches
   the archives and their SHA-256 checksums to a draft, and publishes it, which tags the merge
   commit `vX.Y.Z`. A version with a suffix is marked as a pre-release.
3. A PR sets `main` back to a `-dev` version: the same one after a pre-release, the next one after
   a release.

A fix for a release comes from a branch off its tag: fix, bump the patch version, tag
`vX.Y.(Z+1)`, push the tag (which releases it the same way), and bring the fix to `main` too.

Releases are immutable: once published, a release's tag and archives can't change, and GitHub
attests to them. Release tags can't be moved or deleted either. A broken release is fixed by the
next version.

## What mori promises before 1.0

- **No lost work across versions.** Every removal is journalled and restorable; an upgrade never
  removes a tree.
- **The database migrates forward by itself.** A newer mori upgrades an older database on first
  use; an older mori refuses a newer database rather than misreading it.
- **`--json` changes only with the API version.** Fields are added, never renamed or reused
  within `v1alpha1`; anything else is a new API version, and the CHANGELOG says so.
- **Everything else may change** (commands, flags, text output, layout of the root's `context/`
  folder) with a CHANGELOG line.

## Distribution

Releases and pre-releases are on GitHub Releases first. crates.io and a Homebrew tap follow once a
release has had real users; until then, head is installed with `cargo install --git`.

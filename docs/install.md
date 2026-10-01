# Install and release channels

mori is pre-alpha: the API is `mori.v1alpha1`, and anything may change in any release.

## Channels

| Channel     | What you get                            | How to install                                                  | Status             |
| ----------- | --------------------------------------- | --------------------------------------------------------------- | ------------------ |
| **head**    | The latest `main`, built from source    | `cargo install --git https://github.com/nevzheng/mori mori-cli` | The only one today |
| **release** | Tagged versions, with prebuilt binaries | Download from GitHub releases                                   | From v0.1.0        |

**head** is what mori's own development runs on. Every change on `main` has passed the full test
suite on jj and git, but nothing more: there is no soak time and no compatibility promise. Run
`cargo install` again to update.

**release** starts at v0.1.0. A version is tagged only after head has been used day to day for a
week without lost work or stuck trees. Each release attaches prebuilt binaries for macOS (Apple
silicon and Intel) and Linux, plus a changelog. Versions are `0.MINOR.PATCH`: a minor version
may break things, a patch only fixes them. Tags follow readiness, not a schedule.

There is no nightly, beta or dogfood channel: head already is the nightly, and dogfooding means
running head on real work and reporting what goes wrong. More channels can come once mori has
users outside its own development.

## Requirements

- Rust 1.94 or later, for the head channel.
- `jj` on `PATH` for jj clones (the default), or `git` for `mori clone --vcs git`.
- Optionally, `gh`, logged in, so `mori gc` can tell when a pull request merged.

## Report a problem

Open an issue at <https://github.com/nevzheng/mori/issues> with what you ran, what you expected,
and the output of `mori ls`. Agents can keep notes in `~/mori/context/projects/<repo>/`.

## Release policy

A release is a tag. Only the maintainer pushes tags, and a tag `v0.MINOR.PATCH` is cut when all of
these hold:

1. CI is green on the `main` commit being tagged.
2. No open issue labelled `blocker`: lost work, a stuck tree, or a wrong cleanup decision.
3. `CHANGELOG.md` has the version's entry, and `Cargo.toml` has its number.
4. For v0.1.0 only: head has been used day to day for a week.

Pushing the tag runs the release workflow, which builds the binaries and publishes the GitHub
release. A broken release is fixed by a new patch version, never by moving a tag.

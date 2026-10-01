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
silicon and Intel) and Linux, plus a changelog.

There is no separate dogfood channel. Dogfooding means running head on real work and reporting
what goes wrong.

## Requirements

- Rust 1.94 or later, for the head channel.
- `jj` on `PATH` for jj clones (the default), or `git` for `mori clone --vcs git`.
- Optionally, `gh`, logged in, so `mori gc` can tell when a pull request merged.

## Report a problem

Open an issue at <https://github.com/nevzheng/mori/issues> with what you ran, what you expected,
and the output of `mori ls`. Agents can keep notes in `~/mori/context/projects/<repo>/`.

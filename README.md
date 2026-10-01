# mori 森

*memento mori: every tree dies; your work doesn't.*

mori tends a forest of git and jj worktrees for you and your coding agents. It keeps one clone per
repo in a fixed layout, gives every tree an owner and a lifetime, teaches agents how to work in it,
and cleans up without ever losing a change.

Website: <https://nevzheng.github.io/mori/>

**Status: pre-alpha.** The API is `mori.v1alpha1` and the binary is `0.x`: there are no stability
guarantees, and anything may change in any release. The core journeys work (clone, trees, list,
safe removal, cleanup and restore, on jj or git), and mori is being dogfooded.

## Install

You need Rust (1.94 or later) and `jj` on `PATH` for jj clones, or `git` for git clones.

```sh
cargo install --git https://github.com/nevzheng/mori mori-cli
```

## Quick start

```sh
mori init                                      # set up ~/mori and the agent skills
mori clone github.com/acme/widget              # a jj clone; add --vcs git for git worktrees
mori tree create github.com/acme/widget --agent claude --task fix-login
mori ls                                        # every repo and tree, and unsaved work
mori gc                                        # which trees may go, and whether it's safe
```

**Agents:** point yours at the root once, e.g. add "Before working under ~/mori, read
~/mori/llms.txt" to its instructions file (AGENTS.md, CLAUDE.md, …). The skills there explain the
commands, the rules, and how agents work together.

## Build

Bazel is the build system. `cargo` works too, but Bazel is the supported path.

```sh
bazel test //...            # build everything, run clippy and rustfmt, run the tests
bazel run //crates/cli:mori -- --version
```

**jj:** mori supports no particular jj version yet; jj changes quickly and mori is pre-alpha. Tests
run against one pinned release (0.45.1 today, set in `MODULE.bazel`), which Bazel downloads, so you
don't need jj installed to run them.

## License

[Apache-2.0](https://github.com/nevzheng/mori/blob/main/LICENSE).

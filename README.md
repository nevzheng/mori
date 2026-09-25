# mori 森

*memento mori: every tree dies; your work doesn't.*

mori tends a forest of git and jj worktrees for you and your coding agents. It keeps one clone per
repo in a fixed layout, gives every tree an owner and a lifetime, moves work between agents, and
cleans up without ever losing a change.

**Status: pre-alpha.** The API is `mori.v1alpha1` and the binary is `0.x`: there are no stability
guarantees, and anything may change in any release. Nothing useful works yet.

## Build

Bazel is the build system. `cargo` works too, but Bazel is the supported path.

```sh
bazel test //...            # build everything, run clippy and rustfmt, run the tests
bazel run //crates/cli:mori -- --version
```

## License

[Apache-2.0](LICENSE).

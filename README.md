# mori 森

*memento mori: every tree dies; your work doesn't.*

You run several coding agents at once, and each one needs its own checkout. Worktrees pile up in
odd places, nobody remembers which one was for what, and cleaning up means guessing which of them
still holds work that was never pushed.

mori tends that forest for you and your agents. It keeps one clone per repo in a fixed layout,
gives every task its own tree with an owner, a purpose and a lifetime, tells agents where they are
and how to work together, and cleans up only what is safe, with an undo for everything it removes.
It works with jj (the default) and with plain git worktrees.

Website: <https://nevzheng.github.io/mori/>

**Status: pre-alpha.** The API is `mori.v1alpha1` and versions are `0.x`: anything may change in
any release, but mori never loses work across versions (see the
[release policy](https://nevzheng.github.io/mori/releases/)). The core journeys work, and mori is
being dogfooded.

## A day with mori

**Give a task its own tree.** One command, and the agent has a place to work that mori remembers:

```text
$ mori tree create widget --owner claude --task fix-login --purpose "Fix the login redirect"
Created tree claude-fix-login:
  path: ~/mori/trees/widget/claude-fix-login
  repo: github.com/acme/widget
  task: fix-login (owner claude)
  purpose: Fix the login redirect
  lifetime: task-done
```

**Know where you are.** An agent dropped into any directory asks first:

```text
$ mori where
github.com/acme/widget  tree claude-fix-login
  owner claude · task fix-login · task-done
  purpose: Fix the login redirect
  context: ~/mori/context/projects/widget/
```

**See the forest.** Every tree, who it's for, what it's for, and what exists only on this machine:

```text
$ mori ls
github.com/acme/widget  jj  ~/mori/repos/github.com/acme/widget
  NAME              OWNER   TASK       LIFETIME   WORK                PURPOSE
  claude-fix-login  claude  fix-login  task-done  edited, 1 unpushed  Fix the login redirect
  codex-docs        codex   docs       task-done  clean               Document the new flags
  default           nevin   -          pinned     clean               -
```

**Clean up without fear.** mori removes a tree only when its work is safe elsewhere: pushed, or
landed in a merged pull request. Everything it removes is pinned and journalled:

```text
$ mori tree remove widget claude-fix-login
error: claude-fix-login has work only this machine has: 1 unpushed change, working copy edited
hint: push it (or abandon it), then try again; `mori ls` shows what is unsaved

$ mori gc --apply --yes
Can go (1)
  github.com/acme/widget codex-docs  20K  LANDED  removed; undo: mori restore j-1790000000-0a1b0001
```

`mori doctor` finds drift (a tree deleted by hand, a conflicted bookmark) and says how to fix it.

## For agents

- **Skills and `llms.txt`.** `mori init` installs skills into `~/mori/context/`: how to use mori,
  jj and git in a mori repo, lead trees, and recipes for agents working together. Point your agent
  at `~/mori/llms.txt` once.
- **Where am I.** `mori where` tells an agent dropped into any directory which repo and tree it is
  in, what the tree is for, and where the repo's notes are.
- **JSON everywhere.** Every command takes `--json`; errors are `google.rpc.Status` with stable
  reasons and exit codes.

## Install

You need Rust (1.94 or later), and `jj` on `PATH` for jj clones or `git` for git clones.

```sh
cargo install --git https://github.com/nevzheng/mori mori-cli
```

```sh
mori init                                      # set up ~/mori and the agent skills
mori clone github.com/acme/widget              # a jj clone; add --vcs git for git worktrees
mori tree create widget --owner claude --task fix-login --purpose "Fix the login redirect"
mori                                           # the forest at a glance
```

More on the [install page](https://nevzheng.github.io/mori/install/), including shell completions.

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

# Changelog

What changed in each release of mori. Head (`main`) is ahead of the newest entry; see
[Install and release channels](https://nevzheng.github.io/mori/install/).

## 0.1.0-alpha.1 (2026-10-01)

The first pre-release, to prove the release flow and start dogfooding from a tag. Expect rough
edges; the [release policy](https://nevzheng.github.io/mori/releases/) says what is promised.

- `mori init`: sets up the root, its config and database, and the agent skills with `llms.txt`
  indexes.
- `mori clone`: one clone per repo under `repos/<host>/<owner>/<repo>`, as jj (the default,
  colocated with git) or git (`--vcs git`, trees are git worktrees), with a context folder per
  repo under `context/projects/`.
- `mori tree create`, `mori tree set` and `mori tree remove`: a tree per task, with an owner, a
  purpose and a lifetime; removal only when no work would be lost, and a hand-deleted tree's work
  is kept.
- `mori ls`: every repo and tree, what is unsaved and what has landed; `--size` for disk use,
  `--query`, `--owner` and `--status` to find trees. Bare `mori` shows the forest.
- `mori where`: tells an agent which repo and tree it is in.
- `mori gc` and `mori restore`: which trees may go and what each frees, `--free` to pick just
  enough, Bazel output left by deleted trees, confirmed batch removal that pins every removed
  tree's commit, and a journal to bring any of them back.
- `mori doctor`: reports drift between mori's records and the repos; `--fix` repairs what is safe.
- Disk: a warning when free space runs low, opt-in limits that only warn, and checks for build
  caches that are not shared across trees.
- Hints that steer agents and people toward the good path (`MORI_HINTS=0` turns them off).
- Repos by short name, readable text output with colour, help with examples, shell completions
  and a man page.
- Agent skills: using mori, jj and git in mori, lead trees (with the `ready/<topic>` hand-off),
  agent workflows and disk usage.

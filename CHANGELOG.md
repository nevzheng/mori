# Changelog

What changed in each release of mori. Head (`main`) is ahead of the newest entry; see
[Install and release channels](https://nevzheng.github.io/mori/install/).

## Unreleased (0.1.0)

The first release, after dogfooding.

- `mori init`: sets up the root, its config and database, and the agent skills with `llms.txt`
  indexes.
- `mori clone`: one clone per repo under `repos/<host>/<owner>/<repo>`, as jj (the default,
  colocated with git) or git (`--vcs git`), with a context folder per repo.
- `mori tree create` and `mori tree remove`: a tree per task, with an owner and a lifetime;
  removal only when no work would be lost.
- `mori ls`: every repo and tree, what is unsaved, and what has landed.
- `mori gc` and `mori restore`: a report of which trees may go, confirmed batch removal that pins
  every removed tree's commit, and a journal to bring any of them back.
- Agent skills: using mori, jj and git in mori, lead trees, and agent workflows.

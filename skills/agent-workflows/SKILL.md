---
name: agent-workflows
description: >-
  Recipes for agents working together in a repo mori manages: a coordinator with worker agents,
  exchanging work (change IDs, pushed bookmarks, patches), and restacking a stack onto trunk, with
  jj first and git where it applies. Use when splitting work across agents, combining their
  changes, handing work to another machine, or keeping a stack current.
---

# Agent workflows with mori

These are ways that work, not rules. mori gives each agent its own tree and removes trees safely;
how agents combine their work is plain jj (and git), described here. Read `using-mori` first, and
`vcs-in-mori` for which commands work where.

## Coordinator and workers

One agent (the coordinator) splits a task; worker agents each do a piece in their own tree.

1. **Coordinator** makes a long-lived tree for combining work:
   `mori tree create <repo> --agent <me> --task lead --lifetime pinned`. See `lead-tree`.
2. **Each worker** gets its own tree: `mori tree create <repo> --agent <name> --task <slug>`
   (or set `MORI_AGENT` once). One tree per piece of work.
3. **A worker finishing** describes each change (`jj describe`), stops editing, and reports its
   change IDs (`jj log -r '::<tree>@ ~ ::trunk()'`) with a line on what they do. Notes for the
   repo go in `~/mori/context/projects/<repo>/`.
4. **The coordinator** moves the changes into its stack (`jj rebase -s <first> -d <where>`),
   resolves conflicts in its own tree, tests, and pushes only with the person's yes.
5. **Cleanup:** once the coordinator's bookmark is pushed (or the pull request lands), the
   workers' trees are safe to remove: `mori tree remove <repo> <tree>`, or `mori gc --apply --yes`
   when the person asks.

Threads that run in parallel without a coordinator just push their own bookmarks and open their own
pull requests; no combining needed.

## Exchanging work

Pick the first that fits:

1. **Same machine, same clone:** share **change IDs**. Every tree already sees every change;
   the receiver rebases or merges them (`jj rebase -s <id> -d <where>`, `jj new <a> <b>`).
2. **Another machine or a cloud agent:** push a **bookmark** (a branch):
   `jj bookmark create <name> -r <id>` then `jj git push -b <name>`. The receiver runs
   `jj git fetch` and builds on `<name>@origin`.
3. **No shared remote:** send a **patch**. `jj diff --git -r <id> > fix.patch` (or
   `git format-patch` in the clone for authorship); the receiver applies it in their tree with
   `patch -p1 < fix.patch`, or `git am` in a colocated clone, and describes the result.

A patch copies work, so the sender's original stays unpushed: abandon it (`jj abandon <id>`) once
the receiver has it, or `mori` will keep the sender's tree as having unsaved work.

## Restacking

Keep a stack of changes on top of the latest trunk:

1. `jj git fetch`
2. `jj rebase -s <bottom of the stack> -d 'trunk()'`: the whole stack moves, descendants and
   all.
3. Resolve conflicts: `jj status` shows conflicted changes; edit, then `jj squash` the fix into
   the change it belongs to.
4. Fold small fixes into the right change: `jj absorb` moves working-copy edits into the commits
   that last touched those lines.
5. Push every bookmark in the stack: `jj git push -b <one> -b <two>`.

With git (in a colocated clone): `git fetch`, then
`git rebase --update-refs origin/main <top-branch>` moves the stack and its branches together;
push each branch with `--force-with-lease`.

After a pull request in the stack is squash-merged, rebase the rest onto trunk again; the merged
change's original commits drop out of the stack, and mori counts them as landed.

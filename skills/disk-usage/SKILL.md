---
name: disk-usage
description: >-
  How to keep a mori forest from filling the disk: measure with `mori ls --size`, free space safely
  with `mori gc`, and pass on cache warnings. Use when a disk is low or full, a build fails with "no
  space left on device", mori warns about free space or a limit, or before making many trees.
---

# Disk usage in a mori forest

Every tree is a full working copy with its own build outputs, so many trees cost a lot of disk.
mori shows the cost and frees space safely; it never deletes anything by itself. The person's
[Disk usage tips](https://nevzheng.github.io/mori/disk-usage/) page explains the setup side.

## When the disk is low

1. **Measure.** `mori ls --size --json` gives each tree's `sizeBytes` and the disk's free and total
   space. Sizes from the last 15 minutes are reused; add `--fresh` after you've built or deleted a
   lot.
2. **See what can go.** `mori gc --json` lists removable trees with their sizes, the trees blocked
   by unsaved work, and Bazel output left by deleted trees.
3. **Free a target amount.** `mori gc --free 200G` picks trees that are safe to remove (leftover
   Bazel output first, then the least recently changed trees) and shows the plan. Show the person
   the plan. Run `mori gc --free 200G --apply --yes` only when they've said to, or when your task
   already says you may clean up. Removed trees are journalled, so `mori restore <entry>` brings
   one back.
4. **If that isn't enough,** tell the person what is using the space. Don't delete anything mori
   didn't create, and never a tree with unsaved work: finish or push that work first, or ask.

## Warnings

mori warns and carries on; a warning never stops a command. Pass each one on to the person with
its fix.

- **Free space below the floor** (10% by default, `[disk] warn_below`): free space as above.
- **A limit** the person set (`max_trees`, `max_trees_per_repo`, `max_size`, `max_size_per_repo`):
  the warning names the limit and a `gc` command.
- **`NO_SHARED_CACHE`** (Bazel or Cargo) and **`CACHE_NOT_SHARED_ACROSS_TREES`** (ccache without
  `base_dir`), from `mori clone` and `mori doctor`: each tree builds from scratch. The fix is in
  the person's user-level config (`~/.bazelrc`, `~/.cargo/config.toml`, `ccache.conf`). Suggest it;
  don't edit their config yourself.

## Habits that keep trees small

- Remove a tree when its work has landed: `mori tree remove <repo> <name>`.
- In a huge repo where your task touches one corner, a sparse checkout shrinks the working copy:
  `jj sparse set --clear --add <dir>` in a jj tree, `git sparse-checkout set <dir>` in a git tree.
  It changes only your tree. Skip it if the build needs the whole repo.
- Don't share one Cargo `target/` between trees that build at the same time; Cargo locks it.

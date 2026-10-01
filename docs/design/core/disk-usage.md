# Disk usage: see what the forest costs, and free it in one step

|             |            |
| ----------- | ---------- |
| **Author**  | @nevzheng  |
| **Status**  | review     |
| **Area**    | core       |
| **Issue**   | none       |
| **PR**      | #97        |
| **Created** | 2026-10-01 |
| **Updated** | 2026-10-01 |

## Q1. What are you trying to do?

Let agents make as many trees as they like, and make the cost visible. mori measures how much disk
each tree and repo uses (Bazel's output for it included), warns when the disk is nearly full, and
frees a chosen amount in one step: `mori gc --free 200G` picks trees that are safe to remove and
shows the plan before anything goes. And mori nudges people toward the habits that keep a forest
small: it notices a Bazel or Cargo repo with no shared cache and says so, once at clone and in
`mori doctor`. An agent skill, `disk-usage`, and a "Disk usage tips" page say how to do all of this.

## Q2. What problems is this not trying to solve?

- **Limiting agents by default.** No caps on tree counts or disk unless someone sets one.
- **Deleting on its own.** Nothing is removed without `--apply --yes`, as today.
- **Configuring build tools.** mori doesn't write `~/.bazelrc` or `~/.cargo/config.toml`. The
  [shared caches](../../shared-caches.md) page has the recipes.
- **Shrinking a live tree.** Trimming the build output of idle trees (deleting `target/` in a tree
  that stays) is wishlist.
- **Sparse checkouts.** A docs recipe; a `tree create --sparse` flag is wishlist.
- **Cleaning other tools' caches.** mori removes only output left by trees it made.

## Q3. How is it done today, and what are the limits?

`mori ls` lists trees and `mori gc` sorts them into remove, blocked, keep and never, but neither
says how big anything is. With many trees and no shared caches a disk fills; people find out when
builds fail, then run `du` by hand to find what to delete. Bazel keeps each workspace's output base
outside the tree, so deleting a tree leaves its output behind, and nothing reports it. `[trees.lru]
max` caps lru trees per repo, but it removes by count, not size, and only for one lifetime.

## Q4. What is new in your approach, and why will it work?

Measure, show, and let gc act on a size target:

- **Sizes.** `mori ls --size` and `mori gc` add up each tree's directory and its Bazel output base.
  Plain `ls` doesn't measure.
- **Free space.** Every `ls` and `tree create` reads free space on the disk under the root (one
  `statvfs` call, instant) and warns below a floor: 10% free by default.
- **Bazel leftovers.** An output base whose workspace was under the mori root and no longer exists
  belongs to a tree mori deleted. Nothing can use it again. gc lists leftovers as safe to remove.
- **`gc --free <size>`.** From the trees gc already classed as safe to remove, plus leftovers, it
  picks leftovers first (only cache), then trees from the least recently changed, until the total
  reaches the target. Without `--apply` it shows the plan; with `--apply --yes` it carries it out.
  Removed trees are journalled and restorable as always.
- **Nudges toward shared caches.** A repo that builds with Bazel (`MODULE.bazel` or `WORKSPACE`)
  with no `--disk_cache` or `--remote_cache` in any bazelrc it reads, or with Cargo (`Cargo.toml`)
  and no `rustc-wrapper` or `RUSTC_WRAPPER`, gets a doctor finding `NO_SHARED_CACHE` (info) and a
  one-line tip after `mori clone`, both linking the tips page. The low-disk warning names `gc
  --free` and the page too. mori only reads these files; it never edits them.
- **Opt-in limits.** `[disk]` in `config.toml` can set a floor, tree counts and disk budgets. Each
  limit only warns and suggests the `gc --free` that fixes it.

It works because the hard part already exists: gc already decides what is safe, and removal already
pins and journals. This adds numbers and a target.

**Speed.** Measuring walks every file, so it runs in parallel across trees and directories, counts
allocated blocks (not apparent size), and counts a hard-linked file once. Each result is saved in
the database with when it was measured, and reused for 15 minutes; `--fresh` measures again.
Output says how old a number is.

## Q5. Who cares? If it works, what difference does it make?

- **The person running many agents** sees what the forest costs before the disk fills, and frees
  room with one command, with no risk to unsaved work.
- **Agents** get a skill that tells them what to do when a build fails for lack of space: measure,
  free what is safe, and tell the person about shared caches.
- **CUJ 05 (cleanup)** gains sizes and a target.

## Q6. What are the risks?

- **Slow on huge trees.** A cold measure of many large trees can take a while. Mitigated by the
  cache, parallel walks and never measuring in plain `ls`.
- **Wrong Bazel root.** The output user root can be moved (`--output_user_root`). mori looks in the
  default place for the platform and in `[disk] bazel_output_user_root` if set; it misses others,
  which only means it under-reports.
- **Deleting a leftover that is in use.** A leftover's workspace directory is gone, so no build
  can use it. If a tree is restored to the same path, Bazel rebuilds into a new output base; only
  cache is lost.
- **Stale numbers.** A cached size can be up to 15 minutes old. Output shows its age, and `gc
  --free` measures again before applying.

## Q7. How long will it take?

Six CLs after this doc:

1. Measuring: the `Disk` trait in core, its adapter in `mori-store`, the size cache table, `ls
   --size` with a summary line, and the low-disk warning in `ls` and `tree create`.
2. gc sizes: bytes per item, reclaimable total, and `gc --free`.
3. Bazel leftovers: found, sized, listed and removed by gc, and counted in tree sizes.
4. Opt-in limits in `[disk]`.
5. The shared-cache nudge: the `NO_SHARED_CACHE` doctor finding and the tip after `mori clone`.
6. The "Disk usage tips" page (shared caches, sizing, sparse checkouts, what mori does for you)
   and the `disk-usage` skill.

## Q8. How will we know it worked?

Mid-term: `mori ls --size` shows sizes and a summary on a real forest. Final: the scenarios in
Appendix E pass, and on a forest with leftovers `mori gc --free` frees what it planned.

## Appendix A. API (proto) changes

All additive.

- `ListTreesRequest.size` (bool) and `fresh` (bool).
- `TreeRow.size` (`Size`): `bytes`, `bazel_bytes`, `measured_at` (RFC 3339).
- `RepoTrees.bytes`: the sum for the repo, when measured.
- `ListTreesResponse.disk` (`Disk`): `root_bytes` (used by mori, when measured), `free_bytes`,
  `total_bytes`, `low` (bool), and `warnings` (repeated string, from limits).
- `GcRequest.free_bytes` (uint64) and `fresh` (bool).
- `GcItem.bytes` and `GcItem.selected` (picked by `--free`).
- `GcItem.Class` unchanged; a leftover is a `GcItem` with `kind = BAZEL_LEFTOVER` (new enum
  `GcItem.Kind`: `TREE`, `BAZEL_LEFTOVER`), class remove, reason `ORPHANED`.
- `GcResponse.reclaimable_bytes` and `GcResponse.disk`.
- `CreateTreeResponse.warnings` (repeated string).
- `CloneResponse.tips` (repeated string): the shared-cache tip.
- New doctor finding code `NO_SHARED_CACHE`, severity info, subject the repo, fix a link to the
  tips page.

No new error reasons: `--free` with nothing safe to free is a plan with nothing selected, not an
error.

## Appendix B. Design sketch

```text
mori-core::disk      pure: Usage, Limits, warnings(), pick_to_free(), parse/format sizes
mori-core::disk::Disk  trait: measure(path) -> bytes, free(path) -> (free, total),
                     bazel_output_bases() -> [(base, workspace)], remove_leftover(base)
mori-store::disk     the adapter: parallel walk, statvfs (rustix), Bazel root lookup
mori-store           schema v5: tree_sizes(path PRIMARY KEY, bytes, measured_at)
mori-app::ls / gc    ask Disk, cache, attach numbers, call the pure functions
```

`config.toml`, all optional:

```toml
[disk]
warn_below = "10%"           # or a size such as "200G"
max_trees = 100              # unset by default
max_trees_per_repo = 20      # unset by default
max_size = "2T"              # all of mori; unset by default
max_size_per_repo = "500G"   # unset by default
bazel_output_user_root = "/fast/bazel"
```

Text output, for example:

```text
$ mori ls --size
acme/widget (jj) 412G
  default            pinned       31G
  claude-fix-login   task-done   118G (bazel 96G)
  ...
mori uses 1.9T; 310G free of 4T (8%), below the 10% floor. 22G of Bazel leftovers: `mori gc`.

$ mori gc --free 200G
remove (safe)    6 trees, 2 leftovers   240G
  ...
blocked          1 tree                  80G
selected 5 items, 214G. Run `mori gc --free 200G --apply --yes` to remove them.
```

## Appendix C. Rejected designs

- **Default limits** (20 trees per repo, 100 overall, 50% of disk). Rejected by the owner: agents
  should get as much room as they want; mori shows and helps, it doesn't cap.
- **Hooks** (`pre_remove = "bazel clean --expunge"`). Rejected: finding leftovers afterwards
  needs no configuration and also catches trees deleted by hand.
- **Invalidating the size cache by VCS change.** Build outputs change without a commit, so the
  cache is time-based.
- **Measuring in plain `ls`.** Too slow to be the default.

## Appendix D. Failure modes and security

- A walk that hits an unreadable file counts what it can and marks the size partial; it never
  fails the command.
- The walk never follows symlinks and stays on one filesystem, so a link to `/` can't make a tree
  look huge or make mori read outside it.
- A leftover is removed only if its `DO_NOT_BUILD_HERE` names a path under the mori root that
  doesn't exist, and the base itself is under the Bazel output user root. Read-only files are made
  writable first. Leftovers aren't journalled: they hold only cache.
- `gc --free --apply` re-runs the safety checks per tree before removing it, as `--apply` does
  today.

## Appendix E. Test plan

```gherkin
Scenario: Sizes show how much each tree uses
  Given a repo with a task tree holding a 2 MB file
  When I run "mori ls --size"
  Then the task tree's size is at least 2 MB
  And the summary line says how much is free

Scenario: A full disk is called out
  Given the free-disk floor is set above the disk's free space
  When I run "mori ls"
  Then I see a warning that free space is below the floor

Scenario: gc frees a target amount from safe trees only
  Given two landed task trees and one with unsaved work
  When I run "mori gc --free 1K"
  Then the least recently changed landed tree is selected
  And the tree with unsaved work is not

Scenario: Bazel output left by a deleted tree is listed and removed
  Given an output base whose DO_NOT_BUILD_HERE names a deleted tree
  When I run "mori gc --apply --yes"
  Then the output base is gone

Scenario: A Bazel repo without a shared cache gets a tip
  Given a repo with a MODULE.bazel and no disk or remote cache in any bazelrc
  When I run "mori doctor"
  Then I see a NO_SHARED_CACHE finding that links the disk usage tips page

Scenario: Limits only warn
  Given max_trees_per_repo is 1 and the repo has one task tree
  When I run "mori tree create acme/widget second"
  Then the tree is created
  And I see a warning naming the limit and a gc command
```

Unit tests cover `pick_to_free`, limit warnings, size parsing and formatting. Integration tests
cover the walk (hard links counted once, symlinks not followed) and leftover detection in a temp
dir. Tests set the Bazel output user root in config, never touching a real home.

## Appendix F. Migration

Database schema v5 adds `tree_sizes`; migration is automatic. `config.toml` gains an optional
`[disk]` section. Nothing else changes.

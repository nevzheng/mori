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
- **Bazel leftovers.** An output base whose workspace is a tree path mori recorded (a tree row whose
  directory is gone, or a journalled removal) and no longer exists belongs to a tree mori deleted.
  Nothing can use it again. gc lists leftovers as safe to remove, unless a Bazel server still
  runs on it.
- **`gc --free <size>`.** From the trees gc already classed as safe to remove, plus leftovers, it
  picks leftovers first (only cache), then trees from the least recently changed (unknown change
  times last), until the total reaches the target; the last pick may overshoot it. Missing trees
  free nothing and aren't picked. Without `--apply` it shows the plan; with `--apply --yes` it
  carries it out. Removed trees are journalled and restorable as always.
- **Cache checks.** A family of doctor warnings, grouped by language, each shipped only where the
  setting can be read with certainty. Every one leads with the reason: agents make many trees, and
  each tree that builds from scratch costs disk.
  - **Bazel** (`MODULE.bazel`, `WORKSPACE`): no `--disk_cache`, `--remote_cache` or
    `--remote_executor` in any bazelrc it reads. Warning `NO_SHARED_CACHE`.
  - **Cargo** (`Cargo.toml`): no `rustc-wrapper`, shared target directory, or matching env var
    in any Cargo config. Warning `NO_SHARED_CACHE`.
  - **ccache** (a `ccache.conf` exists): no `base_dir`, so trees never share cache hits. Warning
    `CACHE_NOT_SHARED_ACROSS_TREES`.

  The same warning shows once after `mori clone`. Go, uv, pip, Poetry, Maven, Conan, pnpm, Yarn,
  npm downloads, NuGet and Pants share their caches by default and need no check. CMake launchers,
  the Gradle build cache, Turborepo, Nx and Buck2 can be configured too many ways to check without
  false alarms, so they get tips on the page and no check. mori only reads these files; it never
  edits them.
- **Opt-in limits.** `[disk]` in `config.toml` can set tree counts and disk budgets, none set by
  default. Each limit only warns and suggests the `gc --free` that fixes it. `[trees.lru] max`
  stays as it is: it decides which lru trees gc may remove, while these limits only warn.

It works because the hard part already exists: gc already decides what is safe, and removal already
pins and journals. This adds numbers and a target.

**Speed.** Measuring walks every file, so it runs in parallel across trees and directories, counts
allocated blocks (not apparent size), and counts a hard-linked file once within a tree. Each result
is saved in the database with when it was measured, and reused for 15 minutes; `ls --size --fresh`
measures again, and `gc --free --apply` always measures the trees it picked before removing them.
Output says how old a cached number is.

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
- **Sizes are an upper bound.** APFS clones and reflinks share blocks invisibly, and a file
  hard-linked into several trees (a pnpm store) counts in each. Sizes can over-count, never
  under-count what a tree's own directory holds.
- **Stale numbers.** A cached size can be up to 15 minutes old. Output shows its age, and `gc
  --free` measures again before applying.

## Q7. How long will it take?

Six CLs after this doc:

1. Measuring: the `Disk` trait in core, its adapter in `mori-store`, the size cache table, `ls
   --size` with a summary line, and the low-disk warning in `ls` and `tree create`.
2. gc sizes: bytes per item, reclaimable total, and `gc --free`.
3. Bazel leftovers: found, sized, listed and removed by gc, and counted in tree sizes.
4. Opt-in limits in `[disk]`.
5. The cache checks: Bazel, Cargo and ccache warnings in doctor, and the same after `mori clone`.
6. The "Disk usage tips" page, grouped by language (shared caches, remote caches, sizing, sparse
   checkouts, what mori does for you), and the `disk-usage` skill.

## Q8. How will we know it worked?

Mid-term: `mori ls --size` shows sizes and a summary on a real forest. Final: the scenarios in
Appendix E pass, and on a forest with leftovers `mori gc --free` frees what it planned.

## Appendix A. API (proto) changes

All additive. Sums (per repo, reclaimable, total used) are left to the client.

- `ListTreesRequest.include_sizes` (bool) and `skip_size_cache` (bool).
- `TreeRow.size_bytes`, `TreeRow.bazel_output_bytes`, `TreeRow.size_partial` (bool: some files
  couldn't be read) and `TreeRow.size_measured_at` (RFC 3339, as the database's other times).
- `ListTreesResponse.disk` (`Disk`): `free_bytes` and `total_bytes` of the disk under the root.
- `ListTreesResponse.warnings`, `CreateTreeResponse.warnings`, `CloneResponse.warnings`
  (repeated string): the low-space warning, limit warnings, and the shared-cache tip.
- `GcRequest.free_target_bytes` (uint64).
- `GcItem.size_bytes`. A tree picked by `--free` without `--apply` gets `OUTCOME_WOULD_REMOVE`,
  as a dry run does today.
- New enum `GcItem.Kind` (`KIND_UNSPECIFIED`, `KIND_TREE`, `KIND_BAZEL_LEFTOVER`) and
  `GcItem.kind`. A leftover is class remove, reason `ORPHANED`, path its output base.
- `GcResponse.disk` and `GcResponse.warnings`.
- New doctor finding codes `NO_SHARED_CACHE` and `CACHE_NOT_SHARED_ACROSS_TREES`, severity warn,
  subject the repo, fix a link to the tips page.

No new error reasons: `--free` with nothing safe to free is a plan with nothing picked.

## Appendix B. Design sketch

```text
mori-core::disk      pure: DiskPolicy, Floor, Space, warnings, pick_to_free, leftover rules,
                     size parsing and formatting
mori-store::disk     the filesystem side: parallel walk, free space (`df -Pk`, no unsafe code),
                     Bazel output bases and their DO_NOT_BUILD_HERE
mori-store           schema v5: tree_sizes(tree_id PRIMARY KEY, bytes, bazel_bytes, partial,
                     measured_at)
mori-app::ls / gc    measure or reuse, attach numbers, call the pure functions
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
- A leftover is removed only if its `DO_NOT_BUILD_HERE` (trimmed) names a tree path mori
  recorded that doesn't exist, compared by path components after resolving symlinks (so
  `/home/acme/mori2` never matches `/home/acme/mori`), the base itself is a direct child of the
  Bazel output user root, and no Bazel server runs on it (`server/server.pid.txt` names no live
  process). `install/` and `cache/` beside the bases have no `DO_NOT_BUILD_HERE` and are never
  touched. Read-only files are made writable first. Leftovers aren't journalled: they hold only
  cache.
- The cache checks read only. Bazel: `/etc/bazel.bazelrc`, the tree's `.bazelrc`, `~/.bazelrc`
  and each path in `$BAZELRC`, following `import` and `try-import`; a flag on a `build:<config>`
  line counts as set, and a `tools/bazel` wrapper skips the check. Cargo: `.cargo/config.toml` in
  the tree and every parent, and `$CARGO_HOME`'s, following `include`, plus `RUSTC_WRAPPER`,
  `CARGO_BUILD_RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER`, `CARGO_TARGET_DIR`,
  `CARGO_BUILD_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR`. An env var only ever proves a cache is
  set, and a tree with `.envrc`, `mise.toml` or `devbox.json` (which set env elsewhere) is skipped.
  A file that can't be read skips the check rather than warning.
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

Scenario: A Bazel repo without a shared cache gets a warning
  Given a repo with a MODULE.bazel and no disk or remote cache in any bazelrc
  When I run "mori doctor"
  Then I see a NO_SHARED_CACHE warning that links the disk usage tips page

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

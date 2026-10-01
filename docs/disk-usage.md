# Disk usage tips

Agents can make as many trees as they like, and every tree is a full working copy with its own
build outputs. That is what makes running many agents at once cheap to start and expensive in disk.
This page covers what mori shows and does about it, and what to set up for each language so trees
share their work instead of repeating it.

## What mori does

- **Shows the cost.** `mori ls --size` adds each tree's disk use and ends with how much the trees
  use and how much of the disk is free. Sizes measured in the last 15 minutes are reused; `--fresh`
  measures again. Every `ls` says how much is free, even without `--size`.
- **Warns, never blocks.** Below 10% free (`[disk] warn_below` in `config.toml`), `ls`, `tree
  create` and `gc` warn. Limits on tree counts and sizes (`max_trees`, `max_trees_per_repo`,
  `max_size`, `max_size_per_repo`) are off until you set them, and they only warn too.
- **Frees space safely.** `mori gc` lists which trees can go, with their sizes. `mori gc --free
  200G` picks trees that are safe to remove, least recently changed first, until that much is
  freed. Add `--apply --yes` to remove them; `mori restore` brings any of them back.
- **Cleans up Bazel output.** Bazel keeps each tree's outputs outside the tree, and they used to
  stay behind when the tree went. `mori gc` lists output left by trees mori deleted, and removes
  it with `--apply --yes`.
- **Checks your caches.** `mori clone` and `mori doctor` warn when a repo's builds share no cache
  across trees, where that can be read for certain: Bazel and Cargo (`NO_SHARED_CACHE`), and
  ccache without `base_dir` (`CACHE_NOT_SHARED_ACROSS_TREES`). mori only reads your config; it
  never changes it.

## The rule: user-level caches

Set caches in each tool's user-level config, in your home directory, never in the repo or a tree. A
new tree is just a new directory, so it picks up a user-level cache with no setup, and the cache
outlives every tree that used it. [Shared caches](shared-caches.md) has the recipes for Bazel and
Rust.

## By language

What each ecosystem already shares, what to add, and whether mori checks it.

| Language or tool | Shared by default                     | What to add                            | mori checks |
| ---------------- | ------------------------------------- | -------------------------------------- | ----------- |
| Bazel            | downloads (repository cache)          | a disk or remote cache in `~/.bazelrc` | yes         |
| Rust (Cargo)     | the registry and git checkouts        | sccache as `rustc-wrapper`             | yes         |
| C and C++        | nothing                               | ccache or sccache, with `base_dir`     | ccache      |
| Go               | `GOCACHE` and `GOMODCACHE`            | nothing                                | no need     |
| Python           | uv, pip and Poetry caches             | uv, which links venvs to its cache     | no need     |
| JavaScript       | pnpm's store, npm and Yarn downloads  | pnpm, so `node_modules` is hard links  | no need     |
| Java (Maven)     | `~/.m2/repository`                    | nothing for downloads                  | no need     |
| Java (Gradle)    | dependencies in `~/.gradle/caches`    | `org.gradle.caching=true`              | no          |
| Conan            | packages and binaries in `~/.conan2`  | nothing                                | no need     |

### Bazel

Add a disk cache to `~/.bazelrc` (`build --disk_cache=~/.cache/bazel-disk`), and a remote cache if
you have one; see [Shared caches](shared-caches.md#bazel). Each tree still gets its own output
base outside the tree; once the tree is gone, `mori gc` lists it and removes it.

### Rust

Use sccache as Cargo's compiler wrapper in `~/.cargo/config.toml`; see
[Shared caches](shared-caches.md#rust-sccache). Dependencies come from `~/.cargo/registry`, at the
same path in every tree, so they are compiled once. Don't point every tree at one shared `target/`
if agents build at the same time: Cargo locks it, and builds wait for each other.

### C and C++

ccache and sccache both cache C and C++ compiles. With CMake, set `CMAKE_C_COMPILER_LAUNCHER` and
`CMAKE_CXX_COMPILER_LAUNCHER` to `ccache` (in your environment, or a user-level CMake preset).
Trees live at different paths, and ccache only shares hits across paths when `base_dir` is set to
a directory above them. In `~/.config/ccache/ccache.conf`:

```text
base_dir = /home/acme
max_size = 50G
```

With debug info, also set `hash_dir = false`, or the working directory still makes every tree's
compiles different. mori checks only `base_dir`, because launchers can be set in too many places
to check without false alarms.

### Go, Python, JavaScript, Maven and Conan

These already keep one cache per user. Prefer uv over plain venvs, and pnpm over npm, so each tree
links to the shared store instead of copying it. Turborepo and Nx keep their local caches inside
each tree; use their remote cache if you want trees to share it.

### Gradle

Turn on the build cache in `~/.gradle/gradle.properties` with `org.gradle.caching=true`, so tasks
reuse outputs from other trees. mori doesn't check it, since builds and plugins can set it in too
many ways.

## Sparse checkouts

A huge repo where each task touches one corner can check out only that corner. Sparse patterns are
per tree, so each agent's tree can differ:

```sh
# a jj tree
jj sparse set --clear --add services/billing
# a git tree
git sparse-checkout set services/billing
```

This shrinks the working copy, not the build: a build that needs the whole repo will fail or fetch
the rest. mori has no flag for it yet.

## Sizing a disk

Build in one tree, then measure it with `mori ls --size`, plus its Bazel output base if any.
Multiply by the number of trees you keep at once. Shared caches shrink the per-tree part, since
only the first tree pays for a cold build.

# Shared caches

Every mori tree is a full working copy, so every tree builds from scratch unless your tools share
their caches. With a dozen trees that is a dozen copies of the same build outputs, and a large disk
fills faster than you'd expect.

The fix is to set caches in each tool's **user-level** config (your home directory), never in the
repo or the tree. A new tree is just a new directory, so it picks up a user-level cache with no
setup, and the cache outlives every tree that used it.

mori doesn't configure any of this for you. These are recipes.

## What is already shared

Many tools share their caches across directories by default. Nothing to do for these:

| Tool          | Shared by default                                                        |
| ------------- | ------------------------------------------------------------------------ |
| Cargo         | the registry and git checkouts in `~/.cargo` (not `target/`)             |
| pnpm          | the content-addressed store; each tree's `node_modules` is hard links    |
| uv            | its cache; each tree's virtualenv links to it                            |
| Go            | `GOCACHE` and `GOMODCACHE`                                               |
| Bazel         | the repository cache (downloads), under Bazel's output user root         |

## Bazel

### A local disk cache

Add to `~/.bazelrc`, which every workspace reads:

```text
build --disk_cache=~/.cache/bazel-disk
# Bazel 7.4 and later can keep it from growing forever:
build --experimental_disk_cache_gc_max_size=100G
```

With this, a second tree builds from the first tree's outputs instead of compiling everything
again. The disk cache is only a cache: deleting it at any time is safe, and it refills.

### A remote cache

If you have a remote cache (a cache service, or a self-hosted `bazel-remote`), it helps even more:
trees, machines and CI all share one set of outputs. Put its settings in `~/.bazelrc`, or in a
gitignored `user.bazelrc` that the repo's `.bazelrc` imports with
`try-import %workspace%/user.bazelrc`:

```text
build --remote_cache=grpcs://cache.example.com
build --remote_upload_local_results
# Keep the disk cache as a fast local layer in front of it.
build --disk_cache=~/.cache/bazel-disk
```

Keep credentials out of the repo: use your tool's credential helper or a file in your home
directory, never a tracked file.

### The catch: each tree's output base

Bazel keeps each workspace's build outputs in an **output base** outside the tree: under
`~/.cache/bazel/_bazel_$USER/` on Linux, or `/private/var/tmp/_bazel_$USER/` on macOS, one
directory per workspace path. When a tree is deleted, its output base stays behind. Over many
trees, these leftovers can be most of your disk.

Each output base has a `DO_NOT_BUILD_HERE` file that names the workspace it belongs to. This lists
the ones whose workspace is gone, with their size:

```sh
for base in ~/.cache/bazel/_bazel_"$USER"/*/; do
  [ -f "$base/DO_NOT_BUILD_HERE" ] || continue
  workspace=$(cat "$base/DO_NOT_BUILD_HERE")
  [ -d "$workspace" ] || echo "$(du -sh "$base" | cut -f1)  $base  (was $workspace)"
done
```

Each one listed belongs to a directory that no longer exists, so nothing can use it again. Bazel
makes some of its files read-only, so delete one with `chmod -R u+w <dir> && rm -rf <dir>`.

## Rust: sccache

Share one `target/` across trees only if you never build in two trees at once: Cargo locks it, so
parallel builds wait for each other. sccache shares compiled crates without that lock. In
`~/.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "sccache"
```

and in your shell profile:

```sh
export SCCACHE_CACHE_SIZE=50G
```

sccache doesn't cache incrementally compiled crates, so it helps most for dependencies and release
builds. It can also use a remote store (an object store bucket, Redis and others) so trees and
machines share it; see its documentation.

## Sizing a disk

Measure before you guess. Build in one tree, then add up the tree and its tool outputs:

```sh
du -sh ~/mori/trees/<repo>/<tree>
```

plus its Bazel output base, if any. Multiply by the number of trees you keep at once; shared caches
shrink the per-tree part, since only the first tree pays for a cold build.

`mori gc` lists which trees can go, which are blocked by unsaved work, and which their lifetime
still keeps. `mori gc --apply --yes` removes the safe ones, and `mori restore` brings any of them
back.

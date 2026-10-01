# Install and release channels

mori ships through **channels**. A channel is a promise about where a build comes from and how
often it changes. There is one channel today, and a second is coming.

| Channel     | What it is                                    | How you get it                                  | Who it's for                    |
| ----------- | --------------------------------------------- | ----------------------------------------------- | ------------------------------- |
| **head**    | Every commit on `main`, as soon as it merges  | `cargo install --git` (below)                   | Dogfooding, and contributors    |
| **release** | Tagged versions, `v0.1.0` onward (planned)    | Prebuilt binaries on GitHub Releases            | Everyone else, once it exists   |

**Dogfooding is head.** There is no separate dogfood channel: the people building mori use what is
on `main`, so the bugs they hit are the bugs in head. A third channel would only be worth having
once there are users who need something between the two.

Both channels are pre-alpha: the API is `mori.v1alpha1` and anything may change in any version.
mori never loses work across versions (removals are journalled and restorable), but commands,
flags and output can change.

## head

You need Rust 1.94 or later. For jj clones you also need `jj` on `PATH`, and for git clones `git`.

```sh
cargo install --git https://github.com/nevzheng/mori mori-cli
```

A head build's version ends in `-dev`, for example `0.1.0-dev`: the next release it is heading
toward. To upgrade, run the same command with `--force`, then update the skills in your root:

```sh
cargo install --force --git https://github.com/nevzheng/mori mori-cli
mori skills sync
```

`mori init` and `mori ls` print a one-line hint when the installed skills came from a different
mori than the one you're running.

## release (planned)

The first release, `v0.1.0`, is tagged once mori has been dogfooded:

- a week of daily use with no lost work;
- `mori doctor` built, to find and fix drift between mori's records and the disk;
- no open papercuts that would confuse an agent.

A tag `vX.Y.Z` builds `mori` for macOS (Apple silicon and Intel) and Linux (x86-64 and arm64) and
attaches the archives and their SHA-256 checksums to a GitHub release. A release's version has no
`-dev` suffix. Until then, head is the only channel.

## Which one am I running?

```sh
mori --version
```

`-dev` means head; a plain `X.Y.Z` means a release.

## After installing

```sh
mori init      # sets up ~/mori, its config and database, and the agent skills
```

Then point your agent at the root once, for example by adding "Before working under ~/mori, read
~/mori/llms.txt" to the instructions file it reads (`AGENTS.md`, `CLAUDE.md`, or your tool's rules).

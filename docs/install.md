# Install and release channels

mori ships through **channels**. A channel is a promise about where a build comes from and how
often it changes. There is one channel today, and a second is coming.

| Channel     | What it is                                         | How you get it                       | Who it's for                  |
| ----------- | -------------------------------------------------- | ------------------------------------ | ----------------------------- |
| **head**    | Every commit on `main`, as soon as it merges       | `cargo install --git` (below)        | Dogfooding, and contributors  |
| **release** | Tagged versions, `v0.1.0-alpha.1` onward (planned) | Prebuilt binaries on GitHub Releases | Everyone else, once it exists |

**Dogfooding is head.** There is no separate dogfood channel: the people building mori use what is
on `main`, so the bugs they hit are the bugs in head. A third channel would only be worth having
once there are users who need something between the two.

Both channels are pre-alpha: the API is `mori.v1alpha1` and anything may change in any version.
mori never loses work across versions (removals are journalled and restorable), but commands,
flags and output can change.

## head

You need Rust 1.94 or later. For jj clones you also need `jj` on `PATH`, and for git clones `git`.
Optionally, `gh`, logged in: without it `mori gc` can't tell that a pull request merged, so it
keeps a squash-merged tree until its bookmark is gone from the remote.

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

The first tag, the pre-release `v0.1.0-alpha.1`, is cut once mori has been dogfooded:

- a week of daily use with no lost work;
- `mori doctor` built, to find and fix drift between mori's records and the disk;
- no open papercuts that would confuse an agent.

A tag `vX.Y.Z` builds `mori` for macOS (Apple silicon and Intel) and Linux (x86-64 and arm64) and
attaches the archives and their SHA-256 checksums to a GitHub release. A release's version has no
`-dev` suffix. Until then, head is the only channel. When and how releases are cut, and what they
promise, is the [release policy](releases.md).

## Which one am I running?

```sh
mori --version
```

`-dev` means head; `-alpha.N` a pre-release; a plain `X.Y.Z` a release.

## After installing

```sh
mori init      # sets up ~/mori, its config and database, and the agent skills
```

Shell completions and the man page come from the binary:

```sh
mori completions zsh > ~/.zfunc/_mori    # or bash, fish, elvish, powershell
mori man | man -l -
```

Then point your agent at the root once, for example by adding "Before working under ~/mori, read
~/mori/llms.txt" to the instructions file it reads (`AGENTS.md`, `CLAUDE.md`, or your tool's rules).

## Report a problem

Open an issue at <https://github.com/nevzheng/mori/issues> with what you ran, what you expected,
what happened, and `mori --version`. Add `mori ls --json` if it is about trees, and `mori doctor`
if mori's records look wrong.

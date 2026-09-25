# Working on mori

Instructions for coding agents (and humans). `CLAUDE.md` is a symlink to this file.

## Rules

- **This repo is public.** No company names, private repo names, ticket keys, real paths or real
  data anywhere: code, tests, fixtures, docs or commit messages. Examples use made-up names such as
  `github.com/acme/widget`.
- **No attribution.** Never add `Co-Authored-By`, "Generated with" or similar lines to commits,
  PR titles or PR descriptions. CI rejects them.
- **Spec first.** A behavior change starts as a Given/When/Then scenario, then a failing test, then
  code.
- **One CL per PR.** A CL is one small, self-contained, reviewable change. PRs are squash-merged, so
  the PR title and description become the commit. The title is a Conventional Commit:
  `<type>(<scope>)!: <subject>`, with types `feat fix docs refactor perf test build ci chore style
  revert` and crate names as scopes. The description says what changed and why.
- **Reviews** use Conventional Comments (`issue:`, `suggestion:`, `nitpick:`, …), recommended not required.

## Version control

The repo is jj, colocated with git. Work in your own jj workspace; never edit the default workspace.
Don't run `gt`. Commits are signed when pushed.

## Code

- `mori-core` is pure and synchronous. Side effects (git, jj, GitHub, SQLite, clock, filesystem)
  sit behind traits defined in core and implemented by the adapter crates.
- No `unwrap`, `expect`, `panic!` or `todo!` outside tests. Lints are workspace-wide; don't silence
  them without a reason in the code.
- Errors follow Google AIP-193: each domain error has a canonical code and a stable
  `UPPER_SNAKE_CASE` reason.
- Tests follow the pyramid: most are unit tests of the core, then integration tests against
  throwaway git and jj repos in temp dirs, then a few end-to-end tests of the binary. Tests never
  touch a real home directory.

## Build and test

Bazel is the build system (`cargo` also works, but isn't the supported path):

```sh
bazel test //...    # builds everything; clippy and rustfmt run on every target; runs the tests
```

Lints live in `Cargo.toml` (`[workspace.lints]`) and Bazel applies them through
`extract_cargo_lints`. Adding a crate means adding it to the `manifests` list in `MODULE.bazel`.

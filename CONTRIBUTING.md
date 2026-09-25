# Contributing

mori is pre-alpha. The contribution policy (DCO or CLA) isn't decided yet, so please open an issue
before sending code.

- **CLs:** one small, self-contained change per pull request. PRs are squash-merged.
- **PR titles** are [Conventional Commits](https://www.conventionalcommits.org/); CI checks them.
- **Descriptions** explain what changed and why. No attribution lines.
- **Review comments** use [Conventional Comments](https://conventionalcomments.org/) where they
  help.
- **Build with Bazel:** `bazel test //...` builds everything, runs clippy and rustfmt on every
  target, and runs the tests. `cargo` also works, but Bazel is the supported path.
- **Share a disk cache** across checkouts by adding `common --disk_cache=<path>` to your
  `~/.bazelrc`.
- Run `bazel test //...` before pushing. See [AGENTS.md](AGENTS.md) for the full rules.

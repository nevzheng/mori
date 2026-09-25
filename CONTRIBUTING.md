# Contributing to mori

Thanks for your interest. mori is a hobby project: I build it in my free time, for fun. Issues
and pull requests are welcome.

## Expectations

I reply when I can, and sometimes not at all. There's no schedule, roadmap promise or support
guarantee. A quiet issue isn't a rejection, and a closed one isn't personal. mori is pre-alpha:
anything may change in any release.

## Ways to help

- **Report a bug:** use the bug form. Include your version (`mori --version`), OS, the command,
  what you expected and what happened.
- **Suggest a feature:** use the feature form and start with the problem, not the solution.
- **Ask a question:** use the question form.
- **Fix something small** (typos, docs, an obvious bug): open a PR directly.
- **Build something bigger:** see *Spec- and design-driven development* below. Unsure whether it
  fits? Open an issue and ask first.
- **Security issues:** never in a public issue. See
  [SECURITY.md](https://github.com/nevzheng/mori/blob/main/SECURITY.md).

## The bar

mori aims for great engineering, product and UX, and holds every change to a high
engineering standard. For a good picture of what that looks like, Google's
[Engineering Practices](https://google.github.io/eng-practices/) are an excellent
reference, especially:

- [The standard of code review](https://google.github.io/eng-practices/review/reviewer/standard.html)
- [What to look for in a code review](https://google.github.io/eng-practices/review/reviewer/looking-for.html)
- [Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html)
- [Writing good CL descriptions](https://google.github.io/eng-practices/review/developer/cl-descriptions.html)

Use whatever tools help. Coding agents are welcome, and we encourage using them
responsibly. If AI helped, you're welcome to say so in a PR comment. (Not in the
description: it becomes the commit message.)

Slop isn't accepted: changes the author doesn't understand, can't explain, or
didn't verify. Every contribution meets the same bar, however it was made:

- **You own it.** You understand every line, can say why it's right, and answer
  review questions yourself.
- **It's verified.** Tests prove the behavior; user-visible behavior starts as a
  spec scenario. `bazel test //...` passes.
- **It's one CL.** One small, focused change per pull request.
- **It comes with its design.** Changes beyond a small fix include their design
  doc, written first.

## Spec- and design-driven development

We write the spec and the design first, then the code. For a user-visible change,
that means a Gherkin scenario in `spec/cuj/` before the implementation. For new
commands, new concepts, changes to the model or the API, and anything hard to
undo, it also means a design doc in `docs/design/<area>/`, following Apache
Spark's [SPIP](https://spark.apache.org/improvement-proposals.html) format:

- **Q1–Q8, with no jargon:** what you're trying to do, what it doesn't solve, how
  it's done today, what's new, who cares, the risks, how long it takes, and how
  we'll know it worked.
- **Appendices:** A. API (proto) changes · B. Design sketch · C. Rejected designs ·
  D. Failure modes and security · E. Test plan (CUJs and scenarios) · F. Migration.

The doc is part of the change, and either order works:

- **Doc first:** send the design doc on its own for early feedback on the
  direction, then the code in follow-up PRs.
- **Doc and code together:** send them in the same PR.

Either way, reviews start with the doc. Unsure whether something needs one? Open
an issue and ask.

## Getting started

- Bazel is the build system: `bazel test //...` builds everything, runs clippy and rustfmt on
  every target, and runs the tests, including the end-to-end CUJ scenarios.
- `cargo` works too, but Bazel is the supported path.
- The API is proto-first: edit `proto/`, run `bazel run //tools/protogen`, commit the result.
- [AGENTS.md](https://github.com/nevzheng/mori/blob/main/AGENTS.md) has the full working rules; they
  apply to humans and agents alike.

## Pull requests

- **One CL per PR**, squash-merged: the PR title and description become the commit.
- **Title:** a Conventional Commit, `<type>(<scope>)!: <subject>` (CI checks it).
- **Description:** what changed and why (see Google's guide above). No attribution lines
  (`Co-Authored-By`, "Generated with"); CI rejects them.
- **CI must pass.** Draft PRs are fine for early feedback.
- **Stale PRs:** if a PR waits on its author for a few weeks, I may close it; reopen any time.

## Reviews

Reviews use [Conventional Comments](https://conventionalcomments.org/) (`issue:`, `suggestion:`,
`nitpick:`, `question:`, …) and are about the change, never the person.

## Code of conduct

Everyone taking part in mori follows the
[code of conduct](https://github.com/nevzheng/mori/blob/main/CODE_OF_CONDUCT.md).

## License

Unless you say otherwise, anything you submit is licensed under
[Apache-2.0](https://github.com/nevzheng/mori/blob/main/LICENSE), like the rest of mori (section 5
of the license). No CLA.

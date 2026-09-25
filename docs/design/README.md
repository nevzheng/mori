# Design docs

mori is built spec- and design-first. New commands, new concepts, changes to the model or the API,
and anything hard to undo come with a design doc. Small fixes don't need one; if you're unsure,
open an issue and ask.

Docs follow Apache Spark's [SPIP](https://spark.apache.org/improvement-proposals.html) format:
eight plain-language questions, then appendices for the details. Start from
[TEMPLATE.md](TEMPLATE.md).

## Layout

```text
docs/design/
  README.md
  TEMPLATE.md
  <area>/
    <what-it-does>.md              a doc on its own
    <what-it-does>/README.md       a doc with diagrams or large appendices, beside it
```

- **Area:** a crate or subsystem, such as `cli`, `core`, `api` or `store`. Add a new directory
  when none fits.
- **Name:** plain English saying what the design does, in lowercase words joined by hyphens, e.g.
  `cli/init-sets-up-the-root.md` or `core/cleanup-never-loses-work.md`. No numbers in names.
- **Numbers live inside the doc:** its metadata table links the tracking issue and the PR, which
  gives each design a stable reference without clashes.

## Statuses

Only accepted designs live on `main`. Drafts and reviews happen in pull requests, and a design
that isn't accepted is closed with its PR: the discussion stays there.

| Status       | Where         | Meaning                                                      |
| ------------ | ------------- | ------------------------------------------------------------ |
| `draft`      | an open PR    | Being written. Feedback welcome, nothing decided.            |
| `review`     | an open PR    | Ready for review.                                            |
| `accepted`   | `main`        | Agreed. Code lands against it.                               |
| `superseded` | `main`        | Replaced by a later design; the doc links to it.             |

An accepted doc isn't rewritten when plans change: fix small errors in place, and write a new doc
that supersedes it for a real change of direction.

## How a doc moves

The doc is part of the change, not a gate. Either order works:

- **Doc first:** send the doc on its own PR for early feedback on the direction, then the code in
  follow-up PRs.
- **Doc and code together:** send them in the same PR.

Either way, review starts with the doc. The status changes in the same PR that settles it. A design
issue (the "Design proposal" form) is optional, useful for testing an idea before writing.

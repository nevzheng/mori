# CLI style guide

How mori's command line looks and behaves, for people and for agents. New commands and output
follow it; a change that breaks a rule says why in its PR. Sources: [clig.dev](https://clig.dev),
[12-factor CLI apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46), the
[GNU](https://www.gnu.org/prep/standards/html_node/Command_002dLine-Interfaces.html) and
[POSIX](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html) conventions,
[NO_COLOR](https://no-color.org), and Google's [AIPs](https://google.aip.dev) for the API.

## Two audiences, one command

Every command serves people and agents, through two outputs:

- **`--json`** is the contract for agents and scripts: one JSON object on stdout, the proto
  message for the command (or a `google.rpc.Status` on failure). It never has colour, glyphs or
  prose that must be parsed, and it changes only with the API.
- **Text** is for people. It may change in any release. On an interactive terminal it may use
  colour and glyphs (below); anywhere else (a pipe, a file, CI, `NO_COLOR`, `TERM=dumb`) it is
  plain ASCII text, so an agent reading it still gets something clean.

## Vocabulary

| Word        | Means                                                                 |
| ----------- | --------------------------------------------------------------------- |
| root        | the directory mori manages (`~/mori` by default)                      |
| repo        | a repo mori cloned, named `host/owner/repo`                           |
| clone       | the repo's checkout under `repos/`; also its base tree, `default`     |
| tree        | a workspace (jj) or worktree (git) for one piece of work              |
| owner       | who a tree is for (`--owner`; `MORI_AGENT` sets the default)          |
| lifetime    | when a tree may go: `pinned`, `task-done`, `ttl:<n>d`, `lru`          |
| landed      | its work reached the remote's trunk, or its pushed branch was merged  |
| unsaved     | edits or commits that exist only on this machine                      |

Verbs: `create`, `remove`, `set` (change fields), `ls` (list), `sync`. Never `describe`: jj
already uses it for commit messages.

## Commands and flags

- Nouns group commands that act on one thing (`mori tree create|set|remove`); commands that span
  the root stay top-level (`ls`, `gc`, `doctor`, `restore`, `where`).
- Long flags always (`--dry-run`); short flags only for the most common ones, and never for
  anything that changes or removes.
- **`--dry-run`** on every command that changes anything: do every check, change nothing, and say
  what would happen.
- **`--yes`** confirms anything that removes or repairs in bulk (`gc --apply`, `doctor --fix`).
  mori never prompts: no interactive questions, so agents and scripts never hang.
- A repo is accepted in any form `mori clone` takes, and by its short name when exactly one repo
  matches.
- Environment variables are documented in `mori --help`: `MORI_ROOT`, `MORI_AGENT`, `MORI_GH`,
  `MORI_HINTS`, `NO_COLOR`.

## Output

- Results on **stdout**; errors, notes and hints on **stderr**.
- Text leads with the answer, then the detail. Paths are shown relative to `~`; JSON keeps them
  absolute.
- Lists are columns with a header; a column that would be the same on every row is left out.
- Nothing prints just to say it is working. Long operations show progress on stderr, only on a
  terminal.

## Hints

A hint is a suggestion for the next step, after a command that succeeded: what mori noticed, then
the command that acts on it. Text puts it on stderr; JSON puts it in the response's `hints`, with a
stable code.

```text
hint: tree claude-auth has no purpose; one line on what it is for shows in ls and where: `mori tree set github.com/acme/widget claude-auth --purpose "<what for>"`
```

- A hint fires only for whoever it is about. Agent hints need `MORI_AGENT`, so a person never sees
  them.
- A command gives at most two. `MORI_HINTS=0` turns them off.
- A hint is a fixed template that names only repos, trees and counts, never a purpose or other
  text someone wrote, so it can't carry planted instructions.

| Code              | When                                             | Where                  |
| ----------------- | ------------------------------------------------ | ---------------------- |
| `IN_PERSONS_ROOT` | an agent is in the clone, a person's checkout    | `where`                |
| `NOT_YOUR_TREE`   | an agent is in a tree another owner has          | `where`                |
| `NO_PURPOSE`      | an agent's tree has no purpose                   | `where`, `tree create` |
| `LANDED_TREES`    | trees hold work that landed, so gc may free them | `ls`, `mori`           |

## Errors

Every failure is a `google.rpc.Status` (AIP-193): a canonical code, which is the exit code, and a
stable `UPPER_SNAKE_CASE` reason with its domain. Text errors are written for the person:

```text
error: claude-fix-login has work only this machine has: 1 unpushed change
hint: push it, or abandon it, then run `mori tree remove` again
  TREE_HAS_UNSAVED_WORK  exit 9
```

- The first line says what happened, in plain words, naming the thing it is about.
- `hint:` says what to do next, with the command when there is one.
- The reason and exit code close it, for searching and for scripts.

Exit codes: 0 success; otherwise the canonical code (3 invalid argument, 5 not found, 6 already
exists, 9 failed precondition, 13 internal). Reports aren't failures: `mori doctor` and `mori gc`
exit 0 when they find problems; their findings say so.

## Help

- Every command has a one-line summary, a paragraph on what it does and what it never does, and
  two or three examples.
- Value names say what goes there: `--task <SLUG>`, `--lifetime <LIFETIME>`.
- Typos get a suggestion. `mori completions <shell>` prints shell completions; the man page is
  generated from the same definitions.

## Colour and glyphs

Only when stdout is a terminal, `NO_COLOR` is unset, `TERM` isn't `dumb`, and `--json` isn't
given; `--color auto|always|never` overrides. The palette has four roles. mori uses the
terminal's own ANSI colours, so it follows the person's theme; on the site they are Everforest's:

| Role    | Colour             | For                                       |
| ------- | ------------------ | ----------------------------------------- |
| ok      | green `#a7c080`    | clean, created, removed, landed           |
| warn    | yellow `#dbbc7f`   | edited, unpushed, warnings                |
| problem | red `#e67e80`      | refusals, problems                        |
| quiet   | grey `#859289`     | paths, IDs and other metadata             |

Names are bold. Glyphs are box drawing for trees (`├─ └─`) and `✓ ! ✗` for state, only with a
UTF-8 locale, with ASCII fallbacks (`|- \-`, `ok ! x`). A glyph or colour never carries meaning
that a word on the same line doesn't also carry. No emoji, no banners, and `--version` is one
line. The one mark is `森` in the header of the terminal dashboard (`mori` with no arguments).

## API

The proto follows the AIPs it names: `validate_only` for dry runs (AIP-163), `Status` and
`ErrorInfo` for errors (AIP-193), plain filters for lists (AIP-160, no paging: a forest is small),
and an `update_mask` for updates (AIP-134). Removed fields are `reserved`, never reused.

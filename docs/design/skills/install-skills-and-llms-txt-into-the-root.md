# Skills in the root: mori puts its agent skills and an llms.txt index where agents look

|             |            |
| ----------- | ---------- |
| **Author**  | @nevzheng  |
| **Status**  | accepted   |
| **Area**    | skills     |
| **Issue**   | none       |
| **PR**      | this PR    |
| **Created** | 2026-09-30 |
| **Updated** | 2026-10-01 |

> **Changed after acceptance (2026-10-01):** The context half moved under `context/`:
> `context/skills/`, a generated `context/llms.txt`, and one `context/projects/<repo>/` folder of
> notes per cloned repo. `mori skills sync` moves an older root's files there. The text below is the
> design as accepted.

## Q1. What are you trying to do?

Make mori's agent skills available on every machine that runs mori, not only in its source
repo. `mori init` puts them in the root's `skills/` directory with a generated `skills/llms.txt`
index and a short root `llms.txt`, so any agent working under the root can find out how to use
mori. `mori skills sync` updates them after mori is upgraded, and never overwrites a skill or index
someone changed.

## Q2. What problems is this not trying to solve?

- **Changing agent tools' own config.** mori doesn't write into `~/.claude`, `~/.codex`, Cursor
  rules or any other tool's files. It documents how to point a tool at the root (Q5, Appendix B).
- **A skills registry or package manager.** No downloading, no dependencies between skills, no
  versions for skills other than mori's own.
- **`projects/` and any grouping of the context half.** Still reserved; a workspace profile to
  shape it stays on the wishlist.
- **Stability.** The context half (`skills/`, `projects/`, `llms.txt`) keeps its "no stability
  guarantee" from the forest design: paths inside it may change between releases.

## Q3. How is it done today, and what are the limits?

The skills live in the source repo (`skills/<name>/SKILL.md`, indexed by `skills/llms.txt`), and
the website's `llms.txt` links to them. An agent only finds them if someone points it at the repo
or the site. On a machine where mori is just installed, `~/mori/skills/` is empty: the forest
design reserved the name, and nothing fills it.

## Q4. What is new in your approach, and why will it work?

The skills ship inside the `mori` binary, so the installed version always matches the installed
mori. mori writes them into the root with the same rule it uses for everything else: **it only
changes what it wrote itself.** A small manifest in mori's state directory records the hash of
every file mori wrote. On a sync, a file whose content still matches its hash is mori's to
update; a file that differs was edited by someone and is left alone and reported. Skills people
add themselves sit beside mori's, and the generated indexes list them too.

This works for any agent because the layout is plain files: a `skills/<name>/SKILL.md` directory
per skill (the format several agent tools read), and `llms.txt` indexes that anything can read.
No tool-specific integration is needed.

## Q5. Who cares? If it works, what difference does it make?

- **Agents** working under `~/mori` can read `~/mori/llms.txt`, follow it to `skills/llms.txt`,
  and learn the commands, errors and rules (`using-mori`, `lead-tree`) without being told.
- **People** get one place for skills of their own, indexed with mori's, organized however they
  like, and safe from being overwritten by upgrades.
- **Tool setup** becomes one line per tool: point it at `~/mori/skills` or `~/mori/llms.txt`.

## Q6. What are the risks?

- **Stale skills after an upgrade** if nobody runs `mori skills sync`. Mitigated: `mori init` and
  `mori ls` mention when the installed skills are older than the binary's (one line, no action).
- **Name clashes** between a person's skill and a later mori skill of the same name. mori never
  overwrites a directory it didn't write; the sync reports the clash and skips that skill.
- **Agents trusting skill text.** A skill is instructions; one planted in the root could steer an
  agent. mori only writes its own; people own what they add. Same trust as any skills directory.

## Q7. How long will it take?

Three CLs after this doc:

1. core: the skills plan: which files to write, update, skip or report, from the embedded skills,
   the manifest and what is on disk (pure, unit tests).
2. store: embed the repo's `skills/` in the binary; write files and the manifest; generate the
   indexes.
3. cli: `mori init` installs missing skills; `mori skills sync [--dry-run] [--json]`; e2e
   scenarios; the `using-mori` skill gets a section.

## Q8. How will we know it worked?

- **Mid-term:** after `mori init` on a clean machine, `~/mori/skills/using-mori/SKILL.md` and both
  `llms.txt` files exist, and `skills/llms.txt` lists every skill directory.
- **Final:** the scenarios in Appendix E pass: an upgrade updates untouched skills, keeps edited
  ones, and never touches a person's own skills.

## Appendix A. API (proto) changes

A `SyncSkills` RPC: request `{validate_only}`; response lists each skill with an action
(`ACTION_INSTALLED`, `ACTION_UPDATED`, `ACTION_UNCHANGED`, `ACTION_KEPT_EDITED`,
`ACTION_SKIPPED_NOT_OURS`) and the paths of the regenerated indexes. `InitResponse` lists
installed skill files among its created paths. Nothing existing breaks.

## Appendix B. Design sketch

```text
$MORI_ROOT/
  llms.txt                     generated: what mori is, the root layout, link to skills/llms.txt
  skills/
    llms.txt                   generated: one line per skill directory, from its frontmatter
    using-mori/SKILL.md        mori's (written from the binary)
    lead-tree/SKILL.md         mori's
    my-review-flow/SKILL.md    a person's own: indexed, never touched
$XDG_STATE_HOME/mori/skills.json   what mori wrote: path, sha256, mori version
```

### The rule for each file mori ships

| On disk                                  | In the manifest    | `sync` does             |
| ---------------------------------------- | ------------------ | ----------------------- |
| absent                                   | any                | writes it (installed)   |
| same as the binary's                     | any                | nothing (unchanged)     |
| matches the hash mori last wrote         | yes                | rewrites it (updated)   |
| differs from what mori last wrote        | yes                | keeps it, reports it    |
| present, never written by mori           | no                 | skips it, reports it    |

`mori init` does only the first row: it adds and never changes, as it always has. The two
`llms.txt` files follow the same rule, so a person who edits an index keeps their edit (and loses
automatic updates to it until they delete it).

### The generated indexes

`skills/llms.txt` has a heading, a one-paragraph note on the layout, and one line per directory
under `skills/` that holds a `SKILL.md`: `- [<name>](<dir>/SKILL.md): <description>`, sorted by
name, mori's and people's alike. The root `llms.txt` says what mori is, shows the layout, and
links to `skills/llms.txt` and to mori's website.

### Pointing tools at it (documented, not automated)

- A tool that reads a skills directory: link or configure it to `~/mori/skills` (or link single
  skills, e.g. `ln -s ~/mori/skills/using-mori ~/.claude/skills/using-mori`).
- A tool that reads instructions files (such as `AGENTS.md`): one line, "Before working under
  `~/mori`, read `~/mori/llms.txt`."

## Appendix C. Rejected designs

- **Write into each agent tool's config** (`~/.claude/skills`, …). It prescribes tools, touches
  files mori didn't make, and breaks as tools change. A later opt-in `mori skills link <tool>` is
  possible.
- **Install from the network.** Needs a release channel and trust in it; the binary already has
  the right version.
- **A `mori/` namespace inside `skills/`** (`skills/mori/using-mori`). Many tools expect one level
  of skill directories. The manifest, not a directory name, says what is mori's.
- **Always overwrite mori's skills.** Loses people's edits, which breaks mori's first rule.

## Appendix D. Failure modes and security

- **Interrupted sync:** each file is written to a temporary name and renamed, then the manifest is
  updated. A crash leaves either the old or the new file; a file whose hash isn't in the manifest
  yet is treated as edited, so the next sync reports it rather than guessing.
- **Symlinks:** mori doesn't follow a symlink in `skills/`; it reports it and skips it.
- **Paths:** skill names come from the binary (fixed at build time) and from directory names under
  `skills/` (indexed, never written).

## Appendix E. Test plan

```gherkin
Scenario: init installs mori's skills and the indexes
  Given a clean machine
  When I run "mori init"
  Then "<home>/mori/skills/using-mori/SKILL.md" exists
  And "<home>/mori/skills/llms.txt" lists "using-mori" and "lead-tree"
  And "<home>/mori/llms.txt" links to "skills/llms.txt"

Scenario: sync updates skills nobody edited
  Given mori's skills were installed by an older mori
  When I run "mori skills sync"
  Then "using-mori" is updated

Scenario: sync keeps a skill someone edited
  Given someone edited "<home>/mori/skills/using-mori/SKILL.md"
  When I run "mori skills sync"
  Then the edit is still there
  And the output says "using-mori" was kept because it was edited

Scenario: a person's own skill is indexed and never touched
  Given "<home>/mori/skills/my-flow/SKILL.md" exists with a name and description
  When I run "mori skills sync"
  Then "<home>/mori/skills/llms.txt" lists "my-flow"
  And "<home>/mori/skills/my-flow/SKILL.md" is unchanged
```

Unit tests in core cover every row of the rule table; store tests cover the atomic write and the
manifest.

## Appendix F. Migration

Existing roots have an empty `skills/`. The first `mori init` or `mori skills sync` after upgrading
installs the skills and indexes. Nothing else changes; undoing it is deleting the files.

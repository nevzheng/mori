# The forest: where mori keeps repos and trees, and what it remembers about them

|             |                               |
| ----------- | ----------------------------- |
| **Author**  | @nevzheng                     |
| **Status**  | accepted                      |
| **Area**    | core                          |
| **Issue**   | none                          |
| **PR**      | #16                           |
| **Created** | 2026-09-30                    |
| **Updated** | 2026-10-01                    |

> **Changed after acceptance (2026-10-01):** after a review of the whole command surface, the
> `[trees.landed]` setting was dropped: both landing rules (pull request merged, pushed bookmark
> deleted) always apply. The `lru` lifetime and `[trees.lru] max` stay. Trees no longer store a role
> (schema v4): the base tree is the clone's own workspace, `default`, and every other recorded tree
> is a task tree. The text below is the design as accepted.

## Q1. What are you trying to do?

Fix the layout under mori's root and the model behind it before any command beyond `mori init`
builds on them. This doc says where clones and trees live, what a tree is (its role, owner, name
and lifetime), and what mori's database records about each one. The rule behind all of it: **the
VCS is the source of truth; the database is only mori's operational state.**

## Q2. What problems is this not trying to solve?

- **More than one developer.** The model is one developer plus agents as leverage. Several people
  sharing a forest may come later; nothing here should block it, and nothing here designs for it.
- **The context half of the root** (`skills/`, `projects/`, `llms.txt`). This doc reserves the
  names and nothing else. A later profile could let people shape that half (group by project,
  by document or artifact type).
- **Importing what mori didn't create.** mori only pays attention to the clones and trees it
  created itself; it never imports, loads or adopts others. A workspace with no mori record is
  foreign: `mori ls` lists it, and nothing ever adopts, imports or cleans it up. Adopting foreign
  trees may come later.
- **Cleanup, handoffs and restore.** Lifetimes here only make a tree a *candidate* for removal;
  how removal is reported, confirmed, journalled and undone is its own design.
- **Commands and output.** `mori ls`, `mori clone` and `mori tree create` get their own docs or
  CLs; this doc gives them the model.

## Q3. How is it done today, and what are the limits?

By hand, or by tools that each own a slice. People keep clones in a `ghq`-style tree
(`<root>/<host>/<owner>/<repo>`), let every agent tool create worktrees wherever it likes, and
keep who-owns-what in their heads or in git lock reasons. The limits:

- Nothing knows which tree belongs to which agent or task, or whether it can go.
- Trees pile up in tool-specific places (`/tmp`, `.claude/worktrees`, …) with no common view.
- "Is this merged?" is answered with `git branch --merged`, which is wrong for squash merges: the
  squashed commit on trunk is never an ancestor of the branch.
- A tool that keeps its own copy of VCS facts (current branch, HEAD, dirty) drifts from the repo
  the moment someone runs git or jj directly.

## Q4. What is new in your approach, and why will it work?

mori acts like one member of a swarm working against the remote: it **reads** the truth from the
VCS and the remote every time, and **writes down** only what the VCS can't hold: who made a tree,
for which task, under which policy, and when it was last used. Branches, bookmarks, HEAD, dirty
and pushed state are computed on demand and never cached as fact. If the database and the VCS
disagree, the VCS wins and mori reports the difference instead of failing.

On top of that rule:

1. **A fixed, two-half root.** A managed half (`repos/`, `trees/`) with a stable layout that
   commands rely on, and a context half for people and LLMs with no stability guarantee.
2. **Trees get stable IDs.** Each tree row has an opaque ID that never changes or gets reused,
   joined to the VCS by its workspace name.
3. **Policy, not prescription.** Trees exist for different reasons. Names and lifetimes come from
   defaults in `config.toml` that a flag can override per tree; mori enables workflows and doesn't
   impose one.
4. **"Done" means landed on the remote.** A task tree is done when the remote says its work
   landed (the PR merged, or the pushed bookmark was deleted), not when a commit is an ancestor of
   trunk, which squash merges never make true.

## Q5. Who cares? If it works, what difference does it make?

- **You**, looking at your machine: one command (CUJ 2, `mori ls`) shows every tree, whose it is,
  what it's for and whether it can go, including trees mori didn't make.
- **Agents**: they get a tree with a predictable name and path (CUJ 3) and remove only their own
  (CUJ 4), without mori guessing at state the VCS already knows.
- **Cleanup** (CUJ 6) gets a sound basis: candidates come from policy plus remote facts, and every
  mismatch is visible rather than silently "fixed".

## Q6. What are the risks?

- **Remote facts can be missing.** Offline, or with no GitHub token, mori can't tell a merged PR
  from an open one. Then the tree is simply *not* a candidate: unknown never means done.
- **A deleted bookmark isn't always a landing.** Someone may delete a bookmark to abandon work.
  Treating it as done only makes the tree a candidate; cleanup still refuses to drop unpushed
  changes. Policies can require the merged PR alone.
- **Short repo names can clash** under `trees/<repo>/`: two owners can both have a `widget`.
  mori detects the clash at `mori clone` time and uses `<owner>-<repo>` for the second one.
- **Lock-in to names.** The name template is configurable, so tree paths aren't an API. Commands
  and agents address trees by ID or name through mori, never by guessing a path.

## Q7. How long will it take?

About six CLs after this doc, in order:

1. store: schema v2 (`repos`, `trees`), with the migration from v1.
2. core: tree names from a template, lifetime policies from `config.toml`, pure and tested.
3. jj backend: list workspaces and join them to tree rows; report missing and foreign trees.
4. cli: `mori clone` (CUJ 1): the VCS clone, then a best-effort record of the repo and its base
   tree. It comes first because mori only shows what it created, so `ls` needs something to show.
5. cli: `mori ls` (CUJ 2), read-only, text and `--json`.
6. cli: `mori tree create` / `mori tree remove` (CUJs 3 and 4), with the landing check.

## Q8. How will we know it worked?

- **Mid-term:** schema v2 merged, and the `mori ls` scenarios in Appendix E pass, including a
  foreign workspace and a row whose workspace is gone.
- **Final:** mori manages its own repo: clone it, create a task tree, land a squash-merged PR, and
  see that tree reported as done, end to end.

## Appendix A. API (proto) changes

None in this doc. CUJ 2 adds a `Tree` message to `mori.v1alpha1` carrying the fields in
Appendix B (ID, repo, name, path, role, owner, task, lifetime, and the live VCS state computed at
read time), plus reasons for the mismatches: `TREE_MISSING` and `TREE_FOREIGN`. Both are reported
as statuses on the tree, not returned as errors.

## Appendix B. Design sketch

### The root

```text
$MORI_ROOT/                              default ~/mori
  repos/<host>/<owner>/<repo>/           the clone; also its base tree       managed, stable
  trees/<repo>/<name>/                   every other tree of that repo       managed, stable
  skills/                                reserved                            context, may move
  projects/                              reserved                            context, may move
  llms.txt                               reserved                            context, may move
```

- **Managed half.** mori creates, lists and removes things here, and commands rely on its shape.
  Changing it needs a migration.
- **Context half.** For people and LLMs to read: skills, project notes, an index. mori reserves the
  names so nothing else lands there, but promises no layout and may move it.
- Trees other tools made elsewhere (`/tmp`, `.claude/worktrees`, …) stay where they are. mori lists
  them as foreign when the VCS reports them, and never acts on them.

### A tree

A tree is one jj workspace (or git worktree) of a clone. It has:

| Field                              | Where it comes from      | Example                                 |
| ---------------------------------- | ------------------------ | --------------------------------------- |
| **ID**                             | database                 | `tree_01JB7Q…` (opaque, never reused)   |
| **Repo**                           | database                 | `github.com/acme/widget`                |
| **Name**                           | database                 | `claude-fix-login`                      |
| **Path**                           | derived                  | `trees/widget/claude-fix-login`         |
| **Role**                           | database                 | `base`, `task`                          |
| **Owner**                          | database                 | `you`, or an agent such as `claude`     |
| **Task**                           | database                 | `fix-login`                             |
| **Lifetime**                       | database                 | `pinned`, `task-done`, `ttl 14d`, `lru` |
| **Used at**                        | database                 | the last time mori saw it used          |
| **HEAD, bookmarks, dirty, pushed** | the VCS, at read time    | never stored                            |
| **Landed?**                        | the remote, at read time | PR merged, or pushed bookmark deleted   |

**Roles** say who a tree belongs to, not how to work in it. The base tree is the clone itself and
is yours. Every tree mori creates is a task tree: one piece of work, usually an agent's. A foreign
tree is one mori didn't make; it has no row, only a report line.

**Lead trees are a convention, not a role.** A lead tree is a long-lived coordinating tree: it holds
a coordinating branch (or bookmark) that other trees' work gets merged and staged into before it
goes up for review. That is useful with git and jj alike. mori supports it without a special role:
a lead is a task tree with a pinned lifetime and a name that says so, such as `{owner}-lead`. How to
run one (what to merge, when to restack, when to submit) is a skill in `skills/`, not something the
database tracks.

**Names** come from a template in `config.toml`; the default is `{owner}-{task}`. The workspace
name in the VCS is the tree's name, which is what joins a row to its workspace.

### Lifetimes

A lifetime only makes a tree a candidate for cleanup; cleanup still runs every safety check.

| Policy        | Candidate when                                                            |
| ------------- | ------------------------------------------------------------------------- |
| **pinned**    | never                                                                     |
| **task-done** | the remote says its work landed (see below)                               |
| **ttl**       | it hasn't been used for the given time                                    |
| **lru**       | its repo is over a configured cap and it is among the least recently used |

The base tree is always pinned and has no setting, so no policy can make your clone a cleanup
candidate. For task trees, `config.toml` sets the default and `mori tree create --lifetime`
overrides it for one tree:

```toml
[trees]
name = "{owner}-{task}"

[trees.lifetime]
task = "task-done"

[trees.landed]
# Any of these means landed. Ancestry is never used: squash merges break it.
when = ["pr-merged", "pushed-bookmark-deleted"]
```

### What the database stores

Schema v2 adds two tables beside `meta`. Everything in them is something the VCS can't tell mori.

```sql
CREATE TABLE repos (
    id         TEXT PRIMARY KEY NOT NULL,   -- opaque, stable
    remote     TEXT NOT NULL UNIQUE,        -- github.com/acme/widget
    dir_name   TEXT NOT NULL UNIQUE,        -- the <repo> under trees/
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE trees (
    id           TEXT PRIMARY KEY NOT NULL, -- opaque, stable, never reused
    repo_id      TEXT NOT NULL REFERENCES repos (id),
    name         TEXT NOT NULL,             -- also the workspace name
    role         TEXT NOT NULL,             -- base | task
    owner        TEXT NOT NULL,
    task         TEXT,
    lifetime     TEXT NOT NULL,             -- pinned | task-done | ttl:<duration> | lru
    created_at   TEXT NOT NULL,
    last_used_at TEXT NOT NULL,
    UNIQUE (repo_id, name)
) STRICT;
```

### Joining rows to the VCS

Every read lists the clone's workspaces and matches them to rows by name:

| Row | Workspace | Reported as | mori does                                              |
| --- | --------- | ----------- | ------------------------------------------------------ |
| yes | yes       | the tree    | shows it with live VCS state                           |
| yes | no        | missing     | shows it; never recreates it; cleanup can drop the row |
| no  | yes       | foreign     | shows it; never adopts, imports or touches it          |

A mismatch is never fatal: the command finishes and the report says what didn't match.

## Appendix C. Rejected designs

- **Cache VCS state in the database** (current branch, HEAD, dirty). Fast to read, wrong as soon
  as anyone runs git or jj directly. The VCS is cheap enough to ask.
- **Detect landing by ancestry** (`git branch --merged`). Squash merges put a new commit on trunk,
  so a landed branch never looks merged.
- **Key trees by path.** Paths change when a template or the root changes; an opaque ID doesn't.
- **Trees grouped by agent** (`trees/<agent>/<repo>/<task>`). It encodes one workflow in the
  layout. Grouping by repo keeps a repo's trees together, and the owner lives in the name template,
  which people can change.
- **A built-in lead role** with its own place (`repos/<repo>-lead`) and database tracking. A pinned
  task tree plus a skill gives the same coordinating tree without fixing one way to use it.
- **Store owner and task only in git lock reasons.** jj workspaces have no lock reason, and one
  opaque string can't hold a lifetime and timestamps.

## Appendix D. Failure modes and security

- **Interrupted create.** mori writes the row after the workspace exists. A crash in between
  leaves a foreign workspace, which is reported, not lost.
- **Interrupted remove.** mori forgets the workspace before deleting the row. A crash in between
  leaves a missing row, which is reported and can be dropped.
- **Concurrent runs.** SQLite serializes writes; the `(repo_id, name)` key stops two sessions from
  claiming the same name.
- **Hostile input.** Names from templates are checked (lowercase letters, digits and hyphens) so a
  task slug can't escape `trees/<repo>/`. Remote data (PR state) only ever makes a tree a
  candidate; it never deletes anything by itself.

## Appendix E. Test plan

CUJs 2, 3 and 4 get these scenarios first; unit tests cover naming and lifetime rules in core, and
integration tests run against throwaway jj repos.

```gherkin
Scenario: A workspace mori didn't make is listed as foreign
  Given mori manages "github.com/acme/widget"
  And someone ran "jj workspace add" in its clone outside mori
  When I run "mori ls"
  Then it succeeds
  And the workspace is listed as foreign

Scenario: A tree whose workspace is gone is listed as missing
  Given mori created a task tree "claude-fix-login"
  And its workspace was forgotten outside mori
  When I run "mori ls"
  Then it succeeds
  And "claude-fix-login" is listed as missing

Scenario: A squash-merged task is done
  Given a task tree whose bookmark was pushed and its PR squash-merged
  When I run "mori ls"
  Then the tree's lifetime says it is done

Scenario: The name template decides the tree's path
  Given the name template is "{task}"
  When an agent runs "mori tree create --repo github.com/acme/widget --task fix-login"
  Then the tree is at "trees/widget/fix-login"
```

## Appendix F. Migration

- **Database:** schema v1 → v2 adds `repos` and `trees`; no existing data changes. Opening a v1
  database migrates it in one transaction.
- **Config:** the new `[trees]` sections are optional; missing keys use the defaults above.
- **Disk:** none. `mori init` already creates `repos/` and `trees/`; the context names are only
  reserved.

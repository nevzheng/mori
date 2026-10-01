# Cleanup that never loses work: a report, confirmed batches, a journal and restore

|             |            |
| ----------- | ---------- |
| **Author**  | @nevzheng  |
| **Status**  | accepted   |
| **Area**    | core       |
| **Issue**   | none       |
| **PR**      | this PR    |
| **Created** | 2026-09-30 |
| **Updated** | 2026-10-01 |

> **Changed after acceptance (2026-10-01):** a review of the whole command surface found no need
> for the `lru` lifetime or the `[trees.lru]` and `[trees.landed]` settings, so they were dropped:
> lifetimes are `pinned`, `task-done` and `ttl`, and both landing rules (pull request merged, pushed
> bookmark deleted) always apply. The text below is the design as accepted.

## Q1. What are you trying to do?

Clear away the task trees that have done their job, safely and in bulk. `mori gc` reports which
trees their lifetime says may go and whether each is safe to remove. The person confirms a batch;
`mori gc apply` checks each tree again and removes it; every removal goes into a journal; and
`mori restore` puts any removed tree back. The promise: every tree dies, and no work does.

## Q2. What problems is this not trying to solve?

- **Bookmarks and branches.** Deleting merged or stale bookmarks is its own design; gc only
  removes trees.
- **Foreign trees and other tools' worktrees.** mori only manages what it created; gc lists
  foreign workspaces for information and never acts on them.
- **Running unattended.** No daemon or schedule; gc runs when someone runs it, and removal always
  needs a confirmation.
- **The base tree.** The clone itself is pinned and never a candidate.

## Q3. How is it done today, and what are the limits?

`mori tree remove` (CUJ 4) removes one tree at a time, when its owner asks and its work is pushed.
Nothing clears the trees nobody removes: agents that crashed, forgot, or finished work that has
since landed. Lifetimes (`task-done`, `ttl`, `lru`) are recorded on every tree but nothing reads
them yet. By hand, cleanup means checking each tree for unpushed work, which is slow and easy to
get wrong.

## Q4. What is new in your approach, and why will it work?

Two separate questions per tree, answered from the VCS and the remote, never from mori's memory:

1. **May it go?** Its lifetime says so: `task-done` and the work landed (the PR merged, or a
   bookmark pushed from the tree was deleted on the remote); `ttl` and nothing changed in it for
   that long; `lru` and its repo is over the configured cap.
2. **Is it safe to go?** No edits, after a fresh snapshot, and every change in the tree is either
   on the remote or part of work that landed. `mori tree remove` uses the same rule.

"On the remote" alone isn't enough. In the usual flow an agent pushes a bookmark, the pull request
is squash-merged, and the remote deletes the bookmark: trunk gets a new, squashed commit, and the
tree's own commits are on no remote bookmark any more. Their work is safe, but "pushed" can no
longer see it, and after a fetch jj forgets the deleted bookmark entirely. So mori **records the
bookmarks it sees pushed from each tree**, with the commit each pointed to. That is operational
state the VCS can't keep, which is what mori's database is for. A tree's changes that are
ancestors of a landed bookmark's last-seen commit count as saved; anything newer still blocks.

Only trees that pass both are removed, and only in a batch the person confirmed. Each removal
records what is needed to undo it: the tree's record and the commit its working copy was on, and
**pins** that commit so it can't be garbage-collected. Pushed doesn't mean kept: after a squash
merge the remote may delete the bookmark, leaving the tree's own commits only in the local clone,
where jj's or git's garbage collection could drop them. The pin is a git ref in a namespace mori
reserves (`refs/mori/removed/<entry>`), outside jj's bookmarks, so it is never shown or pushed
but keeps the commit alive.

## Q5. Who cares? If it works, what difference does it make?

- **The person**: one command shows every tree that can go and why; one confirmation clears them;
  a mistake is one `mori restore` away.
- **Agents**: they can leave finished trees behind without harm; they never need to be trusted to
  clean up after themselves.
- **CUJs 6 and 7** (clean up safely, undo a removal) are done.

## Q6. What are the risks?

- **Removing a tree someone is using.** A `ttl` tree may be idle but still wanted. Mitigated:
  removal needs confirmation, the report says why each tree is a candidate, the safety check
  snapshots first, and restore brings it back.
- **Remote facts are slow or missing.** Without network or GitHub access, "landed" is unknown and
  a `task-done` tree simply isn't a candidate. Unknown never means yes.
- **A deleted bookmark that wasn't a landing** (someone abandoned the work). With the default
  `[trees.landed]` it counts as landed, so the tree's changes up to that commit count as saved
  although no pull request merged them. They stay pinned and restorable after removal; people who
  want only merged PRs to count set `when = ["pr-merged"]`.
- **The journal is the only record of removed trees.** It is append-only, in mori's state
  directory, and each entry is complete on its own.
- **A pinned commit can still disappear** if someone deletes the ref or the clone. Restore then
  says the commit is gone and stops; it never guesses or recreates a different state.
- **Pins accumulate.** One small ref per removal; pruning old pins (with the journal entries they
  back) is a later, confirmed cleanup.

## Q7. How long will it take?

About nine CLs after this doc:

1. core: the candidate rules (lifetime and landing facts to "may go") and the report classes.
2. store: schema v3, the pushed bookmarks recorded per tree.
3. jj: `jj git fetch`, a tree's last-change time, the remote bookmarks pointing into a tree, and
   whether its changes are covered by given commits.
4. github: whether a bookmark's pull request merged, through the `gh` CLI.
5. core + cli: the landed-aware safety rule, used by `mori tree remove` too.
6. store: saved reports and the append-only journal; jj/git: pin a commit under
   `refs/mori/removed/` and make it visible to jj again.
7. cli: `mori gc` (the report).
8. cli: `mori gc apply` (confirmed batches, re-check, journal).
9. cli: `mori restore` (through the pin), plus the skill sections.

## Q8. How will we know it worked?

- **Mid-term:** `mori gc` lists a squash-merged task tree as ready to remove and a tree with
  unpushed work as blocked, without changing anything.
- **Final:** the Appendix E scenarios pass: a batch is removed, each removal is journalled, and
  `mori restore` brings a removed tree back with its record and its change.

## Appendix A. API (proto) changes

- `Gc` RPC: request `{repo, offline}`; response `{report_id, items}`, each item a tree with its
  class (`CLASS_REMOVE`, `CLASS_BLOCKED`, `CLASS_KEEP`, `CLASS_NEVER`), the reason code and the
  facts behind it.
- `GcApply` RPC: request `{report_id, names, max, validate_only}`; response the items acted on,
  each with its outcome (`REMOVED`, `SKIPPED_CHANGED`, `SKIPPED_UNSAVED`) and journal entry ID.
- `Restore` RPC: request `{entry_id}`; response the restored `Tree`.

## Appendix B. Design sketch

### Classes and reasons (first match wins)

| Class     | Reason                         | When                                                                 |
| --------- | ------------------------------ | -------------------------------------------------------------------- |
| `never`   | `BASE`                         | the clone itself                                                     |
| `never`   | `FOREIGN`                      | a workspace mori didn't make (listed, never acted on)                |
| `never`   | `PINNED`                       | lifetime `pinned`                                                    |
| `remove`  | `MISSING`                      | recorded, but the workspace is gone: only the record is dropped      |
| `keep`    | `NOT_YET`                      | its lifetime doesn't make it a candidate (not landed, too recent, …) |
| `keep`    | `UNKNOWN`                      | `task-done`, but landing can't be checked (offline, no `gh`)         |
| `blocked` | `UNSAVED`                      | a candidate, but it has edits or unpushed changes                    |
| `remove`  | `LANDED` / `IDLE` / `OVER_CAP` | a candidate, and nothing in it exists only on this machine           |

### The flow

```text
mori gc [repo] [--offline]
  fetch each repo (unless --offline)            the only thing a report changes: remote refs
  for each recorded tree: lifetime facts + state (as of jj's last snapshot)
  classify, save the report (state dir, reports/<id>.json), print it

mori gc apply <report-id> [--only <names>] [--max 10] [--dry-run]
  show the batch, ask for yes (or --yes)
  for each item: snapshot, re-check every fact; skip it if anything changed
                 pin the working-copy commit: git ref refs/mori/removed/<entry>
                 forget the workspace, delete the directory, drop the record
                 append a journal entry
  stop at the first unexpected error

mori restore <entry-id>
  if the pinned commit is gone: say so and stop
  make the pin visible to jj through a temporary bookmark (git import), then drop the bookmark
  jj workspace add at the recorded path, on the recorded commit
  record the tree again with its old ID, owner, task and lifetime
  the pin stays until pins are pruned
```

### What mori records about pushed bookmarks

Schema v3 adds one table. Whenever `mori ls`, `mori gc` or `mori tree remove` looks at a tree, it
notes every remote bookmark pointing into the tree's own history (its changes not in trunk),
before any fetch:

```sql
CREATE TABLE tree_bookmarks (
    tree_id    TEXT NOT NULL REFERENCES trees (id),
    bookmark   TEXT NOT NULL,   -- e.g. claude/fix-login
    remote     TEXT NOT NULL,   -- e.g. origin
    commit_id  TEXT NOT NULL,   -- where it pointed when last seen
    seen_at    TEXT NOT NULL,
    PRIMARY KEY (tree_id, remote, bookmark)
) STRICT;
```

A row is updated when the bookmark moves and kept when it disappears: its disappearance is the
fact that matters.

### Landing facts

- **`pr-merged`**: for a bookmark recorded for the tree, `gh pr view <bookmark> --json state` says
  `MERGED`. Uses the person's existing `gh` login; without `gh`, this fact is unknown.
- **`pushed-bookmark-deleted`**: a bookmark recorded for the tree is gone from the remote after
  the fetch. After a fetch jj no longer shows it at all, which is why mori records it first.

A bookmark that landed either way is a **landed bookmark**; its recorded `commit_id` marks the
work that landed.

### Safe to remove (gc and `mori tree remove`)

After a fresh snapshot, a tree is safe when its working copy has no edits and each of its
non-empty changes not in trunk is either reachable from a remote bookmark or an ancestor of a
landed bookmark's recorded commit. The first rule alone is the earlier "pushed = safe"; the
second makes a squash-merged tree removable without trusting anything but the recorded commit
and the remote's answer.

- **`ttl`**: the time of the latest change in the tree (the working-copy commit's timestamp), from
  jj. The database's `last_used_at` isn't used; the VCS is the truth.
- **`lru`**: `[trees.lru] max = <n>` per repo in `config.toml`; the least recently changed task
  trees beyond `n` are candidates.

### A journal entry

One JSON line per removal in `$XDG_STATE_HOME/mori/journal.jsonl`: entry ID, time, report ID, repo,
the tree's full record (ID, name, path, owner, task, lifetime), the working-copy commit ID and
change ID, the pin ref, the remote bookmarks it was reachable from, and the restore command.

## Appendix C. Rejected designs

- **Remove without confirmation when everything is pushed.** Safe for work, but a tree can still
  be wanted. Confirmation is cheap; surprise is not.
- **Ancestry to detect landing.** Squash merges break it, as the forest design says.
- **A GitHub token in mori's config.** `gh` already holds the person's login; mori doesn't need to
  store a secret.
- **Pin with a jj bookmark** (e.g. `mori/removed/<entry>`). It would show in `jj log` and could
  be pushed by `jj git push --all`; a git ref outside `refs/heads` does neither.
- **Rely on the remote to keep the commit.** After a squash merge the remote keeps only the
  squashed commit, not the tree's own.
- **Keep removed trees in a trash directory.** Doubles disk use for work that is already on the
  remote; the journal plus the commit is enough to restore.

## Appendix D. Failure modes and security

- **Interrupted apply:** each removal is pin → forget → delete → drop record → journal. A crash
  before the journal line leaves a missing tree (shown by `mori ls`) that a later gc drops, and a
  pin with no entry, which pruning later removes; the commit stays pinned meanwhile.
- **Facts change between report and apply:** every fact is checked again right before each
  removal; changed items are skipped and say why.
- **Concurrent gc runs:** SQLite serializes the record changes; a tree already removed is skipped.
- **Restore into an occupied path:** refused, like `tree create`.
- **Hostile remote data:** remote facts only make a tree a candidate; the local safety check
  decides whether it goes.

## Appendix E. Test plan

```gherkin
Scenario: The report sorts trees without changing them
  Given a task tree whose pushed bookmark was deleted on the remote
  And a task tree with unpushed work
  And a pinned tree
  When I run "mori gc"
  Then the first is listed to remove because it landed
  And the second is blocked because of unsaved work
  And the pinned tree is never removed
  And no tree changed

Scenario: A squash-merged tree is safe to remove
  Given a task tree whose bookmark was pushed, squash-merged and deleted on the remote
  When I run "mori tree remove <repo> <tree> --agent claude"
  Then it succeeds

Scenario: Work newer than what landed still blocks
  Given a task tree whose bookmark landed
  And a new commit in the tree after it
  When I run "mori gc"
  Then the tree is blocked because of unsaved work

Scenario: A confirmed batch is removed and journalled
  Given a report with two trees to remove
  When I run "mori gc apply <report> --yes"
  Then both trees are gone
  And the journal has an entry for each

Scenario: A tree that changed since the report is skipped
  Given a report listing a tree to remove
  And someone edits that tree
  When I run "mori gc apply <report> --yes"
  Then the tree is kept and the output says it changed

Scenario: Restore brings a removed tree back, even after its bookmark is gone
  Given a tree removed by "mori gc apply"
  And its pushed bookmark was deleted on the remote and fetched
  When I run "mori restore <entry>"
  Then the tree is back at its path, on its change, with its record

Scenario: Restore says so when the commit is gone
  Given a tree removed by "mori gc apply"
  And its pin was deleted and the repo garbage-collected
  When I run "mori restore <entry>"
  Then it fails with reason "RESTORE_COMMIT_GONE" and changes nothing
```

## Appendix F. Migration

- **Database:** schema v2 → v3 adds `tree_bookmarks`; opening an older database upgrades it in
  one transaction, as before. Trees made before the upgrade have no recorded bookmarks until mori
  next looks at them, so a tree whose bookmark was already deleted by then can't be shown as
  landed; it stays blocked, which is the safe side.
- **Files:** the journal and reports are new files in the state directory, and pins are new refs
  under `refs/mori/removed/` in each clone. The unused `last_used_at` column stays for now.
- **`mori tree remove`** now also accepts a squash-merged tree; nothing it accepted before is
  refused.

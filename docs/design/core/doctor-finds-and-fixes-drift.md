# Doctor: find drift between mori and the VCS, and fix only what is safe

|             |            |
| ----------- | ---------- |
| **Author**  | @nevzheng  |
| **Status**  | review     |
| **Area**    | core       |
| **Issue**   | none       |
| **PR**      | this PR    |
| **Created** | 2026-10-01 |
| **Updated** | 2026-10-01 |

## Q1. What are you trying to do?

Give people and agents one command that answers "is this root healthy, and if not, what do I do?".
`mori doctor` checks the root, every clone and every tree against what jj reports, and lists each
problem with a stable code and the command that fixes it. `mori doctor --fix --yes` repairs the
problems that are safe to repair automatically, and only in what mori created. Anything it removes
is pinned and journalled, so `mori restore` brings it back.

## Q2. What problems is this not trying to solve?

- **Adopting foreign trees.** A workspace or directory mori didn't create is reported, never
  imported, moved or removed. Adoption stays on the wishlist.
- **Choosing for you.** Problems that need a judgement, such as a conflicted bookmark or a
  diverged change, get the commands to resolve them, not an automatic pick.
- **Lifetime cleanup.** Removing trees whose job is done is `mori gc`. Doctor is about integrity:
  records, workspaces and directories that disagree.
- **Supporting a jj version.** Doctor prints the jj version it found; it doesn't enforce one.
- **Running in the background.** Doctor runs when someone runs it.

## Q3. How is it done today, and what are the limits?

`mori ls` shows a recorded tree with no workspace as missing and a workspace mori didn't create as
foreign, and `mori gc --apply` drops missing records. Everything else is found by hand:

- A tree directory deleted with `rm -rf` leaves a jj workspace that every jj command in the clone
  then trips over, and a record that `ls` can't read state for.
- A working copy goes stale when another workspace rewrites its commit, a routine event with
  several agents in one clone. jj refuses to work there until someone runs
  `jj workspace update-stale`, and agents rarely know to.
- A conflicted bookmark (`main??`) after a fetch blocks pushing and confuses agents.
- Leftover directories under `trees/`, clones moved or deleted by hand, and stale indexes in
  `context/` go unnoticed.

An agent hitting any of these gets a jj error with no next step. A person has to know jj well to
recover.

## Q4. What is new in your approach, and why will it work?

One read-only pass collects facts, which the pure core judges into **findings**: a stable
`UPPER_SNAKE_CASE` code, a severity (`ok`, `info`, `warn`, `problem`), the subject (root, repo or
tree), a sentence saying what is wrong, and the command that fixes it. Each finding is marked
**auto-fixable** only if all three hold:

1. it concerns only what mori created or mori's own records;
2. the fix can't lose work, or whatever it removes is pinned and journalled first; and
3. there is exactly one right fix, with no choice to make.

`--fix` applies those, re-checking each one right before acting, as `gc --apply` does. Everything
else is reported with its commands. The same facts and journal code as cleanup are reused, so
doctor adds checks, not a second mechanism.

## Q5. Who cares? If it works, what difference does it make?

Agents most. A coordinator can run `mori doctor --json` before dispatching work, or when a jj
command fails, and either fix it or hand the person one clear line. People get a single place to
look after something odd happened, instead of knowing jj internals. It also makes the "VCS is the
truth" rule visible: drift is surfaced, never silently patched over by other commands.

## Q6. What are the risks?

- **Forgetting a workspace that was moved, not deleted.** `TREE_DIR_GONE` only fires when nothing
  exists at the recorded path, and the fix pins the working-copy commit and journals it first, so
  `mori restore` brings the tree back.
- **`update-stale` surprising someone mid-work.** jj keeps any edits as a recovery change rather
  than discarding them; doctor only runs it in trees mori created, and says what it did.
- **Overlap with gc.** Both can drop a missing record. They share one action and one journal
  entry type, so the result is the same whichever runs first.
- **Check sprawl.** Each check is a small pure function with its own code and scenario; new ones
  are added only for drift seen in practice.

## Q7. How long will it take?

About 4 CLs, in order:

1. Core: the finding model and the pure checks, with unit tests for every code.
2. jj: the extra facts (stale working copies, conflicted bookmarks, workspace roots on disk).
3. `mori doctor [repo] [--json]`: the report, proto, CUJ scenarios and the using-mori skill.
4. `mori doctor --fix --yes [--dry-run]`: the safe repairs, pinned and journalled, with scenarios.

## Q8. How will we know it worked?

Mid-term: CL 3 merged, and a root with a hand-deleted tree reports `TREE_DIR_GONE` with its fix.
Final: every scenario in Appendix E passes, and an agent can recover from a deleted tree, a stale
working copy and a missing context folder using only `mori doctor` output.

## Appendix A. API (proto) changes

New, nothing removed or changed:

- `rpc Doctor(DoctorRequest) returns (DoctorResponse)`.
- `DoctorRequest { string repo; bool fix; bool validate_only; }`.
- `DoctorResponse { repeated Finding findings; repeated Fixed fixed; string jj_version; }`.
- `Finding { string code; Severity severity; string subject; string message; string fix; bool
  auto_fixable; }`, with `Severity` as `OK | INFO | WARN | PROBLEM`.
- `Fixed { string code; string subject; string journal_entry; }`; `journal_entry` is empty when
  nothing was removed.
- Error reason `PROBLEMS_FOUND` (canonical code `FAILED_PRECONDITION`) when any `problem` finding
  remains after the run, so scripts can branch on the exit code. `--fix` without `--yes` is
  `CONFIRMATION_NEEDED`, as in gc.

## Appendix B. Design sketch

The checks, by subject. "Fix" names what `--fix` does; "report" means the finding carries commands
for a person or agent to run.

| Code                     | Severity | Subject | What it means                                       | Fix    |
| ------------------------ | -------- | ------- | --------------------------------------------------- | ------ |
| `JJ_NOT_FOUND`           | problem  | root    | `jj` isn't on the path                              | report |
| `DATABASE_UNREADABLE`    | problem  | root    | the database can't be opened or is a newer schema   | report |
| `CLONE_MISSING`          | problem  | repo    | a recorded clone's directory is gone                | report |
| `CLONE_NOT_RECORDED`     | info     | repo    | a jj repo under `repos/` that mori has no record of | report |
| `TREE_DIR_GONE`          | problem  | tree    | the workspace exists, its directory doesn't         | fix    |
| `WORKSPACE_GONE`         | warn     | tree    | the record exists, jj has no such workspace         | fix    |
| `STALE_WORKING_COPY`     | warn     | tree    | jj needs `workspace update-stale` there             | fix    |
| `DIR_WITHOUT_WORKSPACE`  | warn     | path    | a directory under `trees/` no workspace uses        | report |
| `FOREIGN_WORKSPACE`      | info     | tree    | a workspace mori didn't create                      | report |
| `CONFLICTED_BOOKMARK`    | problem  | repo    | a bookmark has several targets (`main??`)           | report |
| `ORPHAN_BOOKMARK_ROWS`   | warn     | repo    | recorded pushed bookmarks for trees that are gone   | fix    |
| `CONTEXT_FOLDER_MISSING` | warn     | repo    | a clone has no `context/projects/<repo>/`           | fix    |
| `CONTEXT_INDEX_STALE`    | warn     | root    | a generated `llms.txt` differs from what it'd be    | fix    |

The fixes:

- `TREE_DIR_GONE`: pin the working-copy commit under `refs/mori/removed/<entry>`, forget the
  workspace, drop the record, journal it. `mori restore <entry>` recreates it.
- `WORKSPACE_GONE`: drop the record and journal it (the same action gc uses for `MISSING`).
- `STALE_WORKING_COPY`: run `jj workspace update-stale` in the tree.
- `ORPHAN_BOOKMARK_ROWS`: delete the rows. They are mori's own state about a tree that no longer
  exists.
- `CONTEXT_FOLDER_MISSING` and `CONTEXT_INDEX_STALE`: what `mori skills sync` already does. A
  context file a person edited is never overwritten.

Output groups findings by subject, worst first, and ends with one line: `3 problems, 2 can be fixed
with mori doctor --fix --yes`. `--json` gives the response message.

Reads use `--ignore-working-copy`, as everywhere else in mori, so a doctor run never snapshots or
disturbs an agent's tree. The one exception is the stale check, which asks jj whether each mori
tree's working copy is stale without changing it.

## Appendix C. Rejected designs

- **Self-healing in every command.** Silently fixing drift whenever `ls` or `tree create` notices
  it hides problems and breaks "the VCS is the truth": the person never learns what happened.
- **Folding doctor into gc.** gc is lifetime policy and asks for a batch confirmation per run;
  integrity checks should be cheap to run at any time and safe to script.
- **Fixing everything doctor finds.** Conflicted bookmarks, foreign workspaces and leftover
  directories all involve someone's choice or someone else's files. Doctor gives the commands
  instead.

## Appendix D. Failure modes and security

- **Interrupted `--fix`.** Each repair is pin → act → journal, the same order as gc. A crash after
  the pin and before the journal leaves a pinned commit and nothing lost; the next run re-checks.
- **Concurrent runs.** Each fix re-checks its finding right before acting and skips it if it
  no longer holds.
- **Paths.** A fix only touches paths under the root that mori recorded or generated. It never
  follows a symlink out of the root and never deletes a directory it didn't create.
- **Repo contents.** Bookmark names and paths from the repo are printed, never executed; the fix
  commands shown for them are quoted.

## Appendix E. Test plan

CUJ 8, "check and repair the root" (`spec/cuj/08-doctor.feature`), written first:

```gherkin
Feature: CUJ 8 - check and repair the root

  Scenario: A healthy root reports nothing to fix
    Given a root with a clone of "github.com/acme/widget" and a task tree
    When I run "mori doctor"
    Then the output says "no problems"
    And the exit code is 0

  Scenario: A tree deleted by hand is found and fixed, and can be restored
    Given a task tree "tester-fix-login" whose directory was deleted by hand
    When I run "mori doctor"
    Then the output lists "TREE_DIR_GONE" for "tester-fix-login"
    And the exit code is the code for "PROBLEMS_FOUND"
    When I run "mori doctor --fix --yes"
    Then jj no longer has the workspace "tester-fix-login"
    And the output gives a "mori restore" command
    When I run that command
    Then the tree "tester-fix-login" is back with its last change

  Scenario: Fixing needs a confirmation
    Given a task tree whose directory was deleted by hand
    When I run "mori doctor --fix"
    Then it fails with "CONFIRMATION_NEEDED"
    And nothing changed

  Scenario: A stale working copy is updated
    Given two task trees where one rewrote the other's working-copy commit
    When I run "mori doctor --fix --yes"
    Then jj works in the stale tree again

  Scenario: A foreign workspace is reported and left alone
    Given a jj workspace mori didn't create
    When I run "mori doctor --fix --yes"
    Then the output lists "FOREIGN_WORKSPACE"
    And the workspace is untouched

  Scenario: A conflicted bookmark is reported with the commands to resolve it
    Given the bookmark "main" has two targets
    When I run "mori doctor"
    Then the output lists "CONFLICTED_BOOKMARK" for "main" with the jj commands to resolve it
```

Unit tests in core: one per finding code (fires when it should, not otherwise), the
auto-fixable rule, and the ordering of output. Integration tests in `mori-jj` against throwaway
repos: detecting a stale working copy, a conflicted bookmark and a missing workspace directory.

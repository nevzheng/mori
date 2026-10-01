# Git worktree backend: trees as detached git worktrees, with jj staying the default

|             |            |
| ----------- | ---------- |
| **Author**  | @nevzheng  |
| **Status**  | accepted   |
| **Area**    | core       |
| **Issue**   | none       |
| **PR**      | this PR    |
| **Created** | 2026-10-01 |
| **Updated** | 2026-10-01 |

## Q1. What are you trying to do?

Let a repo's trees be plain git worktrees instead of jj workspaces. `mori clone --vcs git` makes a
plain git clone; every tree mori creates in it is a detached `git worktree`, with a real `.git`
file, so git-based harnesses, IDEs, hooks and `gh` work inside it. Every other command (`ls`,
`tree remove`, `gc`, `restore`) behaves the same for both backends, with the same safety rules.
jj stays the default.

## Q2. What problems is this not trying to solve?

- **Mixing backends in one clone.** A clone is jj or git, decided at clone time. No git worktrees
  of a jj clone, no jj workspaces of a git clone.
- **Converting a clone.** No `mori convert`; clone again with the other backend.
- **Branch management.** Trees stay detached, as jj trees have no bookmark checked out either.
  Creating, pushing and deleting branches is the agent's or person's job, as today.
- **Other VCSs** (Mercurial, Sapling). The traits allow them; nobody is asking.
- **Requiring jj.** On a git-only machine, `--vcs git` works with no jj installed. Making jj
  optional everywhere (install docs, `mori init` checks) is follow-up polish.

## Q3. How is it done today, and what are the limits?

Every clone is a jj repo (colocated with git by default), and every task tree is a jj workspace.
A jj workspace has no `.git`, so inside a tree `git status` fails, and so do tools that shell out
to git: many agent harnesses, IDE git integrations, pre-commit hooks, `gh pr create`. People who
want git worktrees today make them by hand, outside mori, and lose its records, lifetimes, safe
removal and restore.

## Q4. What is new in your approach, and why will it work?

mori's flows already reach the VCS only through the `Vcs` trait in `mori-core` (clone, add and
forget trees, state, pushed bookmarks, pin, fetch), with the rules kept in pure functions. The git
backend is a second implementation of that trait in `crates/git`, driving the `git` CLI the same
way `mori-jj` drives `jj`. Each operation has a direct git equivalent (Appendix B); the rules,
records, journal and restore don't change.

**Which backend a clone uses is read from the clone, not stored:** a clone with a `.jj` directory
is jj, one without is git. The VCS stays the source of truth and the database needs no new
column. `mori clone` picks the backend from `--vcs`, else `[vcs] default` in `config.toml`, else
`jj`.

## Q5. Who cares? If it works, what difference does it make?

- **Agents in git-based harnesses** get a tree where git, hooks and `gh` work, with mori still
  tracking and cleaning it up.
- **People who don't use jj** can use mori for the forest layout, lifetimes and safe cleanup
  without learning jj.
- **mori itself**: the trait boundary gets proved by two real backends, not one.

## Q6. What are the risks?

- **Detached HEAD and lost commits.** A commit made on a detached worktree is on no branch. mori
  already treats "on no remote branch" as unsaved, so `tree remove` and `gc` refuse such a tree,
  and a removal pins the commit under `refs/mori/removed/<entry>` as with jj. Same guarantee.
- **"Changed" means something slightly different.** jj counts edits as of its last snapshot; git
  reads the working tree live (`git status`), untracked files included and ignored files not.
  Live is stricter, so it can only refuse more.
- **Double maintenance.** Two adapters to keep working. Mitigated by running the main e2e
  scenarios against both backends.
- **Worktree admin state.** A worktree whose directory was deleted by hand leaves an entry under
  `.git/worktrees/`. mori shows it as missing, as with a jj workspace. Removing it runs
  `git worktree remove --force <path>`, which clears that one entry even with the directory gone.
  mori never runs a bare `git worktree prune`, which would clear every stale entry, including
  worktrees mori didn't make.
- **Rebase and cherry-pick copy commits.** If a lead rebases or cherry-picks a worker's commits,
  the worker's own commits stay on no remote branch and its tree keeps reading as unsaved. Only a
  merge keeps the worker's commits. The skills say: hand over the commit SHA, and the lead runs
  `git merge <sha>`. If the lead rebases anyway, the worker runs `git switch --detach <lead tip>`
  afterwards, which leaves nothing unsaved.
- **Branches in worker trees.** git refuses to rebase or check out a branch that another worktree
  has checked out, so a worker on a branch blocks the lead. Trees start detached, and the skills
  say to `git switch --detach` before handing off.
- **Detached HEAD trips git habits.** `git push -u origin HEAD` and a bare `git pull` fail on a
  detached HEAD. The skills give `git push origin HEAD:refs/heads/<branch>` (or
  `git switch -c <branch>` first), and `gh pr create --head <branch>`.
- **Stashes hide work.** `git stash` moves edits out of the working tree into the shared
  `refs/stash`, so the safety check sees a clean tree. Stashes can't be told apart by tree, so mori
  can't count them; the skills say to commit, not stash. `mori doctor` may report stashes later.
- **Submodules** come up empty in a new worktree. The skills say to run
  `git submodule update --init`; mori doesn't run it, since it fetches.

## Q7. How long will it take?

Four CLs after this doc:

1. git: `GitCli` implementing `Workspaces` and `Vcs`, with tests against throwaway real git repos.
2. app: route each operation to jj or git by detecting the clone's backend; `forget_tree` callers
   accept that git deletes the directory itself (Appendix B).
3. cli: `mori clone --vcs jj|git` and `[vcs] default`; e2e scenarios for the git backend
   (`spec/cuj/08-git-backend.feature`) and the pinned git in the test environment.
4. skills: `vcs-in-mori` gets the git-tree workflow: the backend rule (`.jj` present means jj,
   else git, so use the matching tool), detached HEAD and pushing it, handing over a SHA for
   `git merge`, commit rather than stash, and submodules (Q6).

## Q8. How will we know it worked?

- **Mid-term:** the git adapter's tests pass: add, list, state, pin and restore a detached
  worktree in a real repo.
- **Final:** the scenarios in Appendix E pass, and the existing 80 scenarios still pass on jj.

## Appendix A. API (proto) changes

- `CloneRequest` gets `vcs` (`VCS_UNSPECIFIED` = the configured default, `VCS_JJ`, `VCS_GIT`);
  `CloneResponse` and `RepoTrees` get the `vcs` actually used.
- New error reasons: `GIT_NOT_FOUND` (FAILED_PRECONDITION) and `GIT_FAILED` (INTERNAL), domain
  `git.mori`, mirroring the jj ones.
- `--no-colocate` with `--vcs git` is refused: `INVALID_ARGUMENT`, `COLOCATE_NEEDS_JJ`.

Nothing existing breaks: an unset `vcs` means jj, as today.

## Appendix B. Design sketch

| `Vcs` operation                 | jj (today)                                      | git                                                                   |
| ------------------------------- | ----------------------------------------------- | --------------------------------------------------------------------- |
| `clone_repo`                    | `jj git clone [--colocate]`                     | `git clone <url> <path>`                                              |
| `add_tree`                      | `jj workspace add --name N -r FROM PATH`        | `git worktree add --detach PATH FROM` (`FROM` default `origin/HEAD`)  |
| `add_tree_at`                   | temporary bookmark, then `add_tree`             | `git worktree add --detach PATH COMMIT` (works on any stored commit)  |
| `list`                          | `jj workspace list`                             | `git worktree list --porcelain`; the main worktree is `default`       |
| `state.changed`                 | working-copy change non-empty                   | `git status --porcelain` non-empty                                    |
| `state.unpushed`                | changes on no remote bookmark                   | `git rev-list --count HEAD --not --remotes <landed>`                  |
| `pushed_bookmarks`              | remote bookmarks in the tree's history          | `git for-each-ref refs/remotes --merged HEAD --no-merged origin/HEAD` |
| `remote_bookmarks`              | `jj bookmark list --all-remotes`                | `git for-each-ref refs/remotes`, skipping `origin/HEAD`               |
| `last_change`                   | working-copy commit time                        | newest of HEAD's commit time and the dirty files' mtimes              |
| `snapshot`                      | `jj util snapshot`                              | nothing: git reads the working tree live                              |
| `working_copy_commit`           | the working-copy commit                         | `HEAD`; only asked for trees with no edits                            |
| `forget_tree`                   | `jj workspace forget`                           | `git worktree remove --force` (also deletes the directory)            |
| `pin`, `commit_exists`, `fetch` | `git update-ref`, `cat-file -e`, `jj git fetch` | the same through `git`; `git fetch --prune`                           |

A tree's name is its worktree's directory name, taken from the path in
`git worktree list --porcelain`. git's own name under `.git/worktrees/` usually matches but can
differ (git adds a number on a clash), so mori never uses it.

**Removal order.** The flows forget the tree and then delete its directory. `git worktree remove`
does both, so the directory delete treats "already gone" as done. The jj path is unchanged.

**Routing.** The binary builds one `Vcs` that holds both adapters and, for every call with a clone
path, asks which backend the clone is (`.jj` present: jj, else git). `clone_repo` is the one call
with no clone yet; the clone flow passes the chosen backend explicitly.

## Appendix C. Rejected designs

- **Store the backend per repo in the database.** A second source of truth that can disagree with
  the disk; detection is one `stat` and can't be wrong about what is there.
- **git worktrees of jj colocated clones.** jj and git would both move the same refs and working
  copies in one repo; jj's docs warn against it, and the safety rules would need both views.
- **gix (pure-Rust git) instead of the CLI.** Faster reads, but worktree support is partial and it
  would be a second git implementation to trust. The CLI is what people's own tools use; mori can
  move hot reads to gix later behind the same trait.
- **Branch-per-worktree** (like `git worktree add -b`). Prescribes a branch workflow and leaves
  branches to clean up; detached matches jj trees and the "only manage what mori made" rule.

## Appendix D. Failure modes and security

- **git missing:** `GIT_NOT_FOUND` before anything is created.
- **A failed `worktree add`:** whatever it left at the path is removed and
  `git worktree remove --force <path>` clears its entry, as the jj adapter forgets a half-made
  workspace. Never a bare `git worktree prune` (Q6).
- **Hooks:** git runs the repo's hooks on some operations (`post-checkout` on `worktree add`). mori
  runs git as the person, as they would; it doesn't disable hooks.
- **Paths and names** come from mori's own validated tree names and are always passed after `--`.

## Appendix E. Test plan

```gherkin
Scenario: A git clone and a git tree
  When I run "mori clone --vcs git github.com/acme/widget"
  And I run "mori tree create github.com/acme/widget --task fix-login --agent claude"
  Then "git -C <home>/mori/trees/widget/claude-fix-login status" succeeds
  And the tree is on a detached HEAD at the remote's default branch

Scenario: A git tree with a commit on no branch is kept
  Given a git tree "claude-fix-login" with a commit that is on no remote branch
  When I run "mori tree remove github.com/acme/widget claude-fix-login"
  Then it fails with status FAILED_PRECONDITION and reason "TREE_HAS_UNSAVED_WORK"

Scenario: gc removes a landed git tree and restore brings it back
  Given a git tree whose pushed branch was merged and deleted on the remote
  When I run "mori gc --apply --yes"
  Then the tree is removed and its commit is pinned
  And "mori restore <entry>" puts it back at its old path on that commit

Scenario: The default stays jj
  When I run "mori clone github.com/acme/widget"
  Then the clone is a jj repo colocated with git
```

Unit tests in `crates/git` cover every row of the table against throwaway repos; the app tests'
fake VCS covers the routing.

## Appendix F. Migration

None: existing clones have `.jj` and keep working as jj. `[vcs] default` is optional and defaults
to `jj`.

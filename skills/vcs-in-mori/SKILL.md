---
name: vcs-in-mori
description: >-
  How to use jj and git in a repo mori manages: which tool works where, what is safe to mix, and
  the jj and git commands for the same jobs. Use before running version-control commands in a mori
  clone or tree, or when unsure whether to reach for jj or git.
---

# jj and git in a mori repo

mori clones are jj repos, colocated with git by default (a `--no-colocate` clone is jj only).
Every tree mori creates is a **jj workspace**, not a git worktree. So:

- **In a task tree (`trees/<repo>/<name>/`), use jj.** It has a `.jj` folder and no `.git`: plain
  `git` commands fail there ("not a git repository"). Everything below works with jj.
- **In the clone itself (`repos/<host>/<owner>/<repo>/`), both work** when it is colocated: jj and
  git share the same commits and refs. That checkout is the person's; don't change it unless
  asked.
- **GitHub and other git tools** still see ordinary branches: jj bookmarks are git branches once
  pushed. `gh` can't find the repo from inside a tree (no `.git`), so name both:
  `gh pr create --repo <owner>/<repo> --head <bookmark>`, or run `gh` from the clone.

If a command says "not a git repository", you are in a jj workspace: use the jj column below.

## Don't

- **`git worktree add`.** It makes a tree mori didn't create (it shows as foreign and is never
  cleaned up). Use `mori tree create`.
- **`git checkout`, `git switch` or `git reset --hard` in the clone** while agents work in its
  trees. They move the clone's working copy under jj's feet, and `reset --hard` can drop
  uncommitted work. Use `jj new <revision>` in your own tree instead.
- **Rewriting history someone else is building on** without saying so: in jj that's
  `jj rebase` or `jj squash` on another tree's changes; tell the person or the lead first.

## The same jobs, jj and git

| Job                          | jj (works in any tree)                    | git (clone only, colocated)        |
| ---------------------------- | ----------------------------------------- | ---------------------------------- |
| See what changed             | `jj status`, `jj diff`                    | `git status`, `git diff`           |
| See history                  | `jj log`                                  | `git log --graph`                  |
| Finish a change, start next  | `jj commit -m "…"`                        | `git commit -am "…"`               |
| Describe the current change  | `jj describe -m "…"`                      | `git commit --amend`               |
| Name it for pushing          | `jj bookmark create <name> -r @-`         | `git switch -c <name>`             |
| Get the latest from remote   | `jj git fetch`                            | `git fetch`                        |
| Move work onto trunk         | `jj rebase -s <first> -d 'trunk()'`       | `git rebase origin/main`           |
| Push                         | `jj git push -b <name>`                   | `git push -u origin <name>`        |
| Make a patch                 | `jj diff --git -r <change> > fix.patch`   | `git format-patch -1 <commit>`     |
| Apply a patch                | `patch -p1 < fix.patch` (jj records it)   | `git am fix.patch`                 |
| Undo the last operation      | `jj undo`                                 | depends; `git reflog` to find it   |

jj tips worth knowing:

- **Change IDs** (like `kxqzlvmy`) stay the same when a change is rebased or amended; commit IDs
  don't. Share change IDs with other agents.
- **The working copy is a commit** (`@`). There is no staging area; `jj commit` finishes it and
  starts a new one.
- **Every tree sees every change** in the clone at once: `jj log -r 'all()'` shows the other
  workspaces' work (`<name>@`).
- **`jj op log` and `jj undo`** take back almost any jj operation.

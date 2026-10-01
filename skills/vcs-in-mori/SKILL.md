---
name: vcs-in-mori
description: >-
  How to use jj and git in a repo mori manages: jj clones and git clones, which tool works where,
  what is safe to mix, and the jj and git commands for the same jobs. Use before running
  version-control commands in a mori clone or tree, or when unsure whether to reach for jj or git.
---

# jj and git in a mori repo

A mori clone is one of two kinds, picked when it was cloned (`mori clone --vcs jj|git`, default
jj). Tell them apart by the clone: a `.jj` folder means jj, otherwise it is a git clone.
`mori ls --json` also says, as `vcs` on each repo.

## jj clones (the default)

jj clones are jj repos, colocated with git by default (a `--no-colocate` clone is jj only).
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

## git clones (`mori clone --vcs git`)

A plain git clone. Every tree mori creates is a **detached git worktree**: it has a `.git` file,
so git, hooks, IDEs and `gh` all work in it, and jj isn't needed.

**Which kind of tree am I in?** A `.jj` folder and no `.git` means a jj workspace: use jj ("not a
git repository" from git means this). A `.git` *file* means a git worktree: use git.

- **You start on a detached HEAD** at trunk (the remote's default branch). Commit as usual.
- **To push, name the branch in the push:** `git push origin HEAD:refs/heads/<branch>`. That
  leaves no local branch behind to clean up. (`git switch -c <branch>` then
  `git push -u origin <branch>` works too, but the branch stays in the clone.)
- **Commit, don't stash.** mori can't see a stash: a tree whose only work is stashed reads as
  clean and may be removed, and the stash goes with it.
- **Unsaved means the same as with jj:** edits (including untracked files) or a commit that is on
  no remote branch keep `mori tree remove` and `mori gc` from removing the tree. Work counts as
  saved once its branch is pushed, or once it lands (PR merged, or the pushed branch deleted
  after a squash merge).
- **Removed trees can come back** with `mori restore`, from the commit mori pinned, as with jj.

## Don't

- **`jj git init` in a git clone,** even when jj suggests it. It turns the clone into a jj clone,
  and mori stops seeing its git worktrees (they show as missing).
- **`git worktree add` (or `jj workspace add`) by hand.** It makes a tree mori didn't create (it
  shows as foreign and is never cleaned up). Use `mori tree create`.
- **`git checkout`, `git switch` or `git reset --hard` in the clone** while agents work in its
  trees. They move the clone's working copy under jj's feet, and `reset --hard` can drop
  uncommitted work. Use `jj new <revision>` in your own tree instead.
- **Rewriting history someone else is building on** without saying so: in jj that's
  `jj rebase` or `jj squash` on another tree's changes; tell the person or the lead first.

## The same jobs, jj and git

| Job                         | jj (any tree of a jj clone)             | git (git trees, or a colocated clone) |
| --------------------------- | --------------------------------------- | ------------------------------------- |
| See what changed            | `jj status`, `jj diff`                  | `git status`, `git diff`              |
| See history                 | `jj log`                                | `git log --graph`                     |
| Finish a change, start next | `jj commit -m "…"`                      | `git commit -am "…"`                  |
| Describe the current change | `jj describe -m "…"`                    | `git commit --amend`                  |
| Name it for pushing         | `jj bookmark create <name> -r @-`       | `git switch -c <name>`                |
| Get the latest from remote  | `jj git fetch`                          | `git fetch`                           |
| Move work onto trunk        | `jj rebase -s <first> -d 'trunk()'`     | `git rebase origin/main`              |
| Push                        | `jj git push -b <name>`                 | `git push -u origin <name>`           |
| Make a patch                | `jj diff --git -r <change> > fix.patch` | `git format-patch -1 <commit>`        |
| Apply a patch               | `patch -p1 < fix.patch` (jj records it) | `git am fix.patch`                    |
| Undo the last operation     | `jj undo`                               | depends; `git reflog` to find it      |

jj tips worth knowing:

- **Change IDs** (like `kxqzlvmy`) stay the same when a change is rebased or amended; commit IDs
  don't. Share change IDs with other agents.
- **The working copy is a commit** (`@`). There is no staging area; `jj commit` finishes it and
  starts a new one.
- **Every tree sees every change** in the clone at once: `jj log -r 'all()'` shows the other
  workspaces' work (`<name>@`).
- **`jj op log` and `jj undo`** take back almost any jj operation.

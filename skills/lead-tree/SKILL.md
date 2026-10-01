---
name: lead-tree
description: >-
  How to run a lead tree with mori: one long-lived tree where work from several agents is merged
  and staged before review. Use when a task asks you to coordinate other agents' work, combine or
  land their changes, or keep a stack of changes together, in a repo mori manages.
---

# Running a lead tree

A lead tree is a convention, not a special kind of tree. It is an ordinary task tree with a
pinned lifetime, so cleanup never picks it, and a name that says what it is for. Other agents work
in their own task trees; the lead tree is where their work comes together before it goes up for
review. mori records no "lead" role: how you run the tree is up to this skill and the person.

Read the `using-mori` skill first; its rules apply here too. `agent-workflows` has the fuller
recipes (workers, exchanging work, restacking).

## Make one

```sh
mori tree create github.com/acme/widget --owner claude --task lead --lifetime pinned \
  --purpose "Combine the auth work for review"
```

That gives `trees/widget/claude-lead`, on a new change on top of trunk. Make one lead tree per
coordinated effort, not one per task; if `claude-lead` exists, use a more specific task slug, such
as `lead-auth`.

## Handing work to the lead

There is no handoff command: with jj, every tree of a clone already sees every change, so handing
over is just saying which changes are ready.

**An agent that finishes work for the lead:**

1. Finishes with `jj commit -m "…"`, so every change has a description (the note the lead reads)
   and the working copy is a fresh empty change. Not `jj describe`: that leaves the work as the
   working copy, and mori keeps reading the tree as edited.
2. Stops editing its tree.
3. Tells the lead its change IDs (`jj log -r '::<its tree>@- ~ ::trunk()'`) in its final
   message, with one line on what they do.

**The lead:**

- Moves the changes into its stack with `jj rebase -s <first change> -d <where>`; the agent's
  working copy follows them. Prefer moving to copying (`jj duplicate`): copied originals are
  never pushed, so the agent's tree can never be removed. If a copy is unavoidable, abandon the
  originals afterwards (`jj abandon`) so nothing is left behind.
- The agent's tree becomes removable (`mori tree remove`) once the lead's bookmark is pushed.
  Until then mori refuses to remove it, which is right: its work isn't on the remote yet.

Across machines, push a bookmark instead.

## Work in it

With jj, every workspace of a clone sees the same changes and bookmarks at once, so you don't pass
work between trees; you point at it.

1. **See what the other trees made:** `jj log` shows every workspace's working-copy change
   (`<name>@`) and every bookmark.
2. **Stack or combine it:** rebase the changes you're landing onto each other or onto trunk,
   e.g. `jj rebase -s <change> -d <onto>`, or make a merge with `jj new <a> <b>`. You change
   other trees' commits this way, so tell the person before you rewrite work you didn't make.
3. **Resolve conflicts here,** in the lead tree, not in the other agents' trees. Fix each one at
   the first conflicted change (see `agent-workflows`); jj won't push a stack that still has one.
4. **Test the combined result** in the lead tree before anything goes up.
5. **Publish only with the person's yes:** bookmarks, pushes and pull requests leave the machine.

## On a git clone

In a git clone (`mori clone --vcs git`) the trees are git worktrees, and they share commits but not
working copies, so handing over means naming commits:

1. **Agents** commit on their detached HEAD and report the SHAs
   (`git log --oneline origin/HEAD..HEAD`). They don't make branches; a branch can be checked out
   in only one worktree.
2. **The lead** makes the combined branch in its own tree (`git switch -c <branch>`) and merges
   each agent's commit (`git merge <sha>`). The lead tree is the one place that branch lives.
3. **Merge rather than rebase or cherry-pick:** a copy leaves the agent's original commits on no
   remote branch, so its tree reads as unsaved until the agent runs
   `git switch --detach <lead's tip>`.
4. **Push** with the person's yes: `git push -u origin <branch>`. The agents' trees become
   removable then, and stay removable after the branch is squash-merged and deleted.

## Handing finished work to the person

People stay on their root checkout (the clone itself, the `default` tree) and look only at finished,
reviewable work. They can open any tree, but a forest of half-done trees gets confusing fast, so
the lead hands them one thing per finished piece of work.

1. **Integrate and test** in the lead tree first. Nothing half-done goes to the person.
2. **Name the result** `ready/<topic>`, local only:
   - jj: `jj bookmark create ready/<topic> -r <last change>`;
   - git: `git branch ready/<topic>` in the lead tree.
3. **Say what it is:** `mori tree set <repo> <lead tree> --purpose "ready for review: <topic>"`.
4. **Tell the person** in one message: the name `ready/<topic>`, what it does, how you tested it,
   and anything they must decide.
5. **Don't touch their root,** and don't push or open a pull request without their yes. Every tree
   shares the clone's commits, so they can already see it:
   - jj: `jj diff -r ready/<topic>` to read it, `jj new ready/<topic>` to try it;
   - git: `git switch --detach ready/<topic>`, and back with `git switch -`.
6. **After their review,** push it and open the pull request when they say so. Once it lands,
   delete the `ready/` name; `mori gc` cleans up the worker trees and, when they ask, the lead tree.

A patch or a pushed branch is only for another machine (see `agent-workflows`).

## When it's done

A pinned tree stays until someone removes it on purpose. When the effort has landed, tell the
person the lead tree can go; don't remove it yourself.

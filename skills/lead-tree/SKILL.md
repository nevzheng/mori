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

Read the `using-mori` skill first; its rules apply here too.

## Make one

```sh
mori tree create github.com/acme/widget --agent claude --task lead --lifetime pinned
```

That gives `trees/widget/claude-lead`, on a new change on top of trunk. Make one lead tree per
coordinated effort, not one per task; if `claude-lead` exists, use a more specific task slug, such
as `lead-auth`.

## Handing work to the lead

There is no handoff command: with jj, every tree of a clone already sees every change, so handing
over is just saying which changes are ready.

**An agent that finishes work for the lead:**

1. Gives every change a description (`jj describe`): that is the note the lead reads.
2. Stops editing its tree.
3. Tells the lead its change IDs (`jj log -r '::<its tree>@ ~ ::trunk()'`) in its final message,
   with one line on what they do.

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
3. **Resolve conflicts here,** in the lead tree, not in the other agents' trees.
4. **Test the combined result** in the lead tree before anything goes up.
5. **Publish only with the person's yes:** bookmarks, pushes and pull requests leave the machine.

With git the same idea holds, and matters more: a branch can be checked out in only one worktree,
so the lead tree is the one place the combined branch lives.

## When it's done

A pinned tree stays until someone removes it on purpose. When the effort has landed, tell the
person the lead tree can go; don't remove it yourself.

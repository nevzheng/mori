---
name: using-mori
description: >-
  How to use the mori CLI, which keeps one clone per repo in a fixed layout and records the repos
  and trees it creates. Use when a task says to set up mori, clone a repo with mori, or work in a
  directory under mori's root (default ~/mori), and before touching anything under that root.
---

# Using mori

mori keeps a forest of repos and trees under one root (`$MORI_ROOT`, default `~/mori`). It
records only what it created. The VCS (jj, colocated with git by default) is the source of truth;
mori's database only tracks who made what, for which task, and how long it may live.

mori is pre-alpha: commands and output may change in any release. This skill describes what the
current version does, and grows with each command.

## Rules

Follow these even when a command would let you do otherwise.

1. **Only touch what mori created.** A clone or workspace mori didn't record is foreign: leave it
   alone. Don't move, delete, adopt or "fix" it.
2. **Never delete work.** Don't remove a clone or tree with uncommitted or unpushed changes. When
   mori keeps something after a failure, it says so; leave it there and report it.
3. **Don't edit mori's state by hand.** Never write to `mori.db` or `config.toml`, and never
   create directories under `repos/` or `trees/` yourself. Use the commands.
4. **Use `--json` and read the error.** Decide what to do from the exit code and the `reason`,
   not from the message text.

## Context

Everything agents should know lives under `context/` in the root, indexed by `context/llms.txt`
(start from the root's `llms.txt`). mori's skills are in `context/skills/<name>/SKILL.md`, where
`mori init` installs them; skills of your own go beside them, and the index lists them too.

### A repo's context folder

Every repo mori clones gets `context/projects/<repo>/` (named like its `trees/<repo>/` folder),
listed in `context/llms.txt`. Before working on a repo, read what is there; put notes, plans and
anything the next agent should know there too. It is shared by the person and every agent and
harness, and mori never changes it after writing its starter `README.md`.

## The layout

```text
$MORI_ROOT/                        default ~/mori
  repos/<host>/<owner>/<repo>/     one clone per repo; also its base tree (yours, pinned)
  trees/<repo>/<name>/             task trees, one per piece of work
  context/                         skills and notes, indexed in context/llms.txt; no stability
                                   guarantee
  llms.txt                         start here
```

Repo identities are all lowercase: `github.com/Acme/Widget` and `github.com/acme/widget` are the
same repo. Config and state follow XDG: `$XDG_CONFIG_HOME/mori/config.toml` and
`$XDG_STATE_HOME/mori/mori.db`, unless `MORI_CONFIG_DIR` or `MORI_STATE_DIR` override them.

## Commands

Every command takes `--json`, which prints exactly one JSON object on stdout, for errors too.

### `mori init`

Sets up the root, config and database once per machine. It only adds things, so it is safe to
run again; a second run changes nothing and says mori is already set up. `--dry-run` lists what it
would create. Clones it finds under `repos/` that mori didn't make are listed as unmanaged and left
alone.

### `mori clone <url>`

Clones a repo into `repos/<host>/<owner>/<repo>` and records it with a pinned base tree.

- **URLs:** `https://host/owner/repo(.git)`, `ssh://git@host/owner/repo`,
  `git@host:owner/repo.git`, or the short `host/owner/repo` (fetched over https). Plain `http://`,
  `file://` and nested groups are refused.
- **`--dry-run`** shows the path, fetch URL and `trees/` directory, and clones nothing.
- **`--vcs jj|git`**: `jj` (the default) makes trees as jj workspaces; `git` makes a plain git
  clone whose trees are detached git worktrees, where git and `gh` work. Without the flag,
  `[vcs] default` in `config.toml` decides. See the `vcs-in-mori` skill.
- **`--no-colocate`** makes a jj-only clone (jj only). By default a jj clone is a git repo too.
- It refuses a repo mori already has, and any path that already exists.

Run `mori init` first. mori needs `jj` on `PATH` for jj clones, and `git` for git clones.

### `mori ls [repo]`

Lists every repo mori manages (or just one) and every tree in it: name, status, owner, task,
lifetime, and the work in it. It only reads; it never snapshots a working copy, so it is always
safe to run, even while others work.

- **Status:** `tree` (mori made it and the VCS has it), `missing` (mori recorded it but the VCS
  has no such workspace or worktree), or `foreign` (the VCS has it but mori didn't make it: leave
  it alone).
- **Work:** `edited` (the working-copy change has edits) and `N unpushed` (changes that exist only
  on this machine). In a jj clone it is as of jj's last snapshot in that tree, so run `jj status`
  in your own tree first if you need it current; a git clone's is always current.
- Clones under `repos/` that mori didn't make are listed apart, and left alone.
- With `--json`, each repo has `vcs` (`VCS_JJ` or `VCS_GIT`); rows are in `repos[].trees[]`, with
  `status` (`STATUS_TREE`, `STATUS_MISSING`, `STATUS_FOREIGN`), `tree`, `state`, and `bookmarks`:
  each bookmark mori has seen the tree push, with `onRemote` and `landed` (gone from the remote
  after a push, as after a squash merge). Fields at their default (`false`, `0`) are left out.

### `mori tree create <repo> --task <slug>`

Gives one piece of work its own tree at `trees/<repo>/<name>`: a jj workspace on a new change on top
of trunk, or in a git clone a detached git worktree at trunk; recorded with its owner, task and
lifetime. Work in that directory; it is yours.

- **`<repo>`** is the full `host/owner/repo` that `mori ls` prints, e.g.
  `github.com/acme/widget`; `widget` alone is refused.
- **`--agent <name>`**: who the tree is for, e.g. `claude`. Always pass it when you are an agent,
  unless your harness sets `MORI_AGENT`; without either, the owner is the person's login name.
- **`--task <slug>`**: lowercase letters, digits and hyphens, e.g. `fix-login`. The tree's name is
  `<owner>-<task>` by default (the `[trees] name` template in `config.toml` can change it).
- **`--lifetime`**: `pinned`, `task-done` (the default), `lru`, or `ttl:<n>d`. A tree only becomes
  a cleanup candidate by its lifetime; nothing is removed without the safety checks.
- **`--from <revset>`**: where the new change starts; `trunk()` by default.
- **`--dry-run`** shows the name, path and lifetime, and creates nothing.

The output (and `tree.path` with `--json`) is the directory to work in. One tree per task: if the
name is taken, pick another task slug rather than reusing someone else's tree. For a long-lived
coordinating tree, see the `lead-tree` skill.

### `mori tree remove <repo> <name>`

Removes a task tree when its work is done and safe elsewhere. mori first lets jj snapshot the
tree, then removes it only if nothing in it exists only on this machine: no edits, and every
change is either on the remote (in a pushed bookmark or trunk) or part of work that landed (a
bookmark mori saw pushed from the tree that was since merged and deleted, as after a squash
merge). Then it forgets the workspace, deletes the directory and drops the record.

- After you push, let mori see the bookmark (`mori ls` does) before the pull request merges;
  that is how it knows the work landed once the remote deletes the bookmark.
- **`--pinned`**: needed for a pinned tree, such as a lead tree. Only with the person's say-so.
- **`--dry-run`**: checks everything and removes nothing.
- If it refuses with `TREE_HAS_UNSAVED_WORK`, push your work (or ask the person whether to
  abandon it). Never delete the directory yourself to get around it.
- It never removes the clone itself or a workspace mori didn't make. If mori's record has no
  workspace any more (`missing` in `mori ls`), it drops the record and leaves the directory.

### `mori gc [repo]`

Reports which trees may be removed and whether each is safe to. Without `--apply` it changes no
tree (it only fetches remote bookmarks; `--offline` skips that and GitHub). Each tree gets a class
and a reason:

- **remove**: its lifetime lets it go and nothing would be lost (`LANDED`, `IDLE`, `OVER_CAP` beyond
  `[trees.lru] max`, or `MISSING` when only a stale record is left).
- **blocked** (`UNSAVED`): it may go, but has edits or unpushed work. Push it or ask the person.
- **keep**: not yet (`NOT_YET`), or it can't be told (`UNKNOWN`: offline, or no `gh` login).
- **never**: the clone itself, a pinned tree, or a workspace mori didn't make.

`mori gc --apply --yes` also removes the removable trees, at most 10 per run (`--max`, `--only
<name>`). Each is snapshotted and judged again first, and kept if it changed; each removal pins the
tree's commit and goes into the journal, and the output gives the `mori restore <entry>` command
that undoes it. Without `--yes` nothing is removed (`CONFIRMATION_NEEDED`); `--apply --dry-run`
shows what would go. Removing is the person's decision: don't run `--apply` unless they asked.

### `mori restore <entry>`

Brings back a tree `mori gc --apply` removed, from the journal entry it printed: its
workspace on the commit mori pinned when it removed it, at its old path, under its old record.
If that commit is gone (`RESTORE_COMMIT_GONE`), it changes nothing; if the name or path is taken
again, it refuses like `tree create`.

### `mori skills sync`

Updates mori's skills in the root (`context/skills/`, and the `context/llms.txt` and `llms.txt`
indexes) to
this mori's version. It changes only files mori wrote and nobody edited since; an edited skill is
kept, and a skill mori never wrote is left alone. `mori init` installs missing skills but never
updates them. `--dry-run` shows what would change.

## Errors

Errors follow Google AIP-193. The exit code is the `google.rpc.Code` number. With `--json`, the
error is a `google.rpc.Status`:

```json
{"code": 6, "message": "mori already manages github.com/acme/widget",
 "details": [{"@type": "type.googleapis.com/google.rpc.ErrorInfo", "reason": "REPO_EXISTS",
              "domain": "repo.mori", "metadata": {"repo": "github.com/acme/widget"}}]}
```

| Exit | Reason                  | What it means, and what to do                                                   |
| ---- | ----------------------- | ------------------------------------------------------------------------------- |
| 3    | `INVALID_USAGE`         | Bad flags or arguments. Check `mori <command> --help`.                          |
| 3    | `CLONE_URL_INVALID`     | The URL isn't a form mori accepts. Use one of the forms above.                  |
| 3    | `COLOCATE_NEEDS_JJ`     | `--no-colocate` with `--vcs git`. A git clone is always git; drop one flag.     |
| 3    | `TREE_NAME_INVALID`     | The task slug makes a bad name. Use lowercase letters, digits and hyphens.      |
| 5    | `REPO_NOT_MANAGED`      | mori didn't clone this repo. `mori clone` it first, if the task allows.         |
| 5    | `TREE_NOT_FOUND`        | No tree of that name. Check `mori ls`.                                          |
| 5    | `ENTRY_NOT_FOUND`       | No such journal entry. Check the output of `mori gc --apply`.                   |
| 6    | `REPO_EXISTS`           | mori already has this repo. Use the existing clone; don't clone again.          |
| 6    | `PATH_EXISTS`           | Something mori didn't make is at the path. Leave it; tell the person.           |
| 6    | `TREE_EXISTS`           | A tree of that name exists. Use a different task slug; don't take it over.      |
| 6    | `WORKSPACE_EXISTS`      | A workspace mori didn't make has that name. Leave it; pick another slug.        |
| 9    | `TREE_HAS_UNSAVED_WORK` | The tree has edits or unpushed changes. Push them, or ask the person.           |
| 9    | `TREE_PINNED`           | A pinned tree. Remove it only if the person asked; then pass `--pinned`.        |
| 9    | `BASE_TREE`             | That's the clone itself. mori never removes it.                                 |
| 9    | `CONFIRMATION_NEEDED`   | `mori gc --apply` needs `--yes`: only when the person asked for it.             |
| 9    | `RESTORE_COMMIT_GONE`   | The removed tree's commit is gone; it can't come back. Tell the person.         |
| 9    | `NOT_INITIALIZED`       | mori isn't set up. Run `mori init` if the task allows, else ask.                |
| 9    | `ROOT_MISMATCH`         | `MORI_ROOT` differs from the recorded root. Don't move it; ask the person.      |
| 9    | `JJ_NOT_FOUND`          | jj isn't installed or on `PATH`. Ask the person to install it.                  |
| 9    | `GIT_NOT_FOUND`         | git isn't installed or on `PATH`. Ask the person to install it.                 |
| 9    | `OWNER_UNKNOWN`         | No owner: pass `--agent <name>`.                                                |
| 13   | `JJ_FAILED`             | jj failed (network, auth, missing repo). Nothing was left behind; see message.  |
| 13   | `GIT_FAILED`            | git failed (network, auth, bad revision). Nothing was left behind; see message. |
| 13   | `CLONE_NOT_RECORDED`    | The clone worked but wasn't recorded. It is kept; don't delete it. Report it.   |
| 13   | `TREE_NOT_RECORDED`     | The tree was made but not recorded. It is kept; don't delete it. Report it.     |

Other reasons (`CONFIG_INVALID`, `DATABASE_NOT_MORI`, `DATABASE_SCHEMA_TOO_NEW`, `IO_ERROR`,
`DATABASE_ERROR`, …) mean something outside the task is wrong: stop and report the message and
reason.

## Not yet

Designed but not built: pruning old pins and journal entries, and cleaning up bookmarks and
branches. Leave those alone. Use `mori tree create` rather than making jj workspaces or git
worktrees in a mori clone by hand: mori would list those as foreign.

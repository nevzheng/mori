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

## Skills

mori's skills live in `skills/<name>/SKILL.md`, indexed by `skills/llms.txt`, both in mori's
source repo and in every root (`~/mori/skills/`), where `mori init` installs them. Read the index
to find the skill for a task. Skills of your own go beside mori's; the index lists them too.

## The layout

```text
$MORI_ROOT/                        default ~/mori
  repos/<host>/<owner>/<repo>/     one clone per repo; also its base tree (yours, pinned)
  trees/<repo>/<name>/             task trees, one per piece of work
  skills/  projects/  llms.txt     reserved for people and LLMs; no stability guarantee
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
- **`--no-colocate`** makes a jj-only clone. By default the clone is a git repo too.
- It refuses a repo mori already has, and any path that already exists.

Run `mori init` first. mori needs `jj` on `PATH`.

### `mori ls [repo]`

Lists every repo mori manages (or just one) and every tree in it: name, status, owner, task,
lifetime, and the work in it. It only reads; it never snapshots a working copy, so it is always
safe to run, even while others work.

- **Status:** `tree` (mori made it and jj has it), `missing` (mori recorded it but jj has no such
  workspace), or `foreign` (jj has it but mori didn't make it: leave it alone).
- **Work:** `edited` (the working-copy change has edits) and `N unpushed` (changes that exist only
  on this machine). It is as of jj's last snapshot in that tree, so run `jj status` in your own
  tree first if you need it current.
- Clones under `repos/` that mori didn't make are listed apart, and left alone.
- With `--json`, rows are in `repos[].trees[]`, with `status` (`STATUS_TREE`, `STATUS_MISSING`,
  `STATUS_FOREIGN`), `tree` and `state`. Fields at their default (`false`, `0`) are left out.

### `mori tree create <repo> --task <slug>`

Gives one piece of work its own tree: a jj workspace at `trees/<repo>/<name>`, on a new change on
top of trunk, recorded with its owner, task and lifetime. Work in that directory; it is yours.

- **`--agent <name>`**: who the tree is for, e.g. `claude`. Always pass it when you are an agent;
  without it the owner is the person's login name.
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
- **`--agent <name>`**: you can only remove your own trees; pass the same name you created it
  with.
- **`--pinned`**: needed for a pinned tree, such as a lead tree. Only with the person's say-so.
- **`--dry-run`**: checks everything and removes nothing.
- If it refuses with `TREE_HAS_UNSAVED_WORK`, push your work (or ask the person whether to
  abandon it). Never delete the directory yourself to get around it.
- It never removes the clone itself or a workspace mori didn't make. If mori's record has no
  workspace any more (`missing` in `mori ls`), it drops the record and leaves the directory.

### `mori skills sync`

Updates mori's skills in the root (`skills/`, and the `skills/llms.txt` and `llms.txt` indexes) to
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

| Exit | Reason                  | What it means, and what to do                                                  |
| ---- | ----------------------- | ------------------------------------------------------------------------------ |
| 3    | `INVALID_USAGE`         | Bad flags or arguments. Check `mori <command> --help`.                         |
| 3    | `CLONE_URL_INVALID`     | The URL isn't a form mori accepts. Use one of the forms above.                 |
| 3    | `TREE_NAME_INVALID`     | The task slug makes a bad name. Use lowercase letters, digits and hyphens.     |
| 5    | `REPO_NOT_MANAGED`      | mori didn't clone this repo. `mori clone` it first, if the task allows.        |
| 5    | `TREE_NOT_FOUND`        | No tree of that name. Check `mori ls`.                                         |
| 6    | `REPO_EXISTS`           | mori already has this repo. Use the existing clone; don't clone again.         |
| 6    | `PATH_EXISTS`           | Something mori didn't make is at the path. Leave it; tell the person.          |
| 6    | `TREE_EXISTS`           | A tree of that name exists. Use a different task slug; don't take it over.     |
| 6    | `WORKSPACE_EXISTS`      | A workspace mori didn't make has that name. Leave it; pick another slug.       |
| 9    | `TREE_HAS_UNSAVED_WORK` | The tree has edits or unpushed changes. Push them, or ask the person.          |
| 9    | `NOT_TREE_OWNER`        | Someone else's tree. Leave it; tell the person if it looks abandoned.          |
| 9    | `TREE_PINNED`           | A pinned tree. Remove it only if the person asked; then pass `--pinned`.       |
| 9    | `BASE_TREE`             | That's the clone itself. mori never removes it.                                |
| 9    | `NOT_INITIALIZED`       | mori isn't set up. Run `mori init` if the task allows, else ask.               |
| 9    | `ROOT_MISMATCH`         | `MORI_ROOT` differs from the recorded root. Don't move it; ask the person.     |
| 9    | `JJ_NOT_FOUND`          | jj isn't installed or on `PATH`. Ask the person to install it.                 |
| 9    | `OWNER_UNKNOWN`         | No owner: pass `--agent <name>`.                                               |
| 13   | `JJ_FAILED`             | jj failed (network, auth, missing repo). Nothing was left behind; see message. |
| 13   | `CLONE_NOT_RECORDED`    | The clone worked but wasn't recorded. It is kept; don't delete it. Report it.  |
| 13   | `TREE_NOT_RECORDED`     | The tree was made but not recorded. It is kept; don't delete it. Report it.    |

Other reasons (`CONFIG_INVALID`, `DATABASE_NOT_MORI`, `DATABASE_SCHEMA_TOO_NEW`, `IO_ERROR`,
`DATABASE_ERROR`, …) mean something outside the task is wrong: stop and report the message and
reason.

## Not yet

These are designed but not built, so don't look for them: automatic cleanup (a report of trees
that may go, confirmed in batches, with a journal) and restore. Until then, remove your own trees
with `mori tree remove` when your work is pushed. Use `mori tree create` rather than making jj
workspaces or git worktrees in a mori clone by hand: mori would list those as foreign.

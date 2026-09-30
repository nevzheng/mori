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

mori's skills live in `skills/<name>/SKILL.md`, indexed by `skills/llms.txt`. Read the index to
find the skill for a task. The same layout suits skills of your own.

## The layout

```text
$MORI_ROOT/                        default ~/mori
  repos/<host>/<owner>/<repo>/     one clone per repo; also its base tree (yours, pinned)
  trees/<repo>/<name>/             task trees (not created by any command yet)
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

## Errors

Errors follow Google AIP-193. The exit code is the `google.rpc.Code` number. With `--json`, the
error is a `google.rpc.Status`:

```json
{"code": 6, "message": "mori already manages github.com/acme/widget",
 "details": [{"@type": "type.googleapis.com/google.rpc.ErrorInfo", "reason": "REPO_EXISTS",
              "domain": "repo.mori", "metadata": {"repo": "github.com/acme/widget"}}]}
```

| Exit | Reason               | What it means, and what to do                                                  |
| ---- | -------------------- | ------------------------------------------------------------------------------ |
| 3    | `INVALID_USAGE`      | Bad flags or arguments. Check `mori <command> --help`.                         |
| 3    | `CLONE_URL_INVALID`  | The URL isn't a form mori accepts. Use one of the forms above.                 |
| 6    | `REPO_EXISTS`        | mori already has this repo. Use the existing clone; don't clone again.         |
| 6    | `PATH_EXISTS`        | Something mori didn't make is at the path. Leave it; tell the person.          |
| 9    | `NOT_INITIALIZED`    | mori isn't set up. Run `mori init` if the task allows, else ask.               |
| 9    | `ROOT_MISMATCH`      | `MORI_ROOT` differs from the recorded root. Don't move it; ask the person.     |
| 9    | `JJ_NOT_FOUND`       | jj isn't installed or on `PATH`. Ask the person to install it.                 |
| 13   | `JJ_FAILED`          | jj failed (network, auth, missing repo). Nothing was left behind; see message. |
| 13   | `CLONE_NOT_RECORDED` | The clone worked but wasn't recorded. It is kept; don't delete it. Report it.  |

Other reasons (`CONFIG_INVALID`, `DATABASE_NOT_MORI`, `DATABASE_SCHEMA_TOO_NEW`, `IO_ERROR`,
`DATABASE_ERROR`, …) mean something outside the task is wrong: stop and report the message and
reason.

## Not yet

These are designed but not built, so don't look for them: `mori ls`, `mori tree create`,
`mori tree remove`, cleanup and restore. Until `mori tree create` exists, don't make your own jj
workspaces or git worktrees in a mori clone unless the person asks; mori would list them as
foreign.

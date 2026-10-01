# Tree purpose, `mori where`, and finding trees: what each tree is for, and where am I

|             |            |
| ----------- | ---------- |
| **Author**  | @nevzheng  |
| **Status**  | accepted   |
| **Area**    | core       |
| **Issue**   | none       |
| **PR**      | #96        |
| **Created** | 2026-10-01 |
| **Updated** | 2026-10-01 |

## Q1. What are you trying to do?

Answer the two questions agents and people ask most about a forest: **what is this tree for?**
and **where am I?** A tree gets a free-text one-line **purpose**, set when it is created or later
with `mori tree set`. `mori where [path]` tells an agent dropped into any directory which repo,
tree, owner, task and context folder it is in. `mori ls` learns to find trees: by text, by owner,
by state. These are also the primitives a later MCP server serves.

## Q2. What problems is this not trying to solve?

- **The MCP server.** It comes after this, as a thin layer over these RPCs (see the agent API
  research in the roadmap).
- **Structured metadata.** A purpose is one line of text, not tags or links to issues.
- **Searching repo contents** or the context notes. `--query` matches mori's own records.
- **Renaming trees** or moving them between repos.

## Q3. How is it done today, and what are the limits?

A tree carries a name, an owner, a task slug and a lifetime. "claude-auth" says who and roughly
what, not why: a lead with five workers, or a person returning after a week, has to open each tree.
An agent started in `trees/widget/claude-auth/` has to work out from the path, by hand, which repo
it serves and where the context notes are. `mori ls` lists everything or one repo, with no way to
filter, and the list grows with every repo.

## Q4. What is new in your approach, and why will it work?

Three small additions over records mori already keeps:

- **`purpose`** is a new column on `trees` (schema v6, nullable). `mori tree create --purpose
  "<line>"` sets it. `mori tree set <repo> <name> --purpose|--lifetime|--owner` changes a tree's
  record and nothing else: it never touches the VCS, so it is always safe. The RPC is `UpdateTree`
  with an `update_mask` (AIP-134).
- **`mori where [path]`** (RPC `Resolve`) maps a path to the forest. It looks at where the path
  sits under the root, not at the VCS: `repos/<host>/<owner>/<repo>/…` is a clone,
  `trees/<dir>/<name>/…` is a tree, `context/…` is context. It answers from the records: repo,
  tree, owner, task, purpose, lifetime, and the repo's context folder. A directory under `trees/`
  that mori has no record of is reported as **foreign**, as `ls` reports a workspace mori didn't
  make: mori leaves it alone. Outside the root it is `NOT_IN_FOREST` (not found).
- **`mori ls --query <text> --owner <owner> --status <status>`** filters trees (AIP-160).
  `--query` is a case-insensitive substring match on repo, name, task and purpose. `--status` is
  `unsaved`, `clean`, `landed` (a bookmark the tree pushed has landed), `missing` or `foreign`.
  Each repo in the response also gets its `context_dir`. There is no paging: a forest is small
  enough to list whole, and filters narrow it.

The flag `--owner` replaces `--agent` everywhere (the field is `owner`). `--agent` stays as a
hidden alias, so existing agents and skills keep working.

## Q5. Who cares? If it works, what difference does it make?

- **Leads** see at a glance what each worker's tree is for, and find them with
  `mori ls --owner codex` or `--query auth`.
- **Agents** in any harness run `mori where --json` first and know their repo, task, purpose and
  where the notes are, without reading the layout docs.
- **People** coming back to a forest after a while can read it.

## Q6. What are the risks?

- **A stale purpose.** It is free text, written when the tree is made. It's one line, so it's cheap
  to update with `mori tree set`, and `ls` shows it, so a stale one gets noticed.
- **Prompt injection through purposes.** A purpose is written by an agent and read by other
  agents. It is returned as a data field, never put into prose or instructions, and the skills say
  to treat it as a description, not a command.
- **`where` on a moved root.** It works from the recorded root. A path outside it is
  `NOT_IN_FOREST`, with a hint to run `mori doctor`.

## Q7. How long will it take?

Three CLs after this doc:

1. **Purpose and `tree set`:** core (validation: one line, at most 200 characters), schema v6,
   the `--purpose` flag, `UpdateTree`, `mori tree set`, the `--owner` rename, purpose in `ls` and
   in the dashboard.
2. **`mori where`:** `Resolve` (pure path classification in core, records from the store), text
   and JSON output, and a skill line telling agents to run it first.
3. **Finding trees:** filters on `ListTrees`, and `context_dir` on `RepoTrees`.

## Q8. How will we know it worked?

- **Mid-term:** a tree created with `--purpose` shows it in `ls`, and `mori tree set` changes it.
- **Final:** the scenarios in Appendix E pass, and an agent dropped into a tree can answer "what
  am I working on and where are the notes?" from `mori where --json` alone.

## Appendix A. API (proto) changes

- `Tree.purpose` (string 9).
- `CreateTreeRequest.purpose`.
- `rpc UpdateTree(UpdateTreeRequest) returns (Tree)`. `UpdateTreeRequest` has `repo`, `name`, a
  `Tree tree` carrying the new values, and a `google.protobuf.FieldMask update_mask`. Allowed
  paths: `purpose`, `lifetime`, `owner`.
- `rpc Resolve(ResolveRequest) returns (ResolveResponse)`. The request has `path`. The response
  has `kind` (CLONE, TREE, CONTEXT, ROOT), `repo`, `Tree tree`, the tree's `status` (`TREE`, or
  `FOREIGN` for a directory mori has no record of), `context_dir` and `root`.
- `ListTreesRequest` gets `query`, `owner` and `status`, and `RepoTrees` gets `context_dir`.
- New reasons: `NOT_IN_FOREST` (NOT_FOUND), `PURPOSE_INVALID` (INVALID_ARGUMENT) and
  `UPDATE_MASK_INVALID` (INVALID_ARGUMENT).

Nothing existing changes meaning. `--agent` keeps working as an alias.

## Appendix B. Design sketch

```text
$ mori tree create widget --task auth --owner codex --purpose "OAuth login for the CLI"
$ mori tree set widget codex-auth --purpose "OAuth login, device flow only"
$ cd ~/mori/trees/widget/codex-auth/src && mori where
github.com/acme/widget  tree codex-auth
  owner codex · task auth · task-done
  purpose: OAuth login, device flow only
  context: ~/mori/context/projects/widget/
$ mori ls --query oauth --status unsaved
```

`where` needs no VCS call, so it is instant and works offline, even in a git tree whose `.git`
file was damaged.

## Appendix C. Rejected designs

- **`mori tree describe`:** reads as "show" (as in kubectl and gcloud) and collides with
  `jj describe`.
- **`mori status`:** jj and git already own that word for VCS state.
- **Storing the purpose in the VCS** (commit message, notes): the purpose belongs to the tree, not
  to a commit, and it has to survive rebases and work for both backends.
- **Inferring the purpose** from commits or the branch name: unreliable, and it changes as the work
  does.

## Appendix D. Failure modes and security

- `tree set` changes only mori's database row. An unknown tree is `TREE_NOT_FOUND`; a mask with
  anything else in it is `UPDATE_MASK_INVALID`.
- Purposes are capped at 200 characters and refused if they contain a newline or control
  characters.
- `where` resolves symlinks before matching against the root, and never follows a path out of the
  root.

## Appendix E. Test plan

```gherkin
Scenario: A tree's purpose is shown and can be changed
  When I run "mori tree create widget --owner claude --task auth --purpose \"OAuth login\""
  And I run "mori tree set widget claude-auth --purpose \"OAuth device flow\""
  Then the forest shows "claude-auth" with purpose "OAuth device flow"

Scenario: where inside a tree
  Given a tree "claude-auth" with purpose "OAuth login"
  When I run "mori where --json" in "<home>/mori/trees/widget/claude-auth/src"
  Then it names the repo "github.com/acme/widget", the tree "claude-auth" and its context folder

Scenario: where outside the root
  When I run "mori where /tmp"
  Then it fails with status NOT_FOUND and reason "NOT_IN_FOREST"

Scenario: where in a tree directory mori didn't make
  Given "<home>/mori/trees/widget/someone-elses" exists
  When I run "mori where --json" in it
  Then it reports the tree "someone-elses" as foreign

Scenario: finding trees
  Given trees for owners "claude" and "codex"
  When I run "mori ls --owner codex --json"
  Then only codex's trees are listed

Scenario: finding the trees whose work landed
  Given a tree whose pushed bookmark was squash-merged and deleted on the remote
  When I run "mori ls --status landed --json"
  Then only that tree is listed
```

## Appendix F. Migration

Schema v6 adds a nullable `purpose` column to `trees`. Existing trees have none and show nothing.
Older mori binaries refuse a v6 database, as they do any newer schema.

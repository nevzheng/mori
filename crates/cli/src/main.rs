//! The `mori` command.
//!
//! Every failure is a `google.rpc.Status`, and the exit code is its canonical code's number: 0 for
//! success, 3 for bad arguments, 9 for a failed precondition, and so on.

mod clone;
mod gc;
mod gc_apply;
mod init;
mod landing;
mod ls;
mod output;
mod restore;
mod skills;
mod state;
mod tree;
mod tree_remove;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mori_core::error::ErrorDetails;
use mori_core::tree::Lifetime;

/// mori looks after a forest of repos and worktrees, for you and your agents.
#[derive(Debug, Parser)]
#[command(name = "mori", version)]
struct Cli {
    /// Print one JSON object on stdout instead of text, for errors too.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Set up mori's root, config and database. Only adds things; safe to run again.
    Init {
        /// Show what would be created, and create nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Clone a repo into repos/<host>/<owner>/<repo> and record it. Refuses a repo mori already
    /// has, and any directory it didn't make.
    Clone {
        /// What to clone: https://…, ssh://…, git@host:owner/repo, or host/owner/repo.
        url: String,

        /// Show what would happen, and clone nothing.
        #[arg(long)]
        dry_run: bool,

        /// Make a jj-only clone. By default the clone is a git repo too (colocated).
        #[arg(long)]
        no_colocate: bool,
    },

    /// Report which trees may be removed and whether each is safe to remove. With --apply --yes,
    /// also remove a batch of them: each is checked again first, and each removal is journalled,
    /// so `mori restore` can undo it.
    Gc {
        /// Only this repo, in any form `mori clone` accepts.
        repo: Option<String>,

        /// Don't fetch or ask GitHub; use only what the clones already know.
        #[arg(long)]
        offline: bool,

        /// Also remove the trees the report finds removable (needs --yes).
        #[arg(long)]
        apply: bool,

        /// Confirm the removal.
        #[arg(long, requires = "apply")]
        yes: bool,

        /// With --apply: only these trees (repeat for more).
        #[arg(long = "only", requires = "apply")]
        names: Vec<String>,

        /// With --apply: the most trees to remove (default 10).
        #[arg(long, requires = "apply")]
        max: Option<u32>,

        /// With --apply: check everything and remove nothing.
        #[arg(long, requires = "apply")]
        dry_run: bool,
    },

    /// List the repos mori manages and their trees: owner, task, lifetime, and work that exists
    /// only on this machine. Reads only; never snapshots a working copy.
    Ls {
        /// Only this repo, in any form `mori clone` accepts.
        repo: Option<String>,
    },

    /// Bring back a tree `mori gc --apply` removed, from its journal entry: on its pinned commit, at
    /// its old path, under its old record.
    Restore {
        /// The journal entry, from `mori gc --apply`.
        entry: String,
    },

    /// Agent skills in the root: mori's own, installed by `mori init`, and anyone else's.
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },

    /// Work with trees: the jj workspaces mori creates for tasks.
    Tree {
        #[command(subcommand)]
        command: TreeCommand,
    },
}

#[derive(Debug, Subcommand)]
enum SkillsCommand {
    /// Update mori's skills and the llms.txt indexes to this mori's version. Changes only files
    /// mori wrote and nobody edited; never touches anyone else's skills.
    Sync {
        /// Show what would change, and write nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum TreeCommand {
    /// Give one task its own tree: a jj workspace under trees/<repo>/, on a new change on top of
    /// trunk, recorded with its owner, task and lifetime.
    Create {
        /// The repo, in any form `mori clone` accepts, e.g. github.com/acme/widget.
        repo: String,

        /// A short slug for the work: lowercase letters, digits and hyphens.
        #[arg(long)]
        task: String,

        /// Who the tree is for, e.g. claude. Defaults to `$MORI_AGENT`, then your login name.
        #[arg(long)]
        agent: Option<String>,

        /// When the tree may go: pinned, task-done, lru, or ttl:<n>d. Defaults to the
        /// [trees.lifetime] task setting (task-done).
        #[arg(long)]
        lifetime: Option<Lifetime>,

        /// The jj revision to start from. Defaults to `trunk()`.
        #[arg(long)]
        from: Option<String>,

        /// Show what would happen, and create nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove a task tree, only when nothing in it exists only on this machine: no edits and no
    /// change missing from the remote. Never removes the clone itself or a workspace mori didn't
    /// make.
    Remove {
        /// The repo, in any form `mori clone` accepts.
        repo: String,

        /// The tree's name, e.g. claude-fix-login.
        name: String,

        /// Who is asking; must be the tree's owner. Defaults to `$MORI_AGENT`, then your login name.
        #[arg(long)]
        agent: Option<String>,

        /// Allow removing a pinned tree.
        #[arg(long)]
        pinned: bool,

        /// Check everything and remove nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return output::usage_error(&error),
    };
    if matches!(cli.command, Command::Init { .. } | Command::Ls { .. }) {
        skills::stale_hint();
    }
    let json = cli.json;
    match cli.command {
        Command::Init { dry_run } => respond(json, init::run(dry_run), output::init_text),
        Command::Clone {
            url,
            dry_run,
            no_colocate,
        } => respond(
            json,
            clone::run(&url, dry_run, no_colocate),
            output::clone_text,
        ),
        Command::Skills {
            command: SkillsCommand::Sync { dry_run },
        } => respond(json, skills::sync(dry_run), output::skills_text),
        Command::Gc {
            repo,
            offline,
            apply,
            yes,
            names,
            max,
            dry_run,
        } => respond(
            json,
            gc::run(&gc::GcArgs {
                repo,
                offline,
                apply: apply.then_some(gc::Apply {
                    yes,
                    names,
                    max,
                    dry_run,
                }),
            }),
            output::gc_text,
        ),
        Command::Restore { entry } => respond(json, restore::run(&entry), output::restore_text),
        Command::Ls { repo } => respond(json, ls::run(repo.as_deref()), output::ls_text),
        Command::Tree {
            command:
                TreeCommand::Create {
                    repo,
                    task,
                    agent,
                    lifetime,
                    from,
                    dry_run,
                },
        } => respond(
            json,
            tree::create(tree::CreateArgs {
                repo,
                task,
                agent,
                lifetime,
                from,
                dry_run,
            }),
            output::tree_create_text,
        ),
        Command::Tree {
            command:
                TreeCommand::Remove {
                    repo,
                    name,
                    agent,
                    pinned,
                    dry_run,
                },
        } => respond(
            json,
            tree_remove::run(tree_remove::RemoveArgs {
                repo,
                name,
                agent,
                pinned,
                dry_run,
            }),
            output::tree_remove_text,
        ),
    }
}

/// Prints a command's result: its response as JSON or text, or its error.
fn respond<T: serde::Serialize>(
    json: bool,
    result: Result<T, Box<dyn ErrorDetails>>,
    text: fn(&T) -> String,
) -> ExitCode {
    match result {
        Ok(response) => output::success(json, &response, &text(&response)),
        Err(error) => output::failure(json, error.as_ref()),
    }
}

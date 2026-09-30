//! The `mori` command.
//!
//! Every failure is a `google.rpc.Status`, and the exit code is its canonical code's number: 0 for
//! success, 3 for bad arguments, 9 for a failed precondition, and so on.

mod clone;
mod init;
mod output;
mod state;
mod tree;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
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

    /// Work with trees: the jj workspaces mori creates for tasks.
    Tree {
        #[command(subcommand)]
        command: TreeCommand,
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

        /// Who the tree is for, e.g. claude. Defaults to your login name.
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
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return output::usage_error(&error),
    };
    match cli.command {
        Command::Init { dry_run } => match init::run(dry_run) {
            Ok(response) => {
                output::success(cli.json, &response, output::init_text(&response).as_str())
            }
            Err(error) => output::failure(cli.json, error.as_ref()),
        },
        Command::Clone {
            url,
            dry_run,
            no_colocate,
        } => match clone::run(&url, dry_run, no_colocate) {
            Ok(response) => {
                output::success(cli.json, &response, output::clone_text(&response).as_str())
            }
            Err(error) => output::failure(cli.json, error.as_ref()),
        },
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
        } => match tree::create(tree::CreateArgs {
            repo,
            task,
            agent,
            lifetime,
            from,
            dry_run,
        }) {
            Ok(response) => output::success(
                cli.json,
                &response,
                output::tree_create_text(&response).as_str(),
            ),
            Err(error) => output::failure(cli.json, error.as_ref()),
        },
    }
}

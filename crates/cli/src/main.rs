//! The `mori` command.
//!
//! Every failure is a `google.rpc.Status`, and the exit code is its canonical code's number: 0 for
//! success, 3 for bad arguments, 9 for a failed precondition, and so on.

mod init;
mod output;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

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
    }
}

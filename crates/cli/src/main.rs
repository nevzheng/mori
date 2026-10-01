//! The `mori` command.
//!
//! Every failure is a `google.rpc.Status`, and the exit code is its canonical code's number: 0 for
//! success, 3 for bad arguments, 9 for a failed precondition, and so on.

mod output;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mori_app::routed::Routed;
use mori_app::{App, Host, clone, doctor, gc, init, ls, restore, skills, tree, tree_remove};
use mori_core::error::ErrorDetails;
use mori_core::tree::Lifetime;
use mori_core::vcs::VcsKind;
use mori_git::GitCli;
use mori_github::GhCli;
use mori_jj::JjCli;

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

        /// Make a jj-only clone. By default a jj clone is a git repo too (colocated).
        #[arg(long)]
        no_colocate: bool,

        /// The VCS for the clone and its trees: jj (workspaces) or git (detached worktrees).
        /// Defaults to `[vcs] default` in config.toml, else jj.
        #[arg(long)]
        vcs: Option<VcsKind>,
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

        /// Free at least this much, e.g. 200G: pick trees that are safe to remove, least
        /// recently changed first. Shows the plan; with --apply --yes, removes them.
        #[arg(long, value_name = "SIZE", value_parser = parse_size, conflicts_with = "names")]
        free: Option<u64>,
    },

    /// List the repos mori manages and their trees: owner, task, lifetime, and work that exists
    /// only on this machine. Reads only; never snapshots a working copy.
    Ls {
        /// Only this repo, in any form `mori clone` accepts.
        repo: Option<String>,

        /// Show how much disk each tree uses. Walks every file, so it can take a while; sizes
        /// measured in the last 15 minutes are reused.
        #[arg(long)]
        size: bool,

        /// With --size: measure every tree again instead of reusing recent sizes.
        #[arg(long, requires = "size")]
        fresh: bool,
    },

    /// Check the root, every clone and every tree against the VCS and the disk, and list each
    /// problem with the command that fixes it. Reads only, unless --fix --yes repairs the safe
    /// ones (removals are journalled for `mori restore`). Exits 9 if a problem remains.
    Doctor {
        /// Only this repo, in any form `mori clone` accepts.
        repo: Option<String>,

        /// Repair the auto-fixable findings (needs --yes).
        #[arg(long)]
        fix: bool,

        /// Confirm the repairs.
        #[arg(long, requires = "fix")]
        yes: bool,

        /// With --fix: check everything and repair nothing.
        #[arg(long, requires = "fix")]
        dry_run: bool,
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
    let app = real_app();
    if matches!(cli.command, Command::Init { .. } | Command::Ls { .. }) {
        skills::stale_hint(&app.host);
    }
    dispatch(&app, cli)
}

/// Runs the parsed command and prints its result.
fn dispatch(app: &App<Routed<JjCli, GitCli>, GhCli>, cli: Cli) -> ExitCode {
    let json = cli.json;
    match cli.command {
        Command::Init { dry_run } => {
            respond(json, init::run(&app.host, dry_run), output::init_text)
        }
        Command::Clone {
            url,
            dry_run,
            no_colocate,
            vcs,
        } => respond(
            json,
            clone::run(app, &url, dry_run, no_colocate, vcs),
            output::clone_text,
        ),
        Command::Skills {
            command: SkillsCommand::Sync { dry_run },
        } => respond(json, skills::sync(&app.host, dry_run), output::skills_text),
        Command::Gc {
            repo,
            offline,
            apply,
            yes,
            names,
            max,
            dry_run,
            free,
        } => respond(
            json,
            gc::run(
                app,
                &gc::GcArgs {
                    repo,
                    offline,
                    apply: apply.then_some(gc::Apply {
                        yes,
                        names,
                        max,
                        dry_run,
                    }),
                    free,
                },
            ),
            output::gc_text,
        ),
        Command::Doctor {
            repo,
            fix,
            yes,
            dry_run,
        } => run_doctor(
            app,
            json,
            repo.as_deref(),
            fix.then_some(doctor::Fix { yes, dry_run }),
        ),
        Command::Restore { entry } => {
            respond(json, restore::run(app, &entry), output::restore_text)
        }
        Command::Ls { repo, size, fresh } => respond(
            json,
            ls::run(
                app,
                &ls::LsArgs {
                    repo,
                    sizes: size,
                    fresh,
                },
            ),
            output::ls_text,
        ),
        Command::Tree { command } => dispatch_tree(app, json, command),
    }
}

/// Runs a `mori tree` command and prints its result.
fn dispatch_tree(
    app: &App<Routed<JjCli, GitCli>, GhCli>,
    json: bool,
    command: TreeCommand,
) -> ExitCode {
    match command {
        TreeCommand::Create {
            repo,
            task,
            agent,
            lifetime,
            from,
            dry_run,
        } => respond(
            json,
            tree::create(
                app,
                tree::CreateArgs {
                    repo,
                    task,
                    agent,
                    lifetime,
                    from,
                    dry_run,
                },
            ),
            output::tree_create_text,
        ),
        TreeCommand::Remove {
            repo,
            name,
            pinned,
            dry_run,
        } => respond(
            json,
            tree_remove::run(
                app,
                tree_remove::RemoveArgs {
                    repo,
                    name,
                    pinned,
                    dry_run,
                },
            ),
            output::tree_remove_text,
        ),
    }
}

/// Parses a size such as 200G for `--free`.
fn parse_size(text: &str) -> Result<u64, String> {
    mori_core::disk::parse_size(text)
}

/// The app for this process: its environment and clock, jj and git on `PATH`, and `gh` (or `$MORI_GH`,
/// which lets tests stand in for it).
fn real_app() -> App<Routed<JjCli, GitCli>, GhCli> {
    App::new(
        Host::from_process(),
        Routed {
            jj: JjCli::from_path(),
            git: GitCli::from_path(),
        },
        std::env::var_os("MORI_GH").map_or_else(GhCli::from_path, GhCli::new),
    )
}

/// Runs `doctor` and prints its report; problems found make the exit code non-zero.
fn run_doctor(
    app: &App<Routed<JjCli, GitCli>, GhCli>,
    json: bool,
    repo: Option<&str>,
    fix: Option<doctor::Fix>,
) -> ExitCode {
    let result = doctor::run(app, repo, fix);
    let problems = result.as_ref().is_ok_and(output::doctor_has_problems);
    let code = respond(json, result, output::doctor_text);
    if problems {
        output::doctor_problems_exit()
    } else {
        code
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

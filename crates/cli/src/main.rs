//! The `mori` command.
//!
//! Every failure is a `google.rpc.Status`, and the exit code is its canonical code's number: 0 for
//! success, 3 for bad arguments, 9 for a failed precondition, and so on.

mod look;
mod output;

use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use mori_api::v1alpha1::Hint;
use mori_app::routed::Routed;
use mori_app::{
    App, Host, clone, doctor, gc, init, ls, place, restore, skills, tree, tree_remove, tree_set,
};
use mori_core::error::ErrorDetails;
use mori_core::tree::Lifetime;
use mori_core::vcs::VcsKind;
use mori_git::GitCli;
use mori_github::GhCli;
use mori_jj::JjCli;

/// mori looks after a forest of repos and worktrees, for you and your agents.
#[derive(Debug, Parser)]
#[command(name = "mori", version, after_help = ENVIRONMENT)]
struct Cli {
    /// Print one JSON object on stdout instead of text, for errors too.
    #[arg(long, global = true)]
    json: bool,

    /// When to use colour: auto (on a terminal, unless `NO_COLOR` is set), always or never.
    #[arg(long, global = true, value_enum, default_value_t, value_name = "WHEN")]
    color: look::ColorWhen,

    /// With no command, on a terminal: the forest at a glance.
    #[command(subcommand)]
    command: Option<Command>,
}

/// The environment section of `mori --help`.
const ENVIRONMENT: &str = "\
Environment:
  MORI_ROOT    The root mori manages (default ~/mori)
  MORI_AGENT   Who new trees are for, when --agent isn't given (default: your login name)
  MORI_GH      The gh program to ask about pull requests (default: gh on PATH)
  MORI_HINTS   0 turns off hints, the one-line suggestions for a next step
  NO_COLOR     Plain text output, whatever the terminal

Every command takes --json for one JSON object on stdout. Guide: https://nevzheng.github.io/mori/";

#[derive(Debug, Subcommand)]
enum Command {
    /// Set up mori's root, config, database and agent skills.
    ///
    /// Only adds things; safe to run again.
    #[command(after_help = "Examples:\n  mori init\n  mori init --dry-run")]
    Init {
        /// Show what would be created, and create nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Clone a repo into the root and record it.
    ///
    /// The clone goes to repos/<host>/<owner>/<repo>. Refuses a repo mori already has, and any
    /// directory it didn't make.
    #[command(
        after_help = "Examples:\n  mori clone github.com/acme/widget\n  mori clone --vcs git git@github.com:acme/widget.git\n  mori clone --dry-run https://github.com/acme/widget"
    )]
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
        #[arg(long, value_name = "jj|git")]
        vcs: Option<VcsKind>,
    },

    /// Find the trees that can go, and remove them safely.
    ///
    /// Reports which trees may be removed and whether each is safe to remove. With --apply --yes,
    /// also removes a batch of them: each is checked again first, and each removal is journalled,
    /// so `mori restore` can undo it.
    #[command(
        after_help = "Examples:\n  mori gc                  # the report\n  mori gc --dry-run        # what --apply would remove\n  mori gc --apply --yes    # remove what can go"
    )]
    Gc {
        /// Only this repo: its full name, any form `mori clone` accepts, or a unique short name.
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

        /// With --apply or --dry-run: only these trees (repeat for more).
        #[arg(long = "only", value_name = "NAME")]
        names: Vec<String>,

        /// With --apply or --dry-run: the most trees to remove (default 10).
        #[arg(long)]
        max: Option<u32>,

        /// Show what --apply would remove, checking each tree again, and remove nothing.
        #[arg(long)]
        dry_run: bool,

        /// Free at least this much, e.g. 200G: pick trees that are safe to remove, least
        /// recently changed first. Shows the plan; with --apply --yes, removes them.
        #[arg(long, value_name = "SIZE", value_parser = parse_size, conflicts_with = "names")]
        free: Option<u64>,
    },

    /// List the repos and their trees.
    ///
    /// For each tree: owner, task, lifetime, and work that exists only on this machine. Reads only;
    /// never snapshots a working copy.
    #[command(
        after_help = "Examples:\n  mori ls\n  mori ls widget\n  mori ls --owner codex --status unsaved\n  mori ls --query oauth --json"
    )]
    Ls {
        /// Only this repo: its full name, any form `mori clone` accepts, or a unique short name.
        repo: Option<String>,

        /// Show how much disk each tree uses. Walks every file, so it can take a while; sizes
        /// measured in the last 15 minutes are reused.
        #[arg(long)]
        size: bool,

        /// With --size: measure every tree again instead of reusing recent sizes.
        #[arg(long, requires = "size")]
        fresh: bool,

        /// Only trees whose repo, name, task or purpose contains this text (any case).
        #[arg(long, value_name = "TEXT")]
        query: Option<String>,

        /// Only trees with this owner.
        #[arg(long, value_name = "OWNER")]
        owner: Option<String>,

        /// Only trees in this state.
        #[arg(long, value_enum, value_name = "STATE")]
        status: Option<StatusArg>,
    },

    /// Find drift between mori, the VCS and the disk, and fix what is safe.
    ///
    /// Checks the root, every clone and every tree, and lists each problem with the command that
    /// fixes it. Reads only, unless --fix --yes repairs the safe ones (removals are journalled for
    /// `mori restore`). Findings don't change the exit code.
    #[command(
        after_help = "Examples:\n  mori doctor\n  mori doctor --dry-run      # what --fix would repair\n  mori doctor --fix --yes"
    )]
    Doctor {
        /// Only this repo: its full name, any form `mori clone` accepts, or a unique short name.
        repo: Option<String>,

        /// Repair the auto-fixable findings (needs --yes).
        #[arg(long)]
        fix: bool,

        /// Confirm the repairs.
        #[arg(long, requires = "fix")]
        yes: bool,

        /// Show what --fix would repair, and repair nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Say where a path is in the forest: repo, tree, owner, task, purpose, context folder.
    ///
    /// Reads mori's records and the root layout only, so it is instant and works offline. Agents:
    /// run it first when you start in a directory.
    #[command(
        after_help = "Examples:\n  mori where\n  mori where ~/mori/trees/widget/claude-auth/src\n  mori where --json"
    )]
    Where {
        /// The path to look up. Defaults to the current directory.
        path: Option<std::path::PathBuf>,
    },

    /// Bring back a removed tree.
    ///
    /// From its journal entry (given by `mori gc --apply` or `mori doctor --fix`): on its pinned
    /// commit, at its old path, under its old record.
    #[command(after_help = "Examples:\n  mori restore j-1790000000-0a1b0002")]
    Restore {
        /// The journal entry, from `mori gc --apply`.
        entry: String,
    },

    /// Manage the agent skills in the root.
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },

    /// Print shell completions.
    ///
    /// For example: `mori completions zsh > ~/.zfunc/_mori`.
    Completions {
        /// bash, zsh, fish, elvish or powershell.
        shell: clap_complete::Shell,
    },

    /// Print mori's man page (roff), e.g. `mori man | man -l -`.
    #[command(hide = true)]
    Man,

    /// Create and remove trees: a jj workspace or git worktree per task.
    Tree {
        #[command(subcommand)]
        command: TreeCommand,
    },
}

#[derive(Debug, Subcommand)]
enum SkillsCommand {
    /// Update mori's skills and the llms.txt indexes to this version.
    ///
    /// Changes only files mori wrote and nobody edited; never touches anyone else's skills.
    #[command(after_help = "Examples:\n  mori skills sync\n  mori skills sync --dry-run")]
    Sync {
        /// Show what would change, and write nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum TreeCommand {
    /// Give one task its own tree.
    ///
    /// The tree goes under trees/<repo>/, on top of trunk, recorded with its owner, task and
    /// lifetime. In a jj clone it is a jj workspace; in a git clone, a detached git worktree.
    #[command(
        after_help = "Examples:\n  mori tree create widget --task fix-login --owner claude --purpose \"Fix the login redirect\"\n  mori tree create widget --task lead --lifetime pinned\n  mori tree create widget --task spike --lifetime ttl:3d --from main"
    )]
    Create {
        /// The repo: its full name (github.com/acme/widget), or a unique short name (widget).
        repo: String,

        /// A short slug for the work: lowercase letters, digits and hyphens.
        #[arg(long, value_name = "SLUG")]
        task: String,

        /// Who the tree is for, e.g. claude. Defaults to `$MORI_AGENT`, then your login name.
        #[arg(long = "owner", alias = "agent", value_name = "OWNER")]
        agent: Option<String>,

        /// What the tree is for, in one line (at most 200 characters).
        #[arg(long, value_name = "TEXT")]
        purpose: Option<String>,

        /// When the tree may go: pinned, task-done, lru, or ttl:<n>d. Defaults to the
        /// [trees.lifetime] task setting (task-done).
        #[arg(long, value_name = "LIFETIME")]
        lifetime: Option<Lifetime>,

        /// The revision to start from. Defaults to trunk (`trunk()`; `origin/HEAD` in git clones).
        #[arg(long, value_name = "REVISION")]
        from: Option<String>,

        /// Show what would happen, and create nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Change a tree's purpose, lifetime or owner.
    ///
    /// Only mori's record changes: never the tree, its files or the VCS. `--purpose ""` clears the
    /// purpose.
    #[command(
        after_help = "Examples:\n  mori tree set widget claude-auth --purpose \"OAuth device flow\"\n  mori tree set widget claude-lead --lifetime pinned\n  mori tree set widget claude-auth --owner codex"
    )]
    Set {
        /// The repo: its full name, or a unique short name.
        repo: String,

        /// The tree's name, e.g. claude-fix-login.
        name: String,

        /// What the tree is for, in one line; empty clears it.
        #[arg(long, value_name = "TEXT")]
        purpose: Option<String>,

        /// When the tree may go: pinned, task-done, lru, or ttl:<n>d.
        #[arg(long, value_name = "LIFETIME")]
        lifetime: Option<Lifetime>,

        /// Who the tree is for.
        #[arg(long, value_name = "OWNER")]
        owner: Option<String>,
    },

    /// Remove a tree whose work is safe elsewhere.
    ///
    /// Only when nothing in it exists only on this machine: no edits and no change missing from
    /// the remote. Never removes the clone itself or a workspace mori didn't make.
    #[command(
        after_help = "Examples:\n  mori tree remove widget claude-fix-login\n  mori tree remove widget claude-fix-login --dry-run"
    )]
    Remove {
        /// The repo: its full name, or a unique short name.
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
    look::init(cli.color, cli.json);
    let app = real_app();
    let Some(command) = cli.command else {
        return dashboard(&app, cli.json);
    };
    if matches!(command, Command::Init { .. } | Command::Ls { .. }) {
        skills::stale_hint(&app.host);
    }
    dispatch(&app, cli.json, command)
}

/// `mori` with no command: the forest at a glance on a terminal (or with `--json`, the same as
/// `mori ls --json`); elsewhere, the help, as a usage error.
fn dashboard(app: &App<Routed<JjCli, GitCli>, GhCli>, json: bool) -> ExitCode {
    use std::io::IsTerminal;
    if !json && !std::io::stdout().is_terminal() {
        let _ = Cli::command().print_help();
        return ExitCode::from(3);
    }
    respond_hinted(
        json,
        ls::run(app, &ls::LsArgs::default()),
        output::forest_text,
        |response| &response.hints,
    )
}

/// Runs the parsed command and prints its result.
fn dispatch(app: &App<Routed<JjCli, GitCli>, GhCli>, json: bool, command: Command) -> ExitCode {
    match command {
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
                    // --dry-run alone previews what --apply would do.
                    apply: (apply || dry_run).then_some(gc::Apply {
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
            // --dry-run alone previews what --fix would do.
            (fix || dry_run).then_some(doctor::Fix { yes, dry_run }),
        ),
        Command::Completions { shell } => print_completions(shell),
        Command::Man => print_man(),
        Command::Where { path } => {
            respond_hinted(json, run_where(app, path), output::where_text, |response| {
                &response.hints
            })
        }
        Command::Restore { entry } => {
            respond(json, restore::run(app, &entry), output::restore_text)
        }
        Command::Ls {
            repo,
            size,
            fresh,
            query,
            owner,
            status,
        } => respond_hinted(
            json,
            ls::run(
                app,
                &ls::LsArgs {
                    repo,
                    sizes: size,
                    fresh,
                    query,
                    owner,
                    status: status.map(StatusArg::filter),
                },
            ),
            output::ls_text,
            |response| &response.hints,
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
            purpose,
            dry_run,
        } => respond_hinted(
            json,
            tree::create(
                app,
                tree::CreateArgs {
                    repo,
                    task,
                    agent,
                    lifetime,
                    from,
                    purpose,
                    dry_run,
                },
            ),
            output::tree_create_text,
            |response| &response.hints,
        ),
        TreeCommand::Set {
            repo,
            name,
            purpose,
            lifetime,
            owner,
        } => respond(
            json,
            tree_set::run(
                app,
                tree_set::SetArgs {
                    repo,
                    name,
                    purpose,
                    lifetime,
                    owner,
                },
            ),
            output::tree_set_text,
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

/// Runs `where` for `path`, made absolute against the current directory.
fn run_where(
    app: &App<Routed<JjCli, GitCli>, GhCli>,
    path: Option<std::path::PathBuf>,
) -> Result<mori_api::v1alpha1::ResolveResponse, Box<dyn ErrorDetails>> {
    let here = std::env::current_dir().unwrap_or_default();
    let path = path.map_or_else(|| here.clone(), |path| here.join(path));
    place::run(app, &path)
}

/// The states `ls --status` takes.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum StatusArg {
    /// Edits or unpushed changes.
    Unsaved,
    /// Recorded, present and clean.
    Clean,
    /// A bookmark it pushed has landed.
    Landed,
    /// Recorded, but the VCS has no such tree.
    Missing,
    /// In the VCS, but mori didn't make it.
    Foreign,
}

impl StatusArg {
    fn filter(self) -> ls::StatusFilter {
        match self {
            Self::Unsaved => ls::StatusFilter::Unsaved,
            Self::Clean => ls::StatusFilter::Clean,
            Self::Landed => ls::StatusFilter::Landed,
            Self::Missing => ls::StatusFilter::Missing,
            Self::Foreign => ls::StatusFilter::Foreign,
        }
    }
}

/// Prints shell completions for `shell`.
fn print_completions(shell: clap_complete::Shell) -> ExitCode {
    clap_complete::generate(shell, &mut Cli::command(), "mori", &mut std::io::stdout());
    ExitCode::SUCCESS
}

/// Prints the man page, generated from the same definitions as `--help`.
fn print_man() -> ExitCode {
    match clap_mangen::Man::new(Cli::command()).render(&mut std::io::stdout()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
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

/// Runs `doctor` and prints its report. Findings are its output, not a failure: it exits 0
/// unless the check itself couldn't run.
fn run_doctor(
    app: &App<Routed<JjCli, GitCli>, GhCli>,
    json: bool,
    repo: Option<&str>,
    fix: Option<doctor::Fix>,
) -> ExitCode {
    respond(json, doctor::run(app, repo, fix), output::doctor_text)
}

/// Prints a command's result: its response as JSON or text, or its error.
/// As [`respond`], then the response's hints on stderr, for people; JSON carries them in the response.
fn respond_hinted<T: serde::Serialize>(
    json: bool,
    result: Result<T, Box<dyn ErrorDetails>>,
    text: fn(&T) -> String,
    hints: fn(&T) -> &[Hint],
) -> ExitCode {
    match result {
        Ok(response) => {
            let code = output::success(json, &response, &text(&response));
            if !json {
                output::print_hints(hints(&response));
            }
            code
        }
        Err(error) => output::failure(json, error.as_ref()),
    }
}

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

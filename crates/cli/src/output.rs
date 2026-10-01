//! What mori prints: text for people, or one JSON object on stdout with `--json`.
//!
//! Errors follow AIP-193. With `--json` they are a `google.rpc.Status` in its proto3 JSON form,
//! carrying one `google.rpc.ErrorInfo`; as text they go to stderr.

use std::fmt::Write as _;
use std::io::Write;
use std::process::ExitCode;

use mori_api::v1alpha1::{
    CloneResponse, CreateTreeResponse, DoctorResponse, GcResponse, Hint, InitResponse,
    ListTreesResponse, RemoveTreeResponse, RepoTrees, ResolveResponse, RestoreResponse,
    SyncSkillsResponse, Tree, TreeRow, Vcs, finding::Severity, skill_file::Action,
    tree_row::Status,
};
use mori_core::disk::{Space, format_size};
use mori_core::error::{Code, ErrorDetails};

use crate::look::{self, Role};
use serde_json::{Map, Value, json};

/// `path` with the home directory shown as `~`, for text output; JSON keeps paths absolute.
fn tilde(path: &str) -> String {
    let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) else {
        return path.to_owned();
    };
    let home = home.to_string_lossy();
    let home = home.trim_end_matches('/');
    match path.strip_prefix(home) {
        Some("") => "~".to_owned(),
        Some(rest) if rest.starts_with('/') => format!("~{rest}"),
        _ => path.to_owned(),
    }
}

/// The domain of errors from the command line itself.
const CLI_DOMAIN: &str = "cli.mori";

/// Prints a successful response and exits 0.
pub fn success(json: bool, response: &impl serde::Serialize, text: &str) -> ExitCode {
    let printed = if json {
        match serde_json::to_string(response) {
            Ok(line) => line + "\n",
            Err(error) => {
                let message = format!("error: can't encode the response as JSON: {error}\n");
                return print_or_fail(
                    &mut std::io::stderr().lock(),
                    &message,
                    exit(Code::Internal),
                );
            }
        }
    } else {
        text.to_owned()
    };
    print_or_fail(&mut std::io::stdout().lock(), &printed, ExitCode::SUCCESS)
}

/// Prints a failure and exits with its code.
pub fn failure(json: bool, error: &dyn ErrorDetails) -> ExitCode {
    let metadata = error
        .metadata()
        .into_iter()
        .map(|(key, value)| (key.to_owned(), Value::String(value)))
        .collect();
    report(
        json,
        error.code(),
        &error.to_string(),
        error.hint().as_deref(),
        error.reason(),
        error.domain(),
        metadata,
    )
}

/// Reports bad arguments as `INVALID_ARGUMENT`. `--help` and `--version` aren't errors.
pub fn usage_error(error: &clap::Error) -> ExitCode {
    if !error.use_stderr() {
        return match error.print() {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => exit(Code::Internal),
        };
    }
    // Parsing failed, so look for `--json` by hand.
    if std::env::args_os().any(|arg| arg == "--json") {
        let rendered = error.to_string();
        let first_line = rendered.lines().next().unwrap_or_default();
        return report(
            true,
            Code::InvalidArgument,
            first_line.trim_start_matches("error: "),
            None,
            "INVALID_USAGE",
            CLI_DOMAIN,
            Map::new(),
        );
    }
    // Best effort: if stderr is gone, there is nowhere left to report to.
    let _ = error.print();
    exit(Code::InvalidArgument)
}

/// The text `init` prints.
pub fn init_text(response: &InitResponse) -> String {
    let mut text = String::new();
    let root = tilde(&response.root);
    let skills = response
        .created
        .iter()
        .filter(|created| created.path.ends_with("/SKILL.md"))
        .count();
    // Writing to a String can't fail.
    let _ = match (response.already_initialized, response.validate_only) {
        (true, _) => writeln!(text, "mori is already set up at {root}"),
        (false, true) => writeln!(
            text,
            "Would set up mori at {root} (dry run). It would create:"
        ),
        (false, false) => writeln!(
            text,
            "Set up mori at {root}: {} new files and directories, {skills} skills.",
            response.created.len()
        ),
    };
    // A dry run is a preview, so it lists everything; a real run sums it up (`--json` lists it).
    if response.validate_only {
        for created in &response.created {
            let _ = writeln!(text, "  {}", tilde(&created.path));
        }
    }
    if !response.unmanaged_repos.is_empty() {
        let _ = writeln!(text, "Clones mori didn't make (left alone):");
        for repo in &response.unmanaged_repos {
            let _ = writeln!(text, "  {}  {}", repo.repo, tilde(&repo.path));
        }
    }
    if !response.validate_only && !response.already_initialized {
        let _ = writeln!(
            text,
            "Next: `mori clone <repo>`, and point your agent at {root}/llms.txt."
        );
    }
    text
}

/// The text `clone` prints.
pub fn clone_text(response: &CloneResponse) -> String {
    let mut text = String::new();
    let (repo, path) = (&response.repo, tilde(&response.path));
    let kind = if response.vcs() == Vcs::Git {
        "git; trees are git worktrees"
    } else if response.colocated {
        "jj, colocated with git"
    } else {
        "jj only"
    };
    // Writing to a String can't fail.
    let _ = if response.validate_only {
        writeln!(text, "Would clone {repo} into {path} (dry run):")
    } else {
        writeln!(text, "Cloned {repo} into {path}:")
    };
    let _ = writeln!(text, "  from {} ({kind})", response.fetch_url);
    let _ = writeln!(text, "  base tree: default (pinned; the clone itself)");
    let _ = writeln!(text, "  task trees: trees/{}/", response.tree_dir);
    let _ = writeln!(text, "  context: {}/", tilde(&response.context_dir));
    warnings_text(&mut text, &response.warnings);
    text
}

/// The text `tree create` prints.
pub fn tree_create_text(response: &CreateTreeResponse) -> String {
    let mut text = String::new();
    let Some(tree) = &response.tree else {
        return text;
    };
    // Writing to a String can't fail.
    let _ = if response.validate_only {
        writeln!(text, "Would create tree {} (dry run):", tree.name)
    } else {
        writeln!(text, "Created tree {}:", tree.name)
    };
    let _ = writeln!(text, "  path: {}", tilde(&tree.path));
    let _ = writeln!(text, "  repo: {}", tree.repo);
    let _ = writeln!(text, "  task: {} (owner {})", tree.task, tree.owner);
    if !tree.purpose.is_empty() {
        let _ = writeln!(text, "  purpose: {}", tree.purpose);
    }
    let _ = writeln!(text, "  lifetime: {}", tree.lifetime);
    let _ = writeln!(text, "  starts from: {}", response.from);
    let hint = if response.vcs() == Vcs::Git {
        "  a git worktree on a detached HEAD: commit, then push with \
         `git push origin HEAD:refs/heads/<branch>` (see the vcs-in-mori skill)"
    } else {
        "  a jj workspace: use jj here, not git (see the vcs-in-mori skill)"
    };
    let _ = writeln!(text, "{hint}");
    warnings_text(&mut text, &response.warnings);
    text
}

/// Prints each hint on stderr, one line each: what mori noticed, then the command for it.
pub fn print_hints(hints: &[Hint]) {
    let mut text = String::new();
    for hint in hints {
        let command = if hint.command.is_empty() {
            String::new()
        } else {
            format!(": `{}`", hint.command)
        };
        // Writing to a String can't fail.
        let _ = writeln!(
            text,
            "{} {}{command}",
            look::paint_err(Role::Warn, "hint:"),
            hint.message
        );
    }
    // A hint is only advice: failing to print one is no reason to fail the command.
    let _ = std::io::Write::write_all(&mut std::io::stderr().lock(), text.as_bytes());
}

/// Appends each warning on its own line.
fn warnings_text(text: &mut String, warnings: &[String]) {
    for warning in warnings {
        let _ = writeln!(text, "warning: {warning}");
    }
}

/// The text `where` prints.
pub fn where_text(response: &ResolveResponse) -> String {
    use mori_api::v1alpha1::resolve_response::Kind;
    let mut text = String::new();
    let root = tilde(&response.root);
    if response.repo.is_empty() {
        let what = match response.kind() {
            Kind::Context => "the context folder",
            Kind::Clone | Kind::Tree => "a directory mori has no record of",
            Kind::Root | Kind::Unspecified => "the mori root",
        };
        let _ = writeln!(text, "{what}, in {root}");
        return text;
    }
    let tree = response.tree.clone().unwrap_or_default();
    let place = match response.kind() {
        Kind::Clone => format!("clone (base tree {})", tree.name),
        Kind::Tree if response.status() == Status::Foreign => format!("foreign tree {}", tree.name),
        Kind::Tree => format!("tree {}", tree.name),
        Kind::Context => "context folder".to_owned(),
        Kind::Root | Kind::Unspecified => String::new(),
    };
    let _ = writeln!(
        text,
        "{}  {}",
        look::paint(Role::Name, &response.repo),
        place
    );
    if response.tree.is_some() {
        if response.status() == Status::Tree {
            let task = if tree.task.is_empty() {
                "(base)"
            } else {
                &tree.task
            };
            let _ = writeln!(
                text,
                "  owner {} · task {task} · {}",
                tree.owner, tree.lifetime
            );
            if !tree.purpose.is_empty() {
                let _ = writeln!(text, "  purpose: {}", tree.purpose);
            }
        } else {
            let _ = writeln!(
                text,
                "  {}",
                look::paint(
                    Role::Quiet,
                    "foreign: mori didn't make this tree, and leaves it alone"
                )
            );
        }
        let _ = writeln!(
            text,
            "  path: {}",
            look::paint(Role::Quiet, &tilde(&tree.path))
        );
    }
    let _ = writeln!(
        text,
        "  context: {}/",
        look::paint(Role::Quiet, &tilde(&response.context_dir))
    );
    text
}

/// The text `tree set` prints: the tree's record as it is now.
pub fn tree_set_text(tree: &Tree) -> String {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "Updated tree {}:",
        look::paint(Role::Name, &tree.name)
    );
    let _ = writeln!(text, "  owner: {}", tree.owner);
    let _ = writeln!(text, "  lifetime: {}", tree.lifetime);
    let purpose = if tree.purpose.is_empty() {
        "-"
    } else {
        &tree.purpose
    };
    let _ = writeln!(text, "  purpose: {purpose}");
    text
}

/// One repo's table for `ls`: a header line, then a row per tree.
fn ls_repo(text: &mut String, repo: &RepoTrees, sized: bool, reused: &dyn Fn(&TreeRow) -> bool) {
    let vcs = match repo.vcs() {
        Vcs::Git => "  git",
        Vcs::Jj => "  jj",
        Vcs::Unspecified => "",
    };
    let size = if sized {
        format!(
            "  {}",
            format_size(repo.trees.iter().map(|row| row.size_bytes).sum())
        )
    } else {
        String::new()
    };
    let _ = writeln!(
        text,
        "{}{vcs}{size}  {}",
        look::paint(Role::Name, &repo.repo),
        look::paint(Role::Quiet, &tilde(&repo.path))
    );
    let mut header: Vec<String> = ["NAME", "STATUS", "OWNER", "TASK", "LIFETIME", "WORK"]
        .map(str::to_owned)
        .to_vec();
    if sized {
        header.push("SIZE".to_owned());
    }
    let mut cells: Vec<Vec<String>> = repo
        .trees
        .iter()
        .map(|row| {
            let mut cells = ls_row(row).to_vec();
            if sized {
                cells.push(size_cell(row, reused(row)));
            }
            cells
        })
        .collect();
    // STATUS only matters when something is off.
    if cells.iter().all(|row| row[1] == "tree") {
        header.remove(1);
        for row in &mut cells {
            row.remove(1);
        }
    }
    // PURPOSE only when some tree has one, cut to fit a line.
    let purposes: Vec<&str> = repo
        .trees
        .iter()
        .map(|row| row.tree.as_ref().map_or("", |tree| tree.purpose.as_str()))
        .collect();
    if purposes.iter().any(|purpose| !purpose.is_empty()) {
        header.push("PURPOSE".to_owned());
        for (row, purpose) in cells.iter_mut().zip(purposes) {
            row.push(if purpose.is_empty() {
                "-".to_owned()
            } else {
                shorten(purpose, 50)
            });
        }
    }
    let work = header.iter().position(|column| column == "WORK");
    table_with(text, &header, &cells, |column, cell| match (column, cell) {
        (0, _) => Some(Role::Name),
        (_, "missing") => Some(Role::Problem),
        (_, "foreign") => Some(Role::Quiet),
        (c, "clean") if Some(c) == work => Some(Role::Ok),
        (c, "-") if Some(c) == work => Some(Role::Quiet),
        (c, _) if Some(c) == work => Some(Role::Warn),
        _ => None,
    });
    let _ = writeln!(text);
}

/// The text `ls` prints: a table of trees per repo, then the clones mori didn't make.
pub fn ls_text(response: &ListTreesResponse) -> String {
    let mut text = String::new();
    if response.repos.is_empty() {
        let _ = writeln!(
            text,
            "mori manages no repos yet; add one with `mori clone`."
        );
    }
    let rows = || response.repos.iter().flat_map(|repo| &repo.trees);
    let sized = rows().any(|row| !row.size_measured_at.is_empty());
    // RFC 3339 in UTC sorts as text: anything older than the newest was reused.
    let newest = rows()
        .map(|row| row.size_measured_at.as_str())
        .max()
        .unwrap_or_default();
    let reused =
        |row: &TreeRow| !row.size_measured_at.is_empty() && row.size_measured_at.as_str() < newest;
    for repo in &response.repos {
        ls_repo(&mut text, repo, sized, &reused);
    }
    if !response.unmanaged_repos.is_empty() {
        let _ = writeln!(text, "Clones mori didn't make (left alone):");
        for repo in &response.unmanaged_repos {
            let _ = writeln!(text, "  {}  {}", repo.repo, tilde(&repo.path));
        }
        let _ = writeln!(text);
    }
    // git reads working trees live; jj's view is as of its last snapshot, and a described
    // working-copy change reads as edited until it is finished.
    if response.repos.iter().any(|repo| repo.vcs() != Vcs::Git) {
        let _ = writeln!(
            text,
            "WORK in jj trees is as of jj's last snapshot. \"edited\" means the working copy\n\
             has changes: `jj commit` (or `jj new`) finishes them."
        );
    }
    if rows().any(|row| row.size_partial) {
        let _ = writeln!(text, "+ some files couldn't be read, so the size is short.");
    }
    if rows().any(reused) {
        let _ = writeln!(
            text,
            "* a size measured in the last 15 minutes, reused; `--fresh` measures again."
        );
    }
    if let Some(disk) = &response.disk {
        let used = if sized {
            format!(
                "Trees use {}; ",
                format_size(rows().map(|row| row.size_bytes).sum())
            )
        } else {
            String::new()
        };
        let _ = writeln!(
            text,
            "{used}{}.",
            space_text(disk.free_bytes, disk.total_bytes)
        );
    }
    warnings_text(&mut text, &response.warnings);
    text
}

/// "310G free of 3.9T (8%)".
fn space_text(free: u64, total: u64) -> String {
    let space = Space { free, total };
    format!(
        "{} free of {} ({}%)",
        format_size(space.free),
        format_size(space.total),
        space.free_percent()
    )
}

/// Appends an indented table with aligned columns, under `header` unless it is empty.
fn table(text: &mut String, header: &[String], rows: &[Vec<String>]) {
    table_with(text, header, rows, |_, _| None);
}

/// Columns padded to their widest cell, with a grey header and each cell painted in the role
/// `role_of(column, cell)` gives. Padding comes first, so colour codes never upset the columns.
fn table_with(
    text: &mut String,
    header: &[String],
    rows: &[Vec<String>],
    role_of: impl Fn(usize, &str) -> Option<Role>,
) {
    let mut widths: Vec<usize> = header.iter().map(String::len).collect();
    for row in rows {
        widths.resize(widths.len().max(row.len()), 0);
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.len());
        }
    }
    let line = |row: &[String], paint: &dyn Fn(usize, &str, String) -> String| {
        let last = row.len().saturating_sub(1);
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .enumerate()
            .map(|(column, (cell, width))| {
                let padded = if column == last {
                    cell.clone()
                } else {
                    format!("{cell:width$}")
                };
                paint(column, cell, padded)
            })
            .collect();
        cells.join("  ").trim_end().to_owned()
    };
    if !header.is_empty() {
        let plain = line(header, &|_, _, padded| padded);
        let _ = writeln!(text, "  {}", look::paint(Role::Quiet, &plain));
    }
    for row in rows {
        let painted = line(row, &|column, cell, padded| match role_of(column, cell) {
            Some(role) => look::paint(role, &padded),
            None => padded,
        });
        let _ = writeln!(text, "  {painted}");
    }
}

/// A tree's size, marked when it was reused, or `-` when it wasn't measured.
fn size_cell(row: &TreeRow, reused: bool) -> String {
    if row.size_measured_at.is_empty() {
        return "-".to_owned();
    }
    let mut cell = format_size(row.size_bytes);
    if row.size_partial {
        cell.push('+');
    }
    if reused {
        cell.push('*');
    }
    cell
}

fn ls_row(row: &TreeRow) -> [String; 6] {
    let tree = row.tree.clone().unwrap_or_default();
    let or_dash = |value: String| {
        if value.is_empty() {
            "-".to_owned()
        } else {
            value
        }
    };
    let status = match row.status() {
        Status::Tree | Status::Unspecified => "tree",
        Status::Missing => "missing",
        Status::Foreign => "foreign",
    };
    let work = match &row.state {
        None => "-".to_owned(),
        Some(state) => {
            let mut parts = Vec::new();
            if state.changed {
                parts.push("edited".to_owned());
            }
            if state.unpushed > 0 {
                parts.push(format!("{} unpushed", state.unpushed));
            }
            if parts.is_empty() {
                "clean".to_owned()
            } else {
                parts.join(", ")
            }
        }
    };
    [
        tree.name,
        status.to_owned(),
        or_dash(tree.owner),
        or_dash(tree.task),
        or_dash(tree.lifetime),
        work,
    ]
}

/// The forest at a glance, for `mori` with no command: each repo with its trees as branches, and
/// a mark for whether each tree's work is safe.
pub fn forest_text(response: &ListTreesResponse) -> String {
    let mut text = String::new();
    let root = response
        .repos
        .first()
        .and_then(|repo| repo.path.split("/repos/").next())
        .map_or_else(|| "~/mori".to_owned(), tilde);
    let mark = if look::glyphs() { " 森" } else { "" };
    let _ = writeln!(
        text,
        "{}{mark}  {}",
        look::paint(Role::Name, "mori"),
        look::paint(Role::Quiet, &root)
    );
    if response.repos.is_empty() {
        let _ = writeln!(text, "No repos yet: `mori clone <repo>` adds one.");
        return text;
    }
    for (index, repo) in response.repos.iter().enumerate() {
        let last_repo = index + 1 == response.repos.len();
        let vcs = match repo.vcs() {
            Vcs::Git => "git",
            Vcs::Jj => "jj",
            Vcs::Unspecified => "",
        };
        let _ = writeln!(
            text,
            "{} {}  {}",
            look::branch(last_repo),
            look::paint(Role::Name, &repo.repo),
            look::paint(Role::Quiet, vcs)
        );
        let width = repo
            .trees
            .iter()
            .filter_map(|row| row.tree.as_ref().map(|tree| tree.name.len()))
            .max()
            .unwrap_or(0);
        let rows: Vec<[String; 6]> = repo.trees.iter().map(ls_row).collect();
        let state_width = rows
            .iter()
            .map(|[_, status, _, _, _, work]| {
                let what = if status == "tree" {
                    work.len()
                } else {
                    status.len() + 2 + work.len()
                };
                look::mark(Role::Warn).chars().count().max(2) + 1 + what
            })
            .max()
            .unwrap_or(0);
        for (tree_index, row) in rows.into_iter().enumerate() {
            let last_tree = tree_index + 1 == repo.trees.len();
            let [name, status, owner, task, lifetime, work] = row;
            let role = match (status.as_str(), work.as_str()) {
                ("missing", _) => Role::Problem,
                ("foreign", _) => Role::Quiet,
                (_, "clean") => Role::Ok,
                _ => Role::Warn,
            };
            let what = if status == "tree" {
                work
            } else {
                format!("{status}, {work}")
            };
            let task = if task == "-" {
                "(base)".to_owned()
            } else {
                task
            };
            let purpose = row_purpose(&repo.trees[tree_index]);
            let _ = writeln!(
                text,
                "{} {} {}  {}  {owner} · {task} · {lifetime}{purpose}",
                look::trunk(last_repo),
                look::branch(last_tree),
                look::paint(Role::Name, &format!("{name:width$}")),
                look::paint(
                    role,
                    &pad(&format!("{} {what}", look::mark(role)), state_width)
                ),
            );
        }
    }
    let _ = writeln!(
        text,
        "{}",
        look::paint(
            Role::Quiet,
            "`mori ls` for the table, `mori doctor` to check, `mori gc` to clean up."
        )
    );
    text
}

/// " — <purpose>" for the forest view, or nothing when the tree has none.
fn row_purpose(row: &TreeRow) -> String {
    match row.tree.as_ref().map(|tree| tree.purpose.as_str()) {
        Some(purpose) if !purpose.is_empty() => {
            let dash = if look::glyphs() { "—" } else { "-" };
            format!("  {dash} {}", shorten(purpose, 60))
        }
        _ => String::new(),
    }
}

/// `text` cut to `max` characters, with `…` (or `...` without glyphs) when cut.
fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let ellipsis = if look::glyphs() { "…" } else { "..." };
    let keep = max.saturating_sub(ellipsis.chars().count());
    format!("{}{ellipsis}", text.chars().take(keep).collect::<String>())
}

/// `text` padded with spaces to `width` characters (not bytes: marks may be multi-byte).
fn pad(text: &str, width: usize) -> String {
    let len = text.chars().count();
    format!("{text}{}", " ".repeat(width.saturating_sub(len)))
}

/// The text `tree remove` prints.
pub fn tree_remove_text(response: &RemoveTreeResponse) -> String {
    let mut text = String::new();
    let Some(tree) = &response.tree else {
        return text;
    };
    // Writing to a String can't fail.
    let _ = match (response.validate_only, response.directory_gone) {
        (true, false) => writeln!(
            text,
            "Would remove tree {} (dry run): it is safe to remove.",
            tree.name
        ),
        (true, true) => writeln!(
            text,
            "Would remove tree {} (dry run): its directory was deleted, so its last commit would \
             be pinned for `mori restore`.",
            tree.name
        ),
        (false, _) => writeln!(text, "Removed tree {}:", tree.name),
    };
    if !response.validate_only {
        if response.directory_gone {
            let _ = writeln!(
                text,
                "  its directory {} was already deleted; pinned its last commit",
                tilde(&tree.path)
            );
            let _ = writeln!(
                text,
                "  bring it back with `mori restore {}`",
                response.journal_entry
            );
        } else if response.directory_removed {
            let _ = writeln!(text, "  deleted {}", tilde(&tree.path));
        } else {
            let _ = writeln!(
                text,
                "  its workspace was already gone; left {} in place",
                tilde(&tree.path)
            );
        }
        let _ = writeln!(text, "  dropped its record");
    }
    text
}

/// The text `skills sync` prints: one line per file mori ships, saying what happened to it.
pub fn skills_text(response: &SyncSkillsResponse) -> String {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "{}",
        if response.validate_only {
            "Skills (dry run):"
        } else {
            "Skills:"
        }
    );
    for file in &response.files {
        let what = match (file.action(), response.validate_only) {
            (Action::Installed, false) => "installed",
            (Action::Installed, true) => "would install",
            (Action::Updated, false) => "updated",
            (Action::Updated, true) => "would update",
            (Action::KeptEdited, _) => "kept (edited since mori wrote it)",
            (Action::SkippedNotOurs, _) => "left alone (mori didn't write it)",
            (Action::Unchanged | Action::Unspecified, _) => "up to date",
        };
        let _ = writeln!(text, "  {what}: {}", tilde(&file.path));
    }
    text
}

/// One tree's row in the `gc` text: name, size, reason, and what happened or why.
fn gc_row(item: &mori_api::v1alpha1::GcItem) -> Vec<String> {
    use mori_api::v1alpha1::gc_item::{Kind, Outcome};
    let what = match item.outcome() {
        Outcome::Removed => format!("removed; undo: mori restore {}", item.entry_id),
        Outcome::WouldRemove => "would remove".to_owned(),
        Outcome::SkippedUnsaved => "kept: it has work only this machine has".to_owned(),
        Outcome::SkippedChanged => "kept: changed since it was judged".to_owned(),
        Outcome::Unspecified => item.facts.clone(),
    };
    let size = if item.size_bytes > 0 {
        format_size(item.size_bytes)
    } else {
        "-".to_owned()
    };
    let name = if item.kind() == Kind::BazelLeftover {
        format!("{} {} (Bazel output)", item.repo, item.name)
            .trim_start()
            .to_owned()
    } else {
        format!("{} {}", item.repo, item.name)
    };
    vec![name, size, item.reason.clone(), what]
}

/// The text `gc` prints: the trees grouped by what can happen to them (with --apply, what did),
/// then what to do next. Trees that never go (clones, pinned trees) are only counted.
pub fn gc_text(response: &GcResponse) -> String {
    use mori_api::v1alpha1::gc_item::{Class, Outcome};
    let mut text = String::new();
    let applied = response.validate_only
        || response
            .items
            .iter()
            .any(|item| !matches!(item.outcome(), Outcome::Unspecified | Outcome::WouldRemove));
    let groups = [
        (Class::Remove, "Can go"),
        (Class::Blocked, "Blocked: work only this machine has"),
        (Class::Keep, "Not yet"),
    ];
    for (class, title) in groups {
        let rows: Vec<Vec<String>> = response
            .items
            .iter()
            .filter(|item| item.class() == class)
            .map(gc_row)
            .collect();
        if rows.is_empty() {
            continue;
        }
        let role = match class {
            Class::Remove => Role::Ok,
            Class::Blocked => Role::Warn,
            _ => Role::Quiet,
        };
        let _ = writeln!(
            text,
            "{}",
            look::paint(role, &format!("{title} ({})", rows.len()))
        );
        table(&mut text, &[], &rows);
    }
    let never = response
        .items
        .iter()
        .filter(|item| item.class() == Class::Never)
        .count();
    if never > 0 {
        let _ = writeln!(
            text,
            "{never} never go: clones, pinned trees and workspaces mori didn't make."
        );
    }
    let sum = |keep: &dyn Fn(&mori_api::v1alpha1::GcItem) -> bool| -> (usize, u64) {
        response
            .items
            .iter()
            .filter(|item| keep(item))
            .fold((0, 0), |(count, bytes), item| {
                (count + 1, bytes + item.size_bytes)
            })
    };
    let (removable, removable_bytes) = sum(&|item| item.class() == Class::Remove);
    let (picked, picked_bytes) = sum(&|item| item.outcome() == Outcome::WouldRemove);
    let _ = if applied {
        Ok(())
    } else if removable == 0 {
        writeln!(text, "Nothing to remove.")
    } else if picked > 0 {
        writeln!(
            text,
            "Picked {picked} of {removable} removable item(s), freeing {}: add `--apply --yes` \
             to remove them, checking each again first.",
            format_size(picked_bytes)
        )
    } else {
        writeln!(
            text,
            "{removable} item(s) can go, freeing {}: `mori gc --apply --yes` removes them, \
             checking each again first; `--free <size>` picks only enough to free that much.",
            format_size(removable_bytes)
        )
    };
    if let Some(disk) = &response.disk {
        let _ = writeln!(text, "{}.", space_text(disk.free_bytes, disk.total_bytes));
    }
    warnings_text(&mut text, &response.warnings);
    text
}

/// The text `restore` prints.
pub fn restore_text(response: &RestoreResponse) -> String {
    let mut text = String::new();
    if let Some(tree) = &response.tree {
        let _ = writeln!(text, "Restored tree {}:", tree.name);
        let _ = writeln!(text, "  path: {}", tilde(&tree.path));
        let _ = writeln!(text, "  on commit: {}", response.commit_id);
        let _ = writeln!(text, "  task: {} (owner {})", tree.task, tree.owner);
    }
    text
}

/// The text `doctor` prints: each finding, worst first, with its fix, then the summary.
pub fn doctor_text(response: &DoctorResponse) -> String {
    let mut text = String::new();
    let verb = if response.validate_only {
        "Would fix"
    } else {
        "Fixed"
    };
    for fixed in &response.fixed {
        let _ = writeln!(text, "{verb} {}  {}", fixed.code, fixed.subject);
        if !fixed.journal_entry.is_empty() {
            let _ = writeln!(text, "  undo: mori restore {}", fixed.journal_entry);
        }
    }
    if !response.fixed.is_empty() {
        let _ = writeln!(text);
    }
    for finding in &response.findings {
        let (severity, role) = match finding.severity() {
            Severity::Problem => ("problem", Role::Problem),
            Severity::Warn => ("warn", Role::Warn),
            Severity::Info | Severity::Unspecified => ("info", Role::Quiet),
        };
        let _ = writeln!(
            text,
            "{} {}  {}",
            look::paint(role, &format!("{severity:<8}")),
            finding.code,
            finding.subject
        );
        let _ = writeln!(text, "         {}", finding.message);
        let _ = writeln!(text, "         fix: {}", finding.fix);
    }
    let _ = writeln!(text, "{}", response.summary);
    text
}

fn report(
    json: bool,
    code: Code,
    message: &str,
    hint: Option<&str>,
    reason: &str,
    domain: &str,
    metadata: Map<String, Value>,
) -> ExitCode {
    if json {
        let mut info = json!({
            "@type": "type.googleapis.com/google.rpc.ErrorInfo",
            "reason": reason,
            "domain": domain,
        });
        if !metadata.is_empty() {
            info["metadata"] = Value::Object(metadata);
        }
        let status = json!({
            "code": code.number(),
            "message": message,
            "details": [info],
        });
        return print_or_fail(
            &mut std::io::stdout().lock(),
            &format!("{status}\n"),
            exit(code),
        );
    }
    // For people: what happened, what to do, then the reason for searching and scripts. The
    // details (metadata) are in the JSON form.
    let mut text = format!("{} {message}\n", look::paint_err(Role::Problem, "error:"));
    if let Some(hint) = hint {
        let _ = writeln!(text, "{} {hint}", look::paint_err(Role::Warn, "hint:"));
    }
    let detail = format!(
        "  {reason} ({domain}), {} exit {}",
        code.name(),
        code.number()
    );
    let _ = writeln!(text, "{}", look::paint_err(Role::Quiet, &detail));
    print_or_fail(&mut std::io::stderr().lock(), &text, exit(code))
}

fn print_or_fail(out: &mut impl Write, text: &str, code: ExitCode) -> ExitCode {
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Ok(()) => code,
        Err(_) => exit(Code::Internal),
    }
}

fn exit(code: Code) -> ExitCode {
    // Canonical codes are 1..=16, so they always fit.
    u8::try_from(code.number()).map_or(ExitCode::FAILURE, ExitCode::from)
}

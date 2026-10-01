//! What mori prints: text for people, or one JSON object on stdout with `--json`.
//!
//! Errors follow AIP-193. With `--json` they are a `google.rpc.Status` in its proto3 JSON form,
//! carrying one `google.rpc.ErrorInfo`; as text they go to stderr.

use std::fmt::Write as _;
use std::io::Write;
use std::process::ExitCode;

use mori_api::v1alpha1::{
    CloneResponse, CreateTreeResponse, DoctorResponse, GcResponse, InitResponse, ListTreesResponse,
    RemoveTreeResponse, RestoreResponse, SyncSkillsResponse, TreeRow, Vcs, finding::Severity,
    skill_file::Action, tree_row::Status,
};
use mori_core::disk::{Space, format_size};
use mori_core::error::{Code, ErrorDetails};
use serde_json::{Map, Value, json};

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
    let root = &response.root;
    // Writing to a String can't fail.
    let _ = match (response.already_initialized, response.validate_only) {
        (true, _) => writeln!(text, "mori is already set up at {root}"),
        (false, true) => writeln!(text, "Would set up mori at {root} (dry run):"),
        (false, false) => writeln!(text, "Set up mori at {root}:"),
    };
    let verb = if response.validate_only {
        "would create"
    } else {
        "created"
    };
    for created in &response.created {
        let _ = writeln!(text, "  {verb} {}", created.path);
    }
    if !response.unmanaged_repos.is_empty() {
        let _ = writeln!(text, "Clones mori didn't make (left alone):");
        for repo in &response.unmanaged_repos {
            let _ = writeln!(text, "  {}  {}", repo.repo, repo.path);
        }
    }
    text
}

/// The text `clone` prints.
pub fn clone_text(response: &CloneResponse) -> String {
    let mut text = String::new();
    let (repo, path) = (&response.repo, &response.path);
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
    let _ = writeln!(text, "  context: {}/", response.context_dir);
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
    let _ = writeln!(text, "  path: {}", tree.path);
    let _ = writeln!(text, "  repo: {}", tree.repo);
    let _ = writeln!(text, "  task: {} (owner {})", tree.task, tree.owner);
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

/// Appends each warning on its own line.
fn warnings_text(text: &mut String, warnings: &[String]) {
    for warning in warnings {
        let _ = writeln!(text, "warning: {warning}");
    }
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
        let vcs = match repo.vcs() {
            Vcs::Git => " (git)",
            Vcs::Jj => " (jj)",
            Vcs::Unspecified => "",
        };
        let size = if sized {
            format!(
                " {}",
                format_size(repo.trees.iter().map(|row| row.size_bytes).sum())
            )
        } else {
            String::new()
        };
        let _ = writeln!(text, "{}{vcs}{size}  {}", repo.repo, repo.path);
        let mut header: Vec<String> = ["NAME", "STATUS", "OWNER", "TASK", "LIFETIME", "WORK"]
            .map(str::to_owned)
            .to_vec();
        if sized {
            header.push("SIZE".to_owned());
        }
        let cells: Vec<Vec<String>> = repo
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
        table(&mut text, &header, &cells);
        let _ = writeln!(text);
    }
    if !response.unmanaged_repos.is_empty() {
        let _ = writeln!(text, "Clones mori didn't make (left alone):");
        for repo in &response.unmanaged_repos {
            let _ = writeln!(text, "  {}  {}", repo.repo, repo.path);
        }
        let _ = writeln!(text);
    }
    // git reads working trees live; jj's view is as of its last snapshot.
    if response.repos.iter().any(|repo| repo.vcs() != Vcs::Git) {
        let _ = writeln!(
            text,
            "WORK in jj trees is as of jj's last snapshot in each tree."
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
    let mut widths: Vec<usize> = header.iter().map(String::len).collect();
    for row in rows {
        widths.resize(widths.len().max(row.len()), 0);
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.len());
        }
    }
    let header = (!header.is_empty()).then_some(header);
    for row in header.into_iter().chain(rows.iter().map(Vec::as_slice)) {
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(cell, width)| format!("{cell:width$}"))
            .collect();
        let _ = writeln!(text, "  {}", cells.join("  ").trim_end());
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
                tree.path
            );
            let _ = writeln!(
                text,
                "  bring it back with `mori restore {}`",
                response.journal_entry
            );
        } else if response.directory_removed {
            let _ = writeln!(text, "  deleted {}", tree.path);
        } else {
            let _ = writeln!(
                text,
                "  its workspace was already gone; left {} in place",
                tree.path
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
        let _ = writeln!(text, "  {what}: {}", file.path);
    }
    text
}

/// The text `gc` prints: every tree with its class and reason (and, with --apply, what happened
/// to it), then what to do next.
pub fn gc_text(response: &GcResponse) -> String {
    use mori_api::v1alpha1::gc_item::{Class, Kind, Outcome};
    let mut text = String::new();
    let applied = response.validate_only
        || response
            .items
            .iter()
            .any(|item| !matches!(item.outcome(), Outcome::Unspecified | Outcome::WouldRemove));
    let _ = writeln!(text, "Cleanup:");
    let rows: Vec<Vec<String>> = response
        .items
        .iter()
        .map(|item| {
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
            vec![
                mori_app::gc::class_name(item.class()).to_owned(),
                if item.kind() == Kind::BazelLeftover {
                    format!("{} {} (Bazel output)", item.repo, item.name)
                        .trim_start()
                        .to_owned()
                } else {
                    format!("{} {}", item.repo, item.name)
                },
                size,
                item.reason.clone(),
                what,
            ]
        })
        .collect();
    table(&mut text, &[], &rows);
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
        let _ = writeln!(text, "  path: {}", tree.path);
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
        let severity = match finding.severity() {
            Severity::Problem => "problem",
            Severity::Warn => "warn",
            Severity::Info | Severity::Unspecified => "info",
        };
        let _ = writeln!(text, "{severity:<8} {}  {}", finding.code, finding.subject);
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
    let mut text = format!("error: {message}\n  status: {}\n", code.name());
    let _ = writeln!(text, "  reason: {reason} ({domain})");
    for (key, value) in &metadata {
        let _ = writeln!(text, "  {key}: {}", value.as_str().unwrap_or_default());
    }
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

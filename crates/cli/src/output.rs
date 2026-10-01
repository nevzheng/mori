//! What mori prints: text for people, or one JSON object on stdout with `--json`.
//!
//! Errors follow AIP-193. With `--json` they are a `google.rpc.Status` in its proto3 JSON form,
//! carrying one `google.rpc.ErrorInfo`; as text they go to stderr.

use std::fmt::Write as _;
use std::io::Write;
use std::process::ExitCode;

use mori_api::v1alpha1::{
    CloneResponse, CreateTreeResponse, GcResponse, InitResponse, ListTreesResponse,
    RemoveTreeResponse, RestoreResponse, SyncSkillsResponse, TreeRow, skill_file::Action,
    tree_row::Status,
};
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
    let kind = if response.colocated {
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
    text
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
    for repo in &response.repos {
        let _ = writeln!(text, "{}  {}", repo.repo, repo.path);
        let rows: Vec<[String; 6]> = repo.trees.iter().map(ls_row).collect();
        let header = ["NAME", "STATUS", "OWNER", "TASK", "LIFETIME", "WORK"].map(str::to_owned);
        let mut widths = header.clone().map(|cell| cell.len());
        for row in &rows {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(cell.len());
            }
        }
        for row in std::iter::once(&header).chain(&rows) {
            let cells: Vec<String> = row
                .iter()
                .zip(widths)
                .map(|(cell, width)| format!("{cell:width$}"))
                .collect();
            let _ = writeln!(text, "  {}", cells.join("  ").trim_end());
        }
        let _ = writeln!(text);
    }
    if !response.unmanaged_repos.is_empty() {
        let _ = writeln!(text, "Clones mori didn't make (left alone):");
        for repo in &response.unmanaged_repos {
            let _ = writeln!(text, "  {}  {}", repo.repo, repo.path);
        }
        let _ = writeln!(text);
    }
    if !response.repos.is_empty() {
        let _ = writeln!(text, "WORK is as of jj's last snapshot in each tree.");
    }
    text
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
    let _ = if response.validate_only {
        writeln!(
            text,
            "Would remove tree {} (dry run): it is safe to remove.",
            tree.name
        )
    } else {
        writeln!(text, "Removed tree {}:", tree.name)
    };
    if !response.validate_only {
        if response.directory_removed {
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
    use mori_api::v1alpha1::gc_item::Outcome;
    let mut text = String::new();
    let applied = response
        .items
        .iter()
        .any(|item| item.outcome() != Outcome::Unspecified);
    let _ = writeln!(text, "Cleanup:");
    let rows: Vec<[String; 4]> = response
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
            [
                crate::gc::class_name(item.class()).to_owned(),
                format!("{} {}", item.repo, item.name),
                item.reason.clone(),
                what,
            ]
        })
        .collect();
    let mut widths = [0; 4];
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.len());
        }
    }
    for row in &rows {
        let cells: Vec<String> = row
            .iter()
            .zip(widths)
            .map(|(cell, width)| format!("{cell:width$}"))
            .collect();
        let _ = writeln!(text, "  {}", cells.join("  ").trim_end());
    }
    let removable = crate::gc::counts(&response.items)
        .get("remove")
        .copied()
        .unwrap_or(0);
    let _ = if applied {
        Ok(())
    } else if removable == 0 {
        writeln!(text, "Nothing to remove.")
    } else {
        writeln!(
            text,
            "{removable} tree(s) can go: `mori gc --apply --yes` removes them, checking each again first."
        )
    };
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

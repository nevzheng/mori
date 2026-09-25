//! What mori prints: text for people, or one JSON object on stdout with `--json`.
//!
//! Errors follow AIP-193. With `--json` they are a `google.rpc.Status` in its proto3 JSON form,
//! carrying one `google.rpc.ErrorInfo`; as text they go to stderr.

use std::fmt::Write as _;
use std::io::Write;
use std::process::ExitCode;

use mori_api::v1alpha1::InitResponse;
use mori_core::error::{Code, ErrorDetails};
use serde_json::{Map, Value, json};

/// The domain of errors from the command line itself.
const CLI_DOMAIN: &str = "cli.mori";

/// Prints a successful response and exits 0.
pub fn success(json: bool, response: &InitResponse, text: &str) -> ExitCode {
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

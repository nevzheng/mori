//! Checks of the built `mori` binary. The user journeys are covered end to end in `e2e/`; these
//! pin the command-line contract: exit codes, streams and JSON shapes.

use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Runs mori with only `HOME` (and any `vars`) set.
fn mori(home: &Path, vars: &[(&str, &Path)], args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_mori"))
        .env_clear()
        .env("HOME", home)
        .envs(vars.iter().copied())
        .args(args)
        .output()
}

fn stdout_json(out: &Output) -> serde_json::Result<Value> {
    serde_json::from_slice(&out.stdout)
}

#[test]
fn version_flag_prints_name_and_version() {
    let home = tempfile::tempdir().unwrap();
    let out = mori(home.path(), &[], &["--version"]).unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.trim(), format!("mori {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn unknown_arguments_exit_invalid_argument() {
    let home = tempfile::tempdir().unwrap();
    let out = mori(home.path(), &[], &["plant", "a-tree"]).unwrap();

    assert_eq!(out.status.code(), Some(3));
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("Usage: mori")
    );
    assert!(out.stdout.is_empty());
}

#[test]
fn unknown_arguments_with_json_print_a_status() {
    let home = tempfile::tempdir().unwrap();
    let out = mori(home.path(), &[], &["plant", "--json"]).unwrap();

    assert_eq!(out.status.code(), Some(3));
    let status = stdout_json(&out).unwrap();
    assert_eq!(status["code"], 3);
    assert_eq!(
        status["details"][0],
        json!({
            "@type": "type.googleapis.com/google.rpc.ErrorInfo",
            "reason": "INVALID_USAGE",
            "domain": "cli.mori",
        })
    );
}

#[test]
fn init_json_lists_what_it_created_then_nothing() {
    let home = tempfile::tempdir().unwrap();
    let root = home.path().join("mori");

    let first = mori(home.path(), &[], &["init", "--json"]).unwrap();
    assert!(first.status.success());
    let response = stdout_json(&first).unwrap();
    assert_eq!(response["root"], root.to_str().unwrap());
    assert!(response["created"].as_array().unwrap().contains(
        &json!({"path": root.join("repos").to_str().unwrap(), "kind": "KIND_DIRECTORY"})
    ));

    let second = mori(home.path(), &[], &["init", "--json"]).unwrap();
    assert!(second.status.success());
    assert_eq!(
        stdout_json(&second).unwrap(),
        json!({"root": root.to_str().unwrap(), "alreadyInitialized": true})
    );
}

#[test]
fn a_moved_root_exits_failed_precondition() {
    let home = tempfile::tempdir().unwrap();
    assert!(mori(home.path(), &[], &["init"]).unwrap().status.success());
    let other = home.path().join("other");

    let out = mori(home.path(), &[("MORI_ROOT", &other)], &["init", "--json"]).unwrap();

    assert_eq!(out.status.code(), Some(9));
    let status = stdout_json(&out).unwrap();
    assert_eq!(status["code"], 9);
    assert_eq!(status["details"][0]["reason"], "ROOT_MISMATCH");
    assert_eq!(status["details"][0]["domain"], "config.mori");
    assert!(!other.exists());
}

#[test]
fn errors_as_text_go_to_stderr() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join("mori"), "a file, not a directory").unwrap();

    let out = mori(home.path(), &[], &["init"]).unwrap();

    assert_eq!(out.status.code(), Some(9));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("reason: NOT_A_DIRECTORY (store.mori)"),
        "{stderr}"
    );
}

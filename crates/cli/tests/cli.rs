//! End-to-end checks of the built `mori` binary.

use std::process::Command;

fn mori(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_mori")).args(args).output()
}

#[test]
fn version_flag_prints_name_and_version() {
    let out = mori(&["--version"]).unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.trim(), format!("mori {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn unknown_arguments_are_a_usage_error() {
    let out = mori(&["plant", "a-tree"]).unwrap();

    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .starts_with("usage: mori")
    );
}

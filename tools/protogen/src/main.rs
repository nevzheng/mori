//! `bazel run //tools/protogen`: regenerates `crates/api/src/gen` from `proto/`.

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    // `bazel run` sets this to the source tree; under `cargo run`, use the current directory.
    let root = std::env::var_os("BUILD_WORKSPACE_DIRECTORY")
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let out_dir = root.join("crates/api/src/gen");
    // Bazel passes the toolchain's rustfmt; under `cargo run`, use the one on PATH.
    let rustfmt =
        std::env::var_os("RUSTFMT").map_or_else(|| PathBuf::from("rustfmt"), PathBuf::from);

    // Start from an empty directory so code for removed messages doesn't linger.
    let result = std::fs::remove_dir_all(&out_dir)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(err)
            }
        })
        .and_then(|()| std::fs::create_dir_all(&out_dir))
        .map_err(Box::from)
        .and_then(|()| mori_protogen::generate(&root.join("proto"), &out_dir, &rustfmt));

    let mut stderr = std::io::stderr().lock();
    match result {
        Ok(()) => {
            let _ = writeln!(stderr, "protogen: wrote {}", out_dir.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            let _ = writeln!(stderr, "protogen: {err}");
            ExitCode::FAILURE
        }
    }
}

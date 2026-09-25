//! The `mori` command.

use std::io::Write;
use std::process::ExitCode;

/// Exit code for bad arguments (`INVALID_ARGUMENT`).
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--version" || flag == "-V" => {
            match writeln!(
                std::io::stdout().lock(),
                "mori {}",
                env!("CARGO_PKG_VERSION")
            ) {
                Ok(()) => ExitCode::SUCCESS,
                Err(_) => ExitCode::FAILURE,
            }
        }
        _ => {
            // Best effort: if stderr is gone there is nowhere left to report to.
            let _ = writeln!(
                std::io::stderr().lock(),
                "usage: mori --version\n\nmori is pre-alpha: nothing else exists yet."
            );
            ExitCode::from(EXIT_USAGE)
        }
    }
}

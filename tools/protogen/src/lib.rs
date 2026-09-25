//! Generates mori's API code from its protos, the single source of the API: protox parses them
//! (no `protoc` needed),
//! prost writes the Rust types, pbjson writes their proto3 JSON mapping, and rustfmt formats
//! the result so that formatting the repo never changes generated code.

use std::error::Error;
use std::path::Path;
use std::process::Command;

use prost::Message;

/// The API's proto files, relative to the proto root.
pub const PROTOS: &[&str] = &["mori/v1alpha1/mori.proto"];

/// Generates and formats every file for [`PROTOS`] into `out_dir`, which must already exist.
///
/// `rustfmt` is the rustfmt binary to format with.
///
/// # Errors
///
/// If a proto doesn't parse, code generation fails, or rustfmt fails.
pub fn generate(proto_root: &Path, out_dir: &Path, rustfmt: &Path) -> Result<(), Box<dyn Error>> {
    let descriptors = protox::compile(PROTOS, [proto_root])?;
    let encoded = descriptors.encode_to_vec();

    prost_build::Config::new()
        .out_dir(out_dir)
        .compile_fds(descriptors)?;
    pbjson_build::Builder::new()
        .register_descriptors(&encoded)?
        .out_dir(out_dir)
        .build(&[".mori"])?;

    format(out_dir, rustfmt)
}

/// Formats every `.rs` file in `dir` with explicit settings, so the result doesn't depend on
/// which `rustfmt.toml` happens to be above `dir`.
fn format(dir: &Path, rustfmt: &Path) -> Result<(), Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    let status = Command::new(rustfmt)
        .args(["--edition", "2024", "--config", "style_edition=2024"])
        .args(&files)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} failed: {status}", rustfmt.display()).into())
    }
}

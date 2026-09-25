//! Fails when `crates/api/src/gen` no longer matches `proto/`.
//! Fix: run `bazel run //tools/protogen` and commit the result.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn Error>>;

/// Under Bazel, `var` names a file; under cargo, `cargo_fallback` is that file relative to this
/// crate. Returns the file's directory.
fn dir_of(var: &str, cargo_fallback: &str) -> Result<PathBuf, Box<dyn Error>> {
    let file = std::env::var_os(var).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(cargo_fallback),
        PathBuf::from,
    );
    Ok(file.parent().ok_or("file has no parent")?.to_path_buf())
}

/// File name → contents for every file in `dir`.
fn read_dir(dir: &Path) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .ok_or("no file name")?
            .to_string_lossy()
            .into_owned();
        files.insert(name, std::fs::read_to_string(&path)?);
    }
    Ok(files)
}

#[test]
fn generated_code_is_up_to_date() -> TestResult {
    // mori/v1alpha1/mori.proto sits two directories below the proto root.
    let proto_root = dir_of("PROTO_FILE", "../../proto/mori/v1alpha1/mori.proto")?
        .parent()
        .and_then(Path::parent)
        .ok_or("proto file is not under <root>/mori/v1alpha1")?
        .to_path_buf();
    let committed = dir_of("GEN_FILE", "../../crates/api/src/gen/mori.v1alpha1.rs")?;
    let scratch = std::env::var_os("TEST_TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
    let fresh = scratch.join(format!("protogen-{}", std::process::id()));
    std::fs::create_dir_all(&fresh)?;

    let rustfmt =
        std::env::var_os("RUSTFMT").map_or_else(|| PathBuf::from("rustfmt"), PathBuf::from);

    mori_protogen::generate(&proto_root, &fresh, &rustfmt)?;

    assert_eq!(
        read_dir(&committed)?,
        read_dir(&fresh)?,
        "crates/api/src/gen is stale: run `bazel run //tools/protogen` and commit the result"
    );
    Ok(())
}

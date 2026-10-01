//! Cache checks: whether a repo's builds share a user-level cache across trees. Agents make many
//! trees, and each tree that builds from scratch costs disk, so doctor warns where a shared cache
//! is missing. A check runs only where the answer can be read with certainty: when anything it
//! reads can't be read, it says nothing rather than guess.
//!
//! The adapter finds and reads the files; these functions read their text.

use std::path::{Path, PathBuf};

/// Environment variables that set a shared cache for a build, by tool. Any one being set means
/// the cache is configured; their absence proves nothing on its own.
pub const CARGO_ENV: [&str; 6] = [
    "RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_TARGET_DIR",
    "CARGO_BUILD_BUILD_DIR",
];

/// Every environment variable the cache checks read, for the adapter to capture: the Cargo ones,
/// plus where Bazel, Cargo and ccache look for their config.
pub const ENV_VARS: [&str; 11] = [
    "RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_TARGET_DIR",
    "CARGO_BUILD_BUILD_DIR",
    "BAZELRC",
    "CARGO_HOME",
    "CCACHE_CONFIGPATH",
    "CCACHE_DIR",
    "CCACHE_BASEDIR",
];

/// Files that set environment variables for a tree outside the shell mori runs in. A repo with
/// one is skipped: its builds may see variables doctor can't.
pub const ENV_FILES: [&str; 4] = [".envrc", "mise.toml", ".mise.toml", "devbox.json"];

/// What one bazelrc says, as far as caching goes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bazelrc {
    /// A disk cache, remote cache or remote executor is set on some line, in any section.
    pub sets_cache: bool,
    /// The files it imports, with `%workspace%` replaced, and whether each is a `try-import`.
    pub imports: Vec<(PathBuf, bool)>,
}

const BAZEL_CACHE_FLAGS: [&str; 3] = ["--disk_cache", "--remote_cache", "--remote_executor"];

/// Reads a bazelrc. A flag on a `build:<config>` line counts as set: the person may pass that
/// config every time, so a warning there could be wrong.
#[must_use]
pub fn read_bazelrc(text: &str, workspace: &Path) -> Bazelrc {
    let mut rc = Bazelrc::default();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        let mut words = line.split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        if command == "import" || command == "try-import" {
            let rest = line[command.len()..].trim();
            let workspace = workspace.display().to_string();
            rc.imports.push((
                PathBuf::from(rest.replace("%workspace%", &workspace)),
                command == "try-import",
            ));
            continue;
        }
        if words.any(|word| {
            BAZEL_CACHE_FLAGS
                .iter()
                .any(|flag| word == *flag || word.starts_with(&format!("{flag}=")))
        }) {
            rc.sets_cache = true;
        }
    }
    rc
}

/// What one Cargo config file says, as far as caching goes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoConfig {
    /// A compiler wrapper (sccache) or a shared target or build directory is set.
    pub sets_cache: bool,
    /// The files it includes, relative to its own directory unless absolute.
    pub includes: Vec<PathBuf>,
}

/// Reads a Cargo config file. None if it doesn't parse, so the check says nothing.
#[must_use]
pub fn read_cargo_config(text: &str) -> Option<CargoConfig> {
    let value: toml::Table = toml::from_str(text).ok()?;
    let build = value.get("build").and_then(toml::Value::as_table);
    let sets_cache = build.is_some_and(|build| {
        [
            "rustc-wrapper",
            "rustc-workspace-wrapper",
            "target-dir",
            "build-dir",
        ]
        .iter()
        .any(|key| build.contains_key(*key))
    });
    let includes = match value.get("include") {
        None => Vec::new(),
        Some(toml::Value::String(path)) => vec![PathBuf::from(path)],
        Some(toml::Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                toml::Value::String(path) => Some(PathBuf::from(path)),
                toml::Value::Table(table) => table
                    .get("path")
                    .and_then(toml::Value::as_str)
                    .map(PathBuf::from),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        Some(_) => return None,
    };
    Some(CargoConfig {
        sets_cache,
        includes,
    })
}

/// Whether a `ccache.conf` sets `base_dir`, which lets trees at different paths share hits.
#[must_use]
pub fn ccache_sets_base_dir(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.split('#').next().unwrap_or_default();
        line.split_once('=')
            .is_some_and(|(key, value)| key.trim() == "base_dir" && !value.trim().is_empty())
    })
}

/// A shared cache a repo, or the machine, is missing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheGap {
    /// A Bazel repo with no disk or remote cache in any bazelrc it reads.
    Bazel,
    /// A Cargo repo with no compiler wrapper or shared target directory.
    Cargo,
    /// ccache is set up, but without `base_dir`, so trees never share its hits.
    CcacheBaseDir,
}

/// Where the shared caches page explains each fix.
const CACHES_PAGE: &str = "https://nevzheng.github.io/mori/shared-caches/";

impl CacheGap {
    /// What is wrong, leading with why it matters in a forest of trees.
    #[must_use]
    pub fn message(self) -> String {
        let why = "agents make many trees, and each one that builds from scratch costs disk";
        match self {
            Self::Bazel => format!(
                "{why}: no Bazel disk or remote cache is set in any bazelrc this repo reads"
            ),
            Self::Cargo => format!(
                "{why}: no Cargo compiler cache (such as sccache) is set, so every tree compiles \
                 every dependency"
            ),
            Self::CcacheBaseDir => format!(
                "{why}: ccache has no base_dir, so trees at different paths never share its hits"
            ),
        }
    }

    /// The gap as one warning line: what is wrong, then what fixes it.
    #[must_use]
    pub fn warning(self) -> String {
        format!("{}; {}", self.message(), self.fix())
    }

    /// What fixes it.
    #[must_use]
    pub fn fix(self) -> String {
        match self {
            Self::Bazel => format!(
                "add `build --disk_cache=~/.cache/bazel-disk` to ~/.bazelrc, or a remote cache: \
                 {CACHES_PAGE}#bazel"
            ),
            Self::Cargo => format!(
                "set `rustc-wrapper = \"sccache\"` under [build] in ~/.cargo/config.toml: \
                 {CACHES_PAGE}#rust-sccache"
            ),
            Self::CcacheBaseDir => format!(
                "set `base_dir` in ccache.conf to a directory above your mori root, e.g. your \
                 home: {CACHES_PAGE}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WS: &str = "/home/acme/mori/repos/github.com/acme/widget";

    #[test]
    fn a_bazelrc_with_a_cache_flag_sets_it() {
        for text in [
            "build --disk_cache=~/.cache/bazel-disk",
            "common --disk_cache",
            "build --remote_cache=grpcs://cache.example.com # shared",
            "build:ci --remote_executor=grpcs://re.example.com",
            "test --config=x --disk_cache=/tmp/d",
        ] {
            assert!(read_bazelrc(text, Path::new(WS)).sets_cache, "{text}");
        }
    }

    #[test]
    fn a_bazelrc_without_one_doesnt() {
        for text in [
            "",
            "# build --disk_cache=/tmp/d",
            "build --jobs=8",
            "build --disk_cache_gc_max_size=1G",
        ] {
            assert!(!read_bazelrc(text, Path::new(WS)).sets_cache, "{text}");
        }
    }

    #[test]
    fn a_bazelrc_lists_its_imports() {
        let rc = read_bazelrc(
            "import %workspace%/tools/common.bazelrc\ntry-import %workspace%/user.bazelrc\n",
            Path::new(WS),
        );

        assert_eq!(
            rc.imports,
            [
                (PathBuf::from(format!("{WS}/tools/common.bazelrc")), false),
                (PathBuf::from(format!("{WS}/user.bazelrc")), true),
            ]
        );
    }

    #[test]
    fn a_cargo_config_with_a_wrapper_or_shared_target_sets_it() {
        for text in [
            "[build]\nrustc-wrapper = \"sccache\"",
            "[build]\ntarget-dir = \"/tmp/target\"",
            "build.rustc-workspace-wrapper = \"sccache\"",
        ] {
            assert!(read_cargo_config(text).unwrap().sets_cache, "{text}");
        }
        assert!(!read_cargo_config("[build]\njobs = 4").unwrap().sets_cache);
    }

    #[test]
    fn a_cargo_config_lists_its_includes_and_refuses_what_it_cant_read() {
        let one = read_cargo_config("include = \"extra.toml\"").unwrap();
        let many = read_cargo_config("include = [\"a.toml\", { path = \"b.toml\" }]").unwrap();

        assert_eq!(one.includes, [PathBuf::from("extra.toml")]);
        assert_eq!(
            many.includes,
            [PathBuf::from("a.toml"), PathBuf::from("b.toml")]
        );
        assert_eq!(read_cargo_config("not toml ["), None);
        assert_eq!(read_cargo_config("include = 3"), None);
    }

    #[test]
    fn ccache_base_dir_is_read() {
        assert!(ccache_sets_base_dir(
            "max_size = 50G\nbase_dir = /home/acme\n"
        ));
        assert!(!ccache_sets_base_dir("max_size = 50G\n"));
        assert!(!ccache_sets_base_dir("# base_dir = /home/acme\n"));
        assert!(!ccache_sets_base_dir("base_dir =\n"));
    }

    #[test]
    fn every_gap_says_why_and_how() {
        for gap in [CacheGap::Bazel, CacheGap::Cargo, CacheGap::CcacheBaseDir] {
            assert!(gap.message().starts_with("agents make many trees"));
            assert!(gap.fix().contains(CACHES_PAGE));
        }
    }
}

//! The cache checks' reading: find the config files Bazel, Cargo and ccache would read, read them,
//! and let `mori-core` say what they set. Reads only. Anything that can't be read with certainty
//! (a file that exists but won't read or parse, a missing `import`) makes that check say nothing.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use mori_core::cache::{
    CARGO_ENV, CacheGap, ENV_FILES, ccache_sets_base_dir, read_bazelrc, read_cargo_config,
};

use crate::Host;

/// Where Bazel reads its system-wide bazelrc.
pub const SYSTEM_BAZELRC: &str = "/etc/bazel.bazelrc";

/// Where ccache may keep a system-wide config, depending on how it was installed.
pub const SYSTEM_CCACHE_CONFS: [&str; 3] = [
    "/etc/ccache.conf",
    "/usr/local/etc/ccache.conf",
    "/opt/homebrew/etc/ccache.conf",
];

/// How deep `import` and `include` chains are followed before giving up on the check.
const MAX_DEPTH: usize = 8;

/// The shared caches the repo at `clone` is missing. `system_bazelrc` is [`SYSTEM_BAZELRC`] outside
/// tests.
#[must_use]
pub fn repo_gaps(host: &Host, clone: &Path, system_bazelrc: &Path) -> Vec<CacheGap> {
    if ENV_FILES.iter().any(|name| clone.join(name).exists()) {
        return Vec::new();
    }
    let mut gaps = Vec::new();
    if bazel_lacks_cache(host, clone, system_bazelrc) == Some(true) {
        gaps.push(CacheGap::Bazel);
    }
    if cargo_lacks_cache(host, clone) == Some(true) {
        gaps.push(CacheGap::Cargo);
    }
    gaps
}

/// Whether ccache is set up (a user `ccache.conf` exists) without `base_dir` anywhere it reads.
/// `system_confs` is [`SYSTEM_CCACHE_CONFS`] outside tests.
#[must_use]
pub fn ccache_lacks_base_dir(host: &Host, system_confs: &[PathBuf]) -> bool {
    if host.cache_vars.contains_key("CCACHE_BASEDIR") {
        return false;
    }
    let Some(user_conf) = ccache_user_conf(host) else {
        return false;
    };
    let mut confs = vec![user_conf];
    confs.extend(system_confs.iter().cloned());
    let mut user_conf_exists = false;
    for (index, conf) in confs.iter().enumerate() {
        match read(conf) {
            Read::Text(text) => {
                if ccache_sets_base_dir(&text) {
                    return false;
                }
                if index == 0 {
                    user_conf_exists = true;
                }
            }
            Read::Missing => {}
            Read::Unreadable => return false,
        }
    }
    user_conf_exists
}

/// The ccache config ccache itself would use, as its documentation orders them.
fn ccache_user_conf(host: &Host) -> Option<PathBuf> {
    if let Some(path) = host.cache_vars.get("CCACHE_CONFIGPATH") {
        return Some(PathBuf::from(path));
    }
    if let Some(dir) = host.cache_vars.get("CCACHE_DIR") {
        return Some(PathBuf::from(dir).join("ccache.conf"));
    }
    let home = host.env.home.as_ref()?;
    let legacy = home.join(".ccache");
    if legacy.is_dir() {
        return Some(legacy.join("ccache.conf"));
    }
    let config = host
        .env
        .xdg_config_home
        .clone()
        .unwrap_or_else(|| home.join(".config"));
    Some(config.join("ccache/ccache.conf"))
}

/// Some(true) when the clone is a Bazel repo and no bazelrc it reads sets a cache; None when that
/// can't be known.
fn bazel_lacks_cache(host: &Host, clone: &Path, system_bazelrc: &Path) -> Option<bool> {
    let markers = ["MODULE.bazel", "WORKSPACE", "WORKSPACE.bazel"];
    if !markers.iter().any(|name| clone.join(name).is_file()) {
        return Some(false);
    }
    if clone.join("tools/bazel").exists() {
        return None;
    }
    let home = host.env.home.as_ref()?;
    // The optional ones: Bazel reads each if it exists.
    let mut rcs = vec![
        (system_bazelrc.to_path_buf(), true),
        (clone.join(".bazelrc"), true),
        (home.join(".bazelrc"), true),
    ];
    if let Some(paths) = host.cache_vars.get("BAZELRC") {
        rcs.extend(std::env::split_paths(paths).map(|path| (path, false)));
    }
    let mut seen = BTreeSet::new();
    for (rc, optional) in rcs {
        if bazelrc_sets_cache(&rc, optional, clone, 0, &mut seen)? {
            return Some(false);
        }
    }
    Some(true)
}

/// Whether `rc`, or anything it imports, sets a cache; None when that can't be known.
fn bazelrc_sets_cache(
    rc: &Path,
    optional: bool,
    workspace: &Path,
    depth: usize,
    seen: &mut BTreeSet<PathBuf>,
) -> Option<bool> {
    if depth > MAX_DEPTH {
        return None;
    }
    if !seen.insert(rc.to_path_buf()) {
        return Some(false);
    }
    let text = match read(rc) {
        Read::Text(text) => text,
        Read::Missing if optional => return Some(false),
        Read::Missing | Read::Unreadable => return None,
    };
    let parsed = read_bazelrc(&text, workspace);
    if parsed.sets_cache {
        return Some(true);
    }
    for (import, try_import) in parsed.imports {
        if bazelrc_sets_cache(&import, try_import, workspace, depth + 1, seen)? {
            return Some(true);
        }
    }
    Some(false)
}

/// Some(true) when the clone is a Cargo repo and neither the environment nor any Cargo config it
/// reads sets a compiler cache or shared target directory; None when that can't be known.
fn cargo_lacks_cache(host: &Host, clone: &Path) -> Option<bool> {
    if !clone.join("Cargo.toml").is_file() {
        return Some(false);
    }
    if CARGO_ENV
        .iter()
        .any(|name| host.cache_vars.contains_key(*name))
    {
        return Some(false);
    }
    let cargo_home = match host.cache_vars.get("CARGO_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => host.env.home.as_ref()?.join(".cargo"),
    };
    let mut dirs: Vec<PathBuf> = clone.ancestors().map(|dir| dir.join(".cargo")).collect();
    dirs.push(cargo_home);
    let mut seen = BTreeSet::new();
    for dir in dirs {
        for name in ["config.toml", "config"] {
            if cargo_config_sets_cache(&dir.join(name), true, 0, &mut seen)? {
                return Some(false);
            }
        }
    }
    Some(true)
}

/// Whether the Cargo config at `path`, or anything it includes, sets a cache; None when that
/// can't be known.
fn cargo_config_sets_cache(
    path: &Path,
    optional: bool,
    depth: usize,
    seen: &mut BTreeSet<PathBuf>,
) -> Option<bool> {
    if depth > MAX_DEPTH {
        return None;
    }
    if !seen.insert(path.to_path_buf()) {
        return Some(false);
    }
    let text = match read(path) {
        Read::Text(text) => text,
        Read::Missing if optional => return Some(false),
        Read::Missing | Read::Unreadable => return None,
    };
    let config = read_cargo_config(&text)?;
    if config.sets_cache {
        return Some(true);
    }
    let dir = path.parent().unwrap_or(Path::new("/"));
    for include in config.includes {
        if cargo_config_sets_cache(&dir.join(include), false, depth + 1, seen)? {
            return Some(true);
        }
    }
    Some(false)
}

enum Read {
    Text(String),
    Missing,
    Unreadable,
}

fn read(path: &Path) -> Read {
    match std::fs::read_to_string(path) {
        Ok(text) => Read::Text(text),
        Err(error) if error.kind() == ErrorKind::NotFound => Read::Missing,
        Err(_) => Read::Unreadable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use mori_core::paths::Env;
    use tempfile::TempDir;

    type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// The variables a test host sets: `(name, value)` pairs.
    fn vars(pairs: &[(&str, &str)]) -> std::collections::BTreeMap<String, std::ffi::OsString> {
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.into()))
            .collect()
    }

    struct Machine {
        home: TempDir,
        clone: PathBuf,
        etc: PathBuf,
        host: Host,
    }

    impl Machine {
        fn new() -> Result<Self> {
            let home = TempDir::new()?;
            let clone = home.path().join("mori/repos/github.com/acme/widget");
            let etc = home.path().join("etc");
            fs::create_dir_all(&clone)?;
            fs::create_dir_all(&etc)?;
            let host = Host {
                env: Env {
                    home: Some(home.path().to_path_buf()),
                    ..Env::default()
                },
                ..Host::default()
            };
            Ok(Self {
                home,
                clone,
                etc,
                host,
            })
        }

        fn gaps(&self) -> Vec<CacheGap> {
            repo_gaps(&self.host, &self.clone, &self.etc.join("bazel.bazelrc"))
        }

        fn ccache(&self) -> bool {
            ccache_lacks_base_dir(&self.host, &[self.etc.join("ccache.conf")])
        }
    }

    fn write(path: &Path, text: &str) -> Result {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, text)?;
        Ok(())
    }

    #[test]
    fn a_repo_without_build_files_has_no_gaps() -> Result {
        let machine = Machine::new()?;

        assert_eq!(machine.gaps(), []);
        Ok(())
    }

    #[test]
    fn a_bazel_repo_with_no_cache_anywhere_has_a_gap() -> Result {
        let machine = Machine::new()?;
        write(&machine.clone.join("MODULE.bazel"), "")?;
        write(&machine.clone.join(".bazelrc"), "build --jobs=8\n")?;

        assert_eq!(machine.gaps(), [CacheGap::Bazel]);
        Ok(())
    }

    #[test]
    fn a_bazel_cache_in_any_rc_or_import_closes_the_gap() -> Result {
        for (file, text) in [
            ("home/.bazelrc", "build --disk_cache=~/.cache/bazel-disk\n"),
            (
                "etc/bazel.bazelrc",
                "common --remote_cache=grpcs://c.example.com\n",
            ),
            ("clone/.bazelrc", "try-import %workspace%/user.bazelrc\n"),
        ] {
            let machine = Machine::new()?;
            write(&machine.clone.join("MODULE.bazel"), "")?;
            write(
                &machine.clone.join("user.bazelrc"),
                "build --disk_cache=/tmp/d\n",
            )?;
            let path = match file.split_once('/') {
                Some(("home", rest)) => machine.home.path().join(rest),
                Some(("etc", rest)) => machine.etc.join(rest),
                Some((_, rest)) => machine.clone.join(rest),
                None => return Err("bad case".into()),
            };
            write(&path, text)?;

            assert_eq!(machine.gaps(), [], "{file}");
        }
        Ok(())
    }

    #[test]
    fn bazel_is_skipped_when_it_cant_be_read_with_certainty() -> Result {
        let wrapper = Machine::new()?;
        write(&wrapper.clone.join("WORKSPACE"), "")?;
        write(&wrapper.clone.join("tools/bazel"), "#!/bin/sh\n")?;
        let missing_import = Machine::new()?;
        write(&missing_import.clone.join("WORKSPACE"), "")?;
        write(
            &missing_import.clone.join(".bazelrc"),
            "import %workspace%/ci.bazelrc\n",
        )?;
        let env_file = Machine::new()?;
        write(&env_file.clone.join("WORKSPACE"), "")?;
        write(&env_file.clone.join(".envrc"), "")?;

        assert_eq!(wrapper.gaps(), []);
        assert_eq!(missing_import.gaps(), []);
        assert_eq!(env_file.gaps(), []);
        Ok(())
    }

    #[test]
    fn a_cargo_repo_with_no_cache_has_a_gap() -> Result {
        let machine = Machine::new()?;
        write(&machine.clone.join("Cargo.toml"), "[workspace]\n")?;
        write(
            &machine.home.path().join(".cargo/config.toml"),
            "[net]\nretry = 3\n",
        )?;

        assert_eq!(machine.gaps(), [CacheGap::Cargo]);
        Ok(())
    }

    #[test]
    fn a_cargo_cache_in_config_include_parent_or_env_closes_the_gap() -> Result {
        let home_config = Machine::new()?;
        write(
            &home_config.home.path().join(".cargo/config.toml"),
            "[build]\nrustc-wrapper = \"sccache\"\n",
        )?;
        let included = Machine::new()?;
        write(
            &included.home.path().join(".cargo/config.toml"),
            "include = \"cache.toml\"\n",
        )?;
        write(
            &included.home.path().join(".cargo/cache.toml"),
            "[build]\ntarget-dir = \"/tmp/t\"\n",
        )?;
        let parent = Machine::new()?;
        write(
            &parent.home.path().join("mori/.cargo/config"),
            "[build]\nrustc-wrapper = \"sccache\"\n",
        )?;
        let mut env = Machine::new()?;
        env.host.cache_vars = vars(&[("RUSTC_WRAPPER", "sccache")]);
        let mut cargo_home = Machine::new()?;
        let elsewhere = cargo_home.home.path().join("cargo-home");
        write(
            &elsewhere.join("config.toml"),
            "[build]\nrustc-wrapper = \"sccache\"\n",
        )?;
        cargo_home.host.cache_vars = vars(&[("CARGO_HOME", &elsewhere.display().to_string())]);

        for machine in [home_config, included, parent, env, cargo_home] {
            write(&machine.clone.join("Cargo.toml"), "[workspace]\n")?;
            assert_eq!(machine.gaps(), []);
        }
        Ok(())
    }

    #[test]
    fn cargo_is_skipped_when_a_config_wont_parse() -> Result {
        let machine = Machine::new()?;
        write(&machine.clone.join("Cargo.toml"), "[workspace]\n")?;
        write(&machine.clone.join(".cargo/config.toml"), "not toml [")?;

        assert_eq!(machine.gaps(), []);
        Ok(())
    }

    #[test]
    fn ccache_without_base_dir_has_a_gap_only_when_it_is_set_up() -> Result {
        let unused = Machine::new()?;
        let set_up = Machine::new()?;
        write(
            &set_up.home.path().join(".config/ccache/ccache.conf"),
            "max_size = 50G\n",
        )?;
        let system_base = Machine::new()?;
        write(
            &system_base.home.path().join(".ccache/ccache.conf"),
            "max_size = 50G\n",
        )?;
        write(&system_base.etc.join("ccache.conf"), "base_dir = /home\n")?;
        let mut env_base = Machine::new()?;
        write(
            &env_base.home.path().join(".config/ccache/ccache.conf"),
            "max_size = 50G\n",
        )?;
        env_base.host.cache_vars = vars(&[("CCACHE_BASEDIR", "/home")]);

        assert!(!unused.ccache());
        assert!(set_up.ccache());
        assert!(!system_base.ccache());
        assert!(!env_base.ccache());
        Ok(())
    }
}

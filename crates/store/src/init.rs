//! `mori init` on the local disk: [`observe`] what exists, then [`apply`] the plan that
//! `mori_core::init::plan` makes from it.
//!
//! Nothing here overwrites or removes anything that existed before. If a step fails part way,
//! what was created stays, and running `init` again plans only what is still missing.

use std::fs::{DirBuilder, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use mori_core::init::{InitPlan, Observed, Step, paths_to_observe};
use mori_core::paths::Paths;

use crate::StoreError;
use crate::database::Database;

/// Directories that mark a clone: git's and jj's.
const CLONE_MARKERS: [&str; 2] = [".git", ".jj"];

/// Looks at everything [`mori_core::init::plan`] needs. Reads only.
///
/// # Errors
///
/// [`StoreError::NotADirectory`] if a file is where a directory must go, errors from opening an
/// existing database, and I/O errors other than "not found".
pub fn observe(paths: &Paths) -> Result<Observed, StoreError> {
    let mut existing_dirs = std::collections::BTreeSet::new();
    for path in paths_to_observe(paths) {
        if is_dir(&path)? {
            existing_dirs.insert(path);
        }
    }
    let config_text = match std::fs::read_to_string(&paths.config_file) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(source) => return Err(io(&paths.config_file, source)),
    };
    let database_root = if exists(&paths.database)? {
        Some(Database::open(&paths.database)?.root()?)
    } else {
        None
    };
    Ok(Observed {
        existing_dirs,
        config_text,
        database_root,
        clones: clones(&paths.repos())?,
    })
}

/// Carries out `plan`, in order. Each step creates something new and fails rather than touch
/// anything that already exists.
///
/// Permission bits are passed to the OS as the mode, so the user's umask can narrow them but never
/// widen them.
///
/// # Errors
///
/// [`StoreError::AlreadyExists`] if something appeared since [`observe`] ran, otherwise I/O or
/// SQLite errors. Steps before the failing one stay done.
pub fn apply(plan: &InitPlan) -> Result<(), StoreError> {
    for step in &plan.steps {
        match step {
            Step::CreateDir { path, mode } => DirBuilder::new()
                .mode(*mode)
                .create(path)
                .map_err(|source| create_error(path, source))?,
            Step::WriteConfig {
                path,
                contents,
                mode,
            } => write_new(path, contents, *mode)?,
            Step::CreateDatabase { path, root, mode } => {
                Database::create(path, root, *mode)?;
            }
        }
    }
    Ok(())
}

/// Writes a new file all at once: to a temporary file beside it, then linked into place, which
/// fails if `path` exists. Readers never see a partly written file.
fn write_new(path: &Path, contents: &str, mode: u32) -> Result<(), StoreError> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".tmp-{}", std::process::id()));
    let temporary = PathBuf::from(temporary);

    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(&temporary)
        .and_then(|mut file| {
            file.write_all(contents.as_bytes())?;
            file.sync_all()
        })
        .map_err(|source| create_error(&temporary, source))
        .and_then(|()| {
            std::fs::hard_link(&temporary, path).map_err(|source| create_error(path, source))
        });
    // Best effort, and safe: this call created the temporary file. When the link succeeded, `path`
    // still holds the contents.
    let _ = std::fs::remove_file(&temporary);
    written
}

/// Clone directories at `<repos>/<host>/<org>/<repo>`: those holding `.git` or `.jj`. Symlinks
/// aren't followed.
fn clones(repos: &Path) -> Result<Vec<PathBuf>, StoreError> {
    let mut clones = Vec::new();
    for host in subdirs(repos)? {
        for org in subdirs(&host)? {
            for repo in subdirs(&org)? {
                let mut marked = false;
                for marker in CLONE_MARKERS {
                    marked |= exists(&repo.join(marker))?;
                }
                if marked {
                    clones.push(repo);
                }
            }
        }
    }
    clones.sort();
    Ok(clones)
}

/// The real directories (not symlinks) directly inside `dir`; none if `dir` doesn't exist.
fn subdirs(dir: &Path) -> Result<Vec<PathBuf>, StoreError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(io(dir, source)),
    };
    let mut dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| io(dir, source))?;
        let file_type = entry
            .file_type()
            .map_err(|source| io(&entry.path(), source))?;
        if file_type.is_dir() {
            dirs.push(entry.path());
        }
    }
    Ok(dirs)
}

/// Whether `path` is a directory (following symlinks, so a root that is a symlink works).
fn is_dir(path: &Path) -> Result<bool, StoreError> {
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(_) => Err(StoreError::NotADirectory {
            path: path.to_path_buf(),
        }),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(source) => Err(io(path, source)),
    }
}

fn exists(path: &Path) -> Result<bool, StoreError> {
    path.try_exists().map_err(|source| io(path, source))
}

fn create_error(path: &Path, source: std::io::Error) -> StoreError {
    if source.kind() == ErrorKind::AlreadyExists {
        StoreError::AlreadyExists {
            path: path.to_path_buf(),
        }
    } else {
        io(path, source)
    }
}

fn io(path: &Path, source: std::io::Error) -> StoreError {
    StoreError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use mori_core::error::ConfigError;
    use mori_core::init::plan;
    use mori_core::paths::Env;

    use super::*;

    /// A temporary HOME, and mori's paths under it.
    fn home() -> Result<(tempfile::TempDir, Paths), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let home = dir.path().to_path_buf();
        let paths = Paths::resolve(&Env {
            home: Some(home),
            ..Env::default()
        })?;
        Ok((dir, paths))
    }

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn first_run_creates_everything() {
        let (_home, paths) = home().unwrap();
        let plan = plan(&paths, &observe(&paths).unwrap()).unwrap();
        apply(&plan).unwrap();

        assert!(paths.repos().is_dir());
        assert!(paths.trees().is_dir());
        assert_eq!(mode(&paths.state_dir), 0o700);
        assert_eq!(mode(&paths.database), 0o600);
        let observed = observe(&paths).unwrap();
        assert_eq!(observed.database_root, Some(paths.root.clone()));
        assert_eq!(
            observed.config_text,
            Some(mori_core::config::Config::render(&paths.root).unwrap())
        );
    }

    #[test]
    fn second_run_plans_nothing() {
        let (_home, paths) = home().unwrap();
        apply(&plan(&paths, &observe(&paths).unwrap()).unwrap()).unwrap();

        let again = plan(&paths, &observe(&paths).unwrap()).unwrap();
        assert!(again.already_initialized());
    }

    #[test]
    fn a_changed_root_is_refused_before_anything_is_created() {
        let (home, paths) = home().unwrap();
        apply(&plan(&paths, &observe(&paths).unwrap()).unwrap()).unwrap();
        let moved = Paths::resolve(&Env {
            home: Some(home.path().to_path_buf()),
            mori_root: Some(home.path().join("other")),
            ..Env::default()
        })
        .unwrap();

        let error = plan(&moved, &observe(&moved).unwrap()).unwrap_err();
        assert!(matches!(error, ConfigError::RootMismatch { .. }));
        assert!(!moved.root.exists());
    }

    #[test]
    fn apply_never_overwrites_a_config_that_appeared() {
        let (_home, paths) = home().unwrap();
        let plan = plan(&paths, &observe(&paths).unwrap()).unwrap();
        std::fs::create_dir_all(&paths.config_dir).unwrap();
        std::fs::write(&paths.config_file, "# mine\n").unwrap();

        let error = apply(&plan).unwrap_err();
        assert!(matches!(error, StoreError::AlreadyExists { .. }));
        assert_eq!(
            std::fs::read_to_string(&paths.config_file).unwrap(),
            "# mine\n"
        );
    }

    #[test]
    fn write_new_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_new(&path, "schema = 1\n", 0o644).unwrap();

        let names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from("config.toml")]);
        assert_eq!(mode(&path), 0o644);
    }

    #[test]
    fn observe_finds_git_and_jj_clones_only() {
        let (_home, paths) = home().unwrap();
        let repos = paths.repos();
        for (repo, marker) in [
            ("github.com/acme/widget", Some(".git")),
            ("github.com/acme/gadget", Some(".jj")),
            ("github.com/acme/empty", None),
        ] {
            let dir = repos.join(repo);
            std::fs::create_dir_all(&dir).unwrap();
            if let Some(marker) = marker {
                std::fs::create_dir(dir.join(marker)).unwrap();
            }
        }
        std::fs::write(repos.join("github.com/acme/README"), "").unwrap();

        let clones = observe(&paths).unwrap().clones;
        assert_eq!(
            clones,
            vec![
                repos.join("github.com/acme/gadget"),
                repos.join("github.com/acme/widget"),
            ]
        );
    }

    #[test]
    fn observe_refuses_a_file_where_a_directory_goes() {
        let (_home, paths) = home().unwrap();
        std::fs::write(&paths.root, "").unwrap();

        let error = observe(&paths).unwrap_err();
        assert!(matches!(error, StoreError::NotADirectory { .. }));
        assert_eq!(error.reason(), "NOT_A_DIRECTORY");
    }
}

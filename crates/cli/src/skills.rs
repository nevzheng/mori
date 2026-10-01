//! Skills in the root: `mori init` installs the missing ones, `mori skills sync` updates them.

use std::path::{Path, PathBuf};

use mori_api::v1alpha1::{SkillFile, SyncSkillsResponse, skill_file::Action as FileAction};
use mori_core::error::{ConfigError, ErrorDetails};
use mori_core::paths::Paths;
use mori_core::skills::{Action, Mode, RepoContext, Step};

use crate::state::{self, boxed};

/// What a skills run did (or would do), in the order it happens.
pub struct Outcome {
    /// Every file mori ships, and what happens to it.
    pub steps: Vec<Step>,
    /// The directories and files that don't exist yet and get created, parents first.
    pub created: Vec<(PathBuf, bool)>,
    /// Files from the old `skills/` layout that someone edited, so they stay where they are.
    pub kept_legacy: Vec<PathBuf>,
}

/// Plans the skills in `mode` and, unless `dry_run`, writes them.
pub fn run(paths: &Paths, mode: Mode, dry_run: bool) -> Result<Outcome, Box<dyn ErrorDetails>> {
    let kept_legacy = if dry_run {
        Vec::new()
    } else {
        mori_store::skills::migrate_legacy(paths).map_err(boxed)?
    };
    let observed = mori_store::skills::observe(paths).map_err(boxed)?;
    let steps = mori_store::skills::sync_plan(&observed, mode, &repo_contexts(paths)?);
    let created = created(paths, &steps);
    if !dry_run {
        mori_store::skills::apply(paths, &steps, observed.manifest).map_err(boxed)?;
    }
    Ok(Outcome {
        steps,
        created,
        kept_legacy,
    })
}

/// Every recorded repo's context folder, for the index. None before mori is set up.
fn repo_contexts(paths: &Paths) -> Result<Vec<RepoContext>, Box<dyn ErrorDetails>> {
    if paths.database.symlink_metadata().is_err() {
        return Ok(Vec::new());
    }
    let db = mori_store::database::Database::open(&paths.database).map_err(boxed)?;
    Ok(db
        .repos()
        .map_err(boxed)?
        .into_iter()
        .map(|repo| RepoContext {
            dir: repo.dir_name,
            repo: repo.remote,
        })
        .collect())
}

/// The new directories (`true`) and files (`false`) the written steps bring into existence.
/// Directories outside the root are `init`'s own business and aren't listed.
fn created(paths: &Paths, steps: &[Step]) -> Vec<(PathBuf, bool)> {
    let mut created: Vec<(PathBuf, bool)> = Vec::new();
    let mut add = |path: &Path, is_dir: bool| {
        if !path.exists() && !created.iter().any(|(seen, _)| seen == path) {
            created.push((path.to_path_buf(), is_dir));
        }
    };
    for step in steps.iter().filter(|step| step.action.writes()) {
        let file = paths.root.join(&step.wanted.path);
        let mut dirs: Vec<&Path> = file
            .ancestors()
            .skip(1)
            .take_while(|dir| *dir != paths.root)
            .collect();
        dirs.reverse();
        for dir in dirs {
            add(dir, true);
        }
        add(&file, false);
    }
    if steps.iter().any(|step| step.action.writes()) {
        add(&mori_store::skills::manifest_path(paths), false);
    }
    created
}

/// Runs `mori skills sync`.
pub fn sync(dry_run: bool) -> Result<SyncSkillsResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths()?;
    if paths.database.symlink_metadata().is_err() {
        return Err(boxed(ConfigError::NotInitialized {
            database: paths.database.clone(),
        }));
    }
    let outcome = run(&paths, Mode::Sync, dry_run)?;
    Ok(SyncSkillsResponse {
        files: outcome
            .steps
            .iter()
            .map(|step| SkillFile {
                path: paths.root.join(&step.wanted.path).display().to_string(),
                action: match step.action {
                    Action::Install => FileAction::Installed,
                    Action::Unchanged => FileAction::Unchanged,
                    Action::Update => FileAction::Updated,
                    Action::KeptEdited => FileAction::KeptEdited,
                    Action::SkippedNotOurs => FileAction::SkippedNotOurs,
                }
                .into(),
            })
            .chain(outcome.kept_legacy.iter().map(|path| SkillFile {
                path: path.display().to_string(),
                action: FileAction::KeptEdited.into(),
            }))
            .collect(),
        validate_only: dry_run,
    })
}

/// A one-line hint when the root's skills came from a different mori than this one. Printed to
/// stderr by `init` and `ls`, never part of their JSON.
pub fn stale_hint() {
    let Ok(paths) = state::paths() else {
        return;
    };
    if let Some(installed) = mori_store::skills::installed_version(&paths)
        && installed != mori_store::skills::THIS_VERSION
    {
        eprintln!(
            "note: the skills in {} are from mori {installed}; run `mori skills sync` to update them",
            paths.root.join("skills").display()
        );
    }
}

//! `mori init`: observe the disk, plan, and (unless it's a dry run) apply.

use std::path::PathBuf;

use mori_api::v1alpha1::{CreatedPath, InitResponse, UnmanagedRepo, created_path::Kind};
use mori_core::error::ErrorDetails;
use mori_core::init::{InitPlan, Step, plan};
use mori_core::paths::Paths;
use mori_core::skills::Mode;

use crate::state::boxed;
use crate::{Host, skills};

/// Runs `init` against the host's environment and the disk.
///
/// # Errors
///
/// The plan's refusal, or a failure of the disk, the database or an adapter, with its code and
/// reason.
pub fn run(host: &Host, dry_run: bool) -> Result<InitResponse, Box<dyn ErrorDetails>> {
    let paths = Paths::resolve(&host.env).map_err(boxed)?;
    let observed = mori_store::init::observe(&paths).map_err(boxed)?;
    let plan = plan(&paths, &observed).map_err(boxed)?;
    if !dry_run {
        mori_store::init::apply(&plan).map_err(boxed)?;
    }
    // Skills are only added, never changed: updating them is `mori skills sync`.
    let skills = skills::run(&paths, Mode::AddOnly, dry_run)?;
    Ok(response(&plan, &skills.created, dry_run))
}

fn response(plan: &InitPlan, skills: &[(PathBuf, bool)], dry_run: bool) -> InitResponse {
    let kind = |is_dir: bool| if is_dir { Kind::Directory } else { Kind::File };
    InitResponse {
        root: plan.root.display().to_string(),
        already_initialized: plan.already_initialized() && skills.is_empty(),
        validate_only: dry_run,
        created: plan
            .steps
            .iter()
            .map(|step| {
                (
                    step.path().to_path_buf(),
                    matches!(step, Step::CreateDir { .. }),
                )
            })
            .chain(skills.iter().cloned())
            .map(|(path, is_dir)| CreatedPath {
                path: path.display().to_string(),
                kind: kind(is_dir).into(),
            })
            .collect(),
        unmanaged_repos: plan
            .unmanaged
            .iter()
            .map(|repo| UnmanagedRepo {
                repo: repo.repo.clone(),
                path: repo.path.display().to_string(),
            })
            .collect(),
    }
}

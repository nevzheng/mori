//! `mori init`: observe the disk, plan, and (unless it's a dry run) apply.

use mori_api::v1alpha1::{CreatedPath, InitResponse, UnmanagedRepo, created_path::Kind};
use mori_core::error::ErrorDetails;
use mori_core::init::{InitPlan, Step, plan};
use mori_core::paths::{Env, Paths};

/// Runs `init` against the real environment and disk.
pub fn run(dry_run: bool) -> Result<InitResponse, Box<dyn ErrorDetails>> {
    let paths = Paths::resolve(&Env::from_vars(|name| std::env::var_os(name))).map_err(boxed)?;
    let observed = mori_store::init::observe(&paths).map_err(boxed)?;
    let plan = plan(&paths, &observed).map_err(boxed)?;
    if !dry_run {
        mori_store::init::apply(&plan).map_err(boxed)?;
    }
    Ok(response(&plan, dry_run))
}

fn response(plan: &InitPlan, dry_run: bool) -> InitResponse {
    InitResponse {
        root: plan.root.display().to_string(),
        already_initialized: plan.already_initialized(),
        validate_only: dry_run,
        created: plan
            .steps
            .iter()
            .map(|step| CreatedPath {
                path: step.path().display().to_string(),
                kind: match step {
                    Step::CreateDir { .. } => Kind::Directory,
                    Step::WriteConfig { .. } | Step::CreateDatabase { .. } => Kind::File,
                }
                .into(),
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

fn boxed(error: impl ErrorDetails + 'static) -> Box<dyn ErrorDetails> {
    Box::new(error)
}

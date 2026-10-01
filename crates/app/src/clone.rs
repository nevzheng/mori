//! `mori clone`: observe, plan, then the VCS clone and a best-effort record of it.

use std::collections::BTreeSet;

use mori_api::v1alpha1::CloneResponse;
use mori_core::clone::{
    BASE_TREE_NAME, BASE_TREE_OWNER, ClonePlan, CloneUrl, Observed, clone_path, plan,
};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::paths::Paths;
use mori_core::skills::RepoContext;
use mori_core::tree::Lifetime;
use mori_core::tree_create::default_owner;
use mori_store::StoreError;
use mori_store::database::Database;
use mori_store::records::{NewRepo, NewTree};

use crate::state::{self, boxed};
use crate::{App, Backend};
use mori_core::vcs::{Forge, VcsKind};

/// Runs `clone`: the VCS clone, then a best-effort record of it.
///
/// # Errors
///
/// The plan's refusal, or a failure of the disk, the database or an adapter, with its code and
/// reason.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    url: &str,
    dry_run: bool,
    jj_only: bool,
    vcs: Option<VcsKind>,
) -> Result<CloneResponse, Box<dyn ErrorDetails>> {
    let url = CloneUrl::parse(url).map_err(boxed)?;
    let paths = state::paths(&app.host)?;
    let mut db = state::open_database(&paths)?;
    let observed = observe(&paths, &db, &url)?;
    let vcs = state::clone_vcs(&paths, vcs)?;
    let plan = plan(&paths, url, vcs, !jj_only, &observed).map_err(boxed)?;
    let mut warnings = Vec::new();
    if !dry_run {
        clone(&app.vcs, &plan)?;
        record(&mut db, &plan, app.host.user.as_deref())?;
        let repo = RepoContext {
            dir: plan.tree_dir.clone(),
            repo: plan.url.repo.to_string(),
        };
        // The clone is made and recorded; its context folder is a convenience on top.
        if let Err(error) = crate::skills::after_clone(&paths, &repo) {
            eprintln!("note: couldn't set up the repo's context folder: {error}");
        }
        warnings = crate::cache::repo_gaps(
            &app.host,
            &plan.path,
            std::path::Path::new(crate::cache::SYSTEM_BAZELRC),
        )
        .into_iter()
        .map(mori_core::cache::CacheGap::warning)
        .collect();
    }
    Ok(CloneResponse {
        warnings,
        ..response(&paths, &plan, dry_run)
    })
}

fn observe(
    paths: &Paths,
    db: &Database,
    url: &CloneUrl,
) -> Result<Observed, Box<dyn ErrorDetails>> {
    let repos = db.repos().map_err(boxed)?;
    let recorded = repos
        .iter()
        .map(|repo| CloneUrl::parse(&repo.remote).map(|url| url.repo))
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(boxed)?;
    Ok(Observed {
        path_exists: clone_path(paths, &url.repo).symlink_metadata().is_ok(),
        recorded,
        tree_dirs: repos.into_iter().map(|repo| repo.dir_name).collect(),
    })
}

fn clone(vcs: &impl Backend, plan: &ClonePlan) -> Result<(), Box<dyn ErrorDetails>> {
    // The host and owner directories under repos/ may be new; they stay if the clone fails.
    if let Some(parent) = plan.path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| {
            boxed(StoreError::Io {
                path: parent.to_path_buf(),
                source,
            })
        })?;
    }
    vcs.clone_as(plan.vcs, &plan.url.fetch, &plan.path, plan.colocate)
        .map_err(boxed)
}

/// Best effort: the clone already exists, so a failure here keeps it and says so.
fn record(
    db: &mut Database,
    plan: &ClonePlan,
    user: Option<&str>,
) -> Result<(), Box<dyn ErrorDetails>> {
    let repo = plan.url.repo.to_string();
    let lifetime = Lifetime::Pinned.to_string();
    // The clone is the person's own checkout, so it's theirs, by the same name task trees use.
    let owner = default_owner(user).unwrap_or_else(|_| BASE_TREE_OWNER.to_owned());
    db.record_clone(
        &NewRepo {
            remote: &repo,
            dir_name: &plan.tree_dir,
        },
        &NewTree {
            name: BASE_TREE_NAME,
            owner: &owner,
            task: None,
            lifetime: &lifetime,
        },
    )
    .map(|_| ())
    .map_err(|error| {
        boxed(RepoError::NotRecorded {
            repo: repo.clone(),
            path: plan.path.clone(),
            why: error.to_string(),
        })
    })
}

fn response(paths: &Paths, plan: &ClonePlan, dry_run: bool) -> CloneResponse {
    CloneResponse {
        repo: plan.url.repo.to_string(),
        path: plan.path.display().to_string(),
        fetch_url: plan.url.fetch.clone(),
        colocated: plan.colocate,
        tree_dir: plan.tree_dir.clone(),
        validate_only: dry_run,
        vcs: crate::api_vcs(plan.vcs).into(),
        context_dir: paths
            .root
            .join("context/projects")
            .join(&plan.tree_dir)
            .display()
            .to_string(),
        warnings: Vec::new(),
    }
}

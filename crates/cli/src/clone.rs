//! `mori clone`: observe, plan, then the VCS clone and a best-effort record of it.

use std::collections::BTreeSet;

use mori_api::v1alpha1::CloneResponse;
use mori_core::clone::{
    BASE_TREE_NAME, BASE_TREE_OWNER, ClonePlan, CloneUrl, Observed, clone_path, plan,
};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::paths::Paths;
use mori_core::tree::Lifetime;
use mori_core::tree_create::default_owner;
use mori_jj::JjCli;
use mori_store::StoreError;
use mori_store::database::Database;
use mori_store::records::{NewRepo, NewTree};

use crate::state::{self, boxed};

/// Runs `clone` against the real environment, disk and `jj` on `PATH`.
pub fn run(
    url: &str,
    dry_run: bool,
    jj_only: bool,
) -> Result<CloneResponse, Box<dyn ErrorDetails>> {
    let url = CloneUrl::parse(url).map_err(boxed)?;
    let paths = state::paths()?;
    let mut db = state::open_database(&paths)?;
    let observed = observe(&paths, &db, &url)?;
    let plan = plan(&paths, url, !jj_only, &observed).map_err(boxed)?;
    if !dry_run {
        clone(&plan)?;
        record(&mut db, &plan)?;
    }
    Ok(response(&plan, dry_run))
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

fn clone(plan: &ClonePlan) -> Result<(), Box<dyn ErrorDetails>> {
    // The host and owner directories under repos/ may be new; they stay if the clone fails.
    if let Some(parent) = plan.path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| {
            boxed(StoreError::Io {
                path: parent.to_path_buf(),
                source,
            })
        })?;
    }
    JjCli::from_path()
        .clone_repo(&plan.url.fetch, &plan.path, plan.colocate)
        .map_err(boxed)
}

/// Best effort: the clone already exists, so a failure here keeps it and says so.
fn record(db: &mut Database, plan: &ClonePlan) -> Result<(), Box<dyn ErrorDetails>> {
    let repo = plan.url.repo.to_string();
    let lifetime = Lifetime::Pinned.to_string();
    // The clone is the person's own checkout, so it's theirs, by the same name task trees use.
    let owner = default_owner(std::env::var("USER").ok().as_deref())
        .unwrap_or_else(|_| BASE_TREE_OWNER.to_owned());
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

fn response(plan: &ClonePlan, dry_run: bool) -> CloneResponse {
    CloneResponse {
        repo: plan.url.repo.to_string(),
        path: plan.path.display().to_string(),
        fetch_url: plan.url.fetch.clone(),
        colocated: plan.colocate,
        tree_dir: plan.tree_dir.clone(),
        validate_only: dry_run,
    }
}

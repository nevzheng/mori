//! `mori clone`: observe, plan, then the VCS clone and a best-effort record of it.

use std::collections::BTreeSet;

use mori_api::v1alpha1::CloneResponse;
use mori_core::clone::{
    BASE_TREE_NAME, BASE_TREE_OWNER, ClonePlan, CloneUrl, Observed, clone_path, plan,
};
use mori_core::error::{ConfigError, ErrorDetails, RepoError, RootSource};
use mori_core::paths::{Env, Paths};
use mori_core::tree::{Lifetime, Role};
use mori_jj::JjCli;
use mori_store::StoreError;
use mori_store::database::Database;
use mori_store::records::{NewRepo, NewTree};

/// Runs `clone` against the real environment, disk and `jj` on `PATH`.
pub fn run(
    url: &str,
    dry_run: bool,
    jj_only: bool,
) -> Result<CloneResponse, Box<dyn ErrorDetails>> {
    let url = CloneUrl::parse(url).map_err(boxed)?;
    let paths = Paths::resolve(&Env::from_vars(|name| std::env::var_os(name))).map_err(boxed)?;
    let mut db = open_database(&paths)?;
    let observed = observe(&paths, &db, &url)?;
    let plan = plan(&paths, url, !jj_only, &observed).map_err(boxed)?;
    if !dry_run {
        clone(&plan)?;
        record(&mut db, &plan)?;
    }
    Ok(response(&plan, dry_run))
}

fn open_database(paths: &Paths) -> Result<Database, Box<dyn ErrorDetails>> {
    if paths.database.symlink_metadata().is_err() {
        return Err(boxed(ConfigError::NotInitialized {
            database: paths.database.clone(),
        }));
    }
    let db = Database::open(&paths.database).map_err(boxed)?;
    let recorded = db.root().map_err(boxed)?;
    if recorded != paths.root {
        return Err(boxed(ConfigError::RootMismatch {
            recorded,
            effective: paths.root.clone(),
            recorded_in: RootSource::Database,
        }));
    }
    Ok(db)
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
    db.record_clone(
        &NewRepo {
            remote: &repo,
            dir_name: &plan.tree_dir,
        },
        &NewTree {
            name: BASE_TREE_NAME,
            role: Role::Base.as_str(),
            owner: BASE_TREE_OWNER,
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

fn boxed(error: impl ErrorDetails + 'static) -> Box<dyn ErrorDetails> {
    Box::new(error)
}

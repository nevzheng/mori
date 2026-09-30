//! `mori tree create`: observe, plan, then `jj workspace add` and a best-effort record of it.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::Path;

use mori_api::v1alpha1::{CreateTreeResponse, Tree};
use mori_core::clone::{CloneUrl, RepoId, clone_path};
use mori_core::error::{ErrorDetails, TreeError};
use mori_core::forest::Workspaces;
use mori_core::paths::Paths;
use mori_core::tree::Lifetime;
use mori_core::tree_create::{DEFAULT_FROM, Observed, Request, TreePlan, default_owner, plan};
use mori_jj::JjCli;
use mori_store::StoreError;
use mori_store::database::Database;
use mori_store::records::NewTree;

use crate::state::{self, boxed};

/// What `mori tree create` was asked for.
pub struct CreateArgs {
    pub repo: String,
    pub task: String,
    pub agent: Option<String>,
    pub lifetime: Option<Lifetime>,
    pub from: Option<String>,
    pub dry_run: bool,
}

/// Runs `tree create` against the real environment, disk and `jj` on `PATH`.
pub fn create(args: CreateArgs) -> Result<CreateTreeResponse, Box<dyn ErrorDetails>> {
    let repo = CloneUrl::parse(&args.repo).map_err(boxed)?.repo;
    let owner = owner(args.agent)?;
    let paths = state::paths()?;
    let mut db = state::open_database(&paths)?;
    let policy = state::tree_policy(&paths)?;
    let jj = JjCli::from_path();
    let (repo_id, observed) = observe(&paths, &db, &jj, &repo)?;
    let request = Request {
        repo,
        task: args.task,
        owner,
        lifetime: args.lifetime,
        from: args.from.unwrap_or_else(|| DEFAULT_FROM.to_owned()),
    };
    let plan = plan(&paths, &policy, request, &observed).map_err(boxed)?;
    let mut id = String::new();
    if !args.dry_run {
        let clone = clone_path(&paths, &plan.repo);
        if let Some(parent) = plan.path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| {
                boxed(StoreError::Io {
                    path: parent.to_path_buf(),
                    source,
                })
            })?;
        }
        jj.add_workspace(&clone, &plan.name, &plan.path, &plan.from)
            .map_err(boxed)?;
        id = record(&mut db, repo_id.as_deref().unwrap_or_default(), &plan)?;
    }
    Ok(response(&plan, id, args.dry_run))
}

/// Who is acting: `--agent` if given, else the login name.
pub fn owner(agent: Option<String>) -> Result<String, Box<dyn ErrorDetails>> {
    match agent {
        Some(agent) => Ok(agent),
        None => default_owner(std::env::var("USER").ok().as_deref()).map_err(boxed),
    }
}

/// Looks at the repo's record, its trees, its workspaces and its `trees/` directory. Reads only.
fn observe(
    paths: &Paths,
    db: &Database,
    jj: &JjCli,
    repo_id: &RepoId,
) -> Result<(Option<String>, Observed), Box<dyn ErrorDetails>> {
    let Some(repo) = db.repo(&repo_id.to_string()).map_err(boxed)? else {
        return Ok((None, Observed::default()));
    };
    let recorded = db
        .trees(&repo.id)
        .map_err(boxed)?
        .into_iter()
        .map(|tree| tree.name)
        .collect();
    let workspaces = jj
        .list(&clone_path(paths, repo_id))
        .map_err(boxed)?
        .into_iter()
        .map(|workspace| workspace.name)
        .collect();
    let entries = dir_entries(&paths.trees().join(&repo.dir_name))?;
    let observed = Observed {
        tree_dir: Some(repo.dir_name),
        recorded,
        workspaces,
        entries,
    };
    Ok((Some(repo.id), observed))
}

/// The names in `dir`, or none if it doesn't exist yet.
fn dir_entries(dir: &Path) -> Result<BTreeSet<String>, Box<dyn ErrorDetails>> {
    let io = |source| {
        boxed(StoreError::Io {
            path: dir.to_path_buf(),
            source,
        })
    };
    match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
            .collect::<Result<_, _>>()
            .map_err(io),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(BTreeSet::new()),
        Err(error) => Err(io(error)),
    }
}

/// Best effort: the workspace already exists, so a failure here keeps it and says so.
fn record(
    db: &mut Database,
    repo_id: &str,
    plan: &TreePlan,
) -> Result<String, Box<dyn ErrorDetails>> {
    let lifetime = plan.lifetime.to_string();
    db.record_tree(
        repo_id,
        &NewTree {
            name: &plan.name,
            role: plan.role.as_str(),
            owner: &plan.owner,
            task: Some(&plan.task),
            lifetime: &lifetime,
        },
    )
    .map(|tree| tree.id)
    .map_err(|error| {
        boxed(TreeError::NotRecorded {
            name: plan.name.clone(),
            path: plan.path.clone(),
            why: error.to_string(),
        })
    })
}

fn response(plan: &TreePlan, id: String, dry_run: bool) -> CreateTreeResponse {
    CreateTreeResponse {
        tree: Some(Tree {
            id,
            repo: plan.repo.to_string(),
            name: plan.name.clone(),
            path: plan.path.display().to_string(),
            role: plan.role.as_str().to_owned(),
            owner: plan.owner.clone(),
            task: plan.task.clone(),
            lifetime: plan.lifetime.to_string(),
        }),
        from: plan.from.clone(),
        validate_only: dry_run,
    }
}

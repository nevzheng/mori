//! `mori where`: which repo and tree a path is in, from the root layout and mori's records.

use std::path::{Path, PathBuf};

use mori_api::v1alpha1::{ResolveResponse, Tree, resolve_response::Kind, tree_row::Status};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, RepoId, clone_path};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::paths::Paths;
use mori_core::place::{Place, locate};
use mori_core::vcs::Forge;
use mori_store::database::Database;
use mori_store::records::{RepoRecord, TreeRecord};

use crate::state::{self, boxed};
use crate::{App, Backend};

/// Runs `mori where` for `path` (absolute). Reads only: no VCS call.
///
/// # Errors
///
/// [`RepoError::NotInForest`] if `path` is outside the root, or a failure of the database.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    path: &Path,
) -> Result<ResolveResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths(&app.host)?;
    let db = state::open_database(&paths)?;
    // Resolve links, so neither the root nor the path can lead out through one.
    let root = paths
        .root
        .canonicalize()
        .unwrap_or_else(|_| paths.root.clone());
    let resolved = resolve(path);
    let canonical = Paths {
        root: root.clone(),
        ..paths.clone()
    };
    let not_in_forest = || {
        boxed(RepoError::NotInForest {
            path: path.to_path_buf(),
            root: paths.root.clone(),
        })
    };
    let place = locate(&canonical, &resolved).ok_or_else(not_in_forest)?;
    let mut response = ResolveResponse {
        root: paths.root.display().to_string(),
        kind: Kind::Root.into(),
        ..ResolveResponse::default()
    };
    let repos = db.repos().map_err(boxed)?;
    let (repo, tree_name) = match place {
        Place::Root => return Ok(response),
        Place::Clone(id) => {
            response.kind = Kind::Clone.into();
            let repo = repos.into_iter().find(|repo| repo.remote == id.to_string());
            (repo, Some(BASE_TREE_NAME.to_owned()))
        }
        Place::Tree { dir, name } => {
            response.kind = Kind::Tree.into();
            (
                repos.into_iter().find(|repo| repo.dir_name == dir),
                Some(name),
            )
        }
        Place::Context(dir) => {
            response.kind = Kind::Context.into();
            let repo = dir.and_then(|dir| repos.into_iter().find(|repo| repo.dir_name == dir));
            (repo, None)
        }
    };
    let Some(repo) = repo else {
        return Ok(response);
    };
    response.repo.clone_from(&repo.remote);
    response.context_dir = paths
        .root
        .join("context/projects")
        .join(&repo.dir_name)
        .display()
        .to_string();
    if let Some(name) = tree_name {
        let record = tree_record(&db, &repo, &name)?;
        response.status = if record.is_some() {
            Status::Tree
        } else {
            Status::Foreign
        }
        .into();
        response.tree = Some(tree(&paths, &repo, &name, record)?);
    }
    Ok(response)
}

fn tree_record(
    db: &Database,
    repo: &RepoRecord,
    name: &str,
) -> Result<Option<TreeRecord>, Box<dyn ErrorDetails>> {
    Ok(db
        .trees(&repo.id)
        .map_err(boxed)?
        .into_iter()
        .find(|tree| tree.name == name))
}

fn tree(
    paths: &Paths,
    repo: &RepoRecord,
    name: &str,
    record: Option<TreeRecord>,
) -> Result<Tree, Box<dyn ErrorDetails>> {
    let id: RepoId = CloneUrl::parse(&repo.remote).map_err(boxed)?.repo;
    let path = if name == BASE_TREE_NAME {
        clone_path(paths, &id)
    } else {
        paths.trees().join(&repo.dir_name).join(name)
    };
    let mut tree = Tree {
        repo: repo.remote.clone(),
        name: name.to_owned(),
        path: path.display().to_string(),
        ..Tree::default()
    };
    if let Some(record) = record {
        tree.id = record.id;
        tree.owner = record.owner;
        tree.task = record.task.unwrap_or_default();
        tree.lifetime = record.lifetime;
        tree.purpose = record.purpose.unwrap_or_default();
    }
    Ok(tree)
}

/// `path` with links resolved, even if it doesn't exist yet: the nearest part that exists is
/// resolved, and the rest is kept as given.
fn resolve(path: &Path) -> PathBuf {
    let mut existing = path;
    let mut rest = Vec::new();
    loop {
        if let Ok(resolved) = existing.canonicalize() {
            return rest.iter().rev().fold(resolved, |acc, part| acc.join(part));
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_owned());
                existing = parent;
            }
            _ => return path.to_path_buf(),
        }
    }
}

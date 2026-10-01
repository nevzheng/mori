//! `mori tree set`: change a tree's record (purpose, lifetime, owner). Never touches the tree.

use mori_api::v1alpha1::Tree;
use mori_core::clone::{BASE_TREE_NAME, clone_path};
use mori_core::error::{ErrorDetails, RepoError, TreeError};
use mori_core::tree::{Lifetime, validate_purpose};
use mori_core::vcs::Forge;
use mori_store::records::TreeUpdate;

use crate::state::{self, boxed};
use crate::{App, Backend};

/// What `mori tree set` was asked to change. `None` leaves a field alone; an empty purpose
/// clears it.
pub struct SetArgs {
    pub repo: String,
    pub name: String,
    pub purpose: Option<String>,
    pub lifetime: Option<Lifetime>,
    pub owner: Option<String>,
}

/// Runs `tree set`, and returns the tree as recorded afterwards.
///
/// # Errors
///
/// [`TreeError::NothingToChange`] if nothing was asked for, [`TreeError::BaseTree`] for a
/// lifetime on the clone itself (always pinned), [`TreeError::PurposeInvalid`],
/// [`TreeError::NotFound`], or a failure of the database.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    args: SetArgs,
) -> Result<Tree, Box<dyn ErrorDetails>> {
    if args.purpose.is_none() && args.lifetime.is_none() && args.owner.is_none() {
        return Err(boxed(TreeError::NothingToChange));
    }
    if args.lifetime.is_some() && args.name == BASE_TREE_NAME {
        return Err(boxed(TreeError::BaseTree { name: args.name }));
    }
    let purpose = match args.purpose.as_deref() {
        None => None,
        Some(purpose) if purpose.trim().is_empty() => Some(None),
        Some(purpose) => Some(Some(validate_purpose(purpose).map_err(boxed)?)),
    };
    let paths = state::paths(&app.host)?;
    let mut db = state::open_database(&paths)?;
    let repo_id = state::repo_id(&db, &args.repo)?;
    let repo = db
        .repo(&repo_id.to_string())
        .map_err(boxed)?
        .ok_or_else(|| {
            boxed(RepoError::NotManaged {
                repo: repo_id.to_string(),
            })
        })?;
    let lifetime = args.lifetime.map(|lifetime| lifetime.to_string());
    let update = TreeUpdate {
        purpose: purpose.as_ref().map(Option::as_deref),
        lifetime: lifetime.as_deref(),
        owner: args.owner.as_deref(),
    };
    let record = db
        .update_tree(&repo.id, &args.name, &update)
        .map_err(boxed)?
        .ok_or_else(|| {
            boxed(TreeError::NotFound {
                name: args.name.clone(),
            })
        })?;
    let path = if record.name == BASE_TREE_NAME {
        clone_path(&paths, &repo_id)
    } else {
        paths.trees().join(&repo.dir_name).join(&record.name)
    };
    Ok(Tree {
        id: record.id,
        repo: repo.remote,
        name: record.name,
        path: path.display().to_string(),
        owner: record.owner,
        task: record.task.unwrap_or_default(),
        lifetime: record.lifetime,
        purpose: record.purpose.unwrap_or_default(),
    })
}

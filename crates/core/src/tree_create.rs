//! `mori tree create`: a new task tree for one piece of work.
//!
//! The adapter looks up the repo's record, the tree names mori recorded, the workspaces the VCS
//! reports and what is already under the repo's `trees/` directory; [`plan`] turns that into a
//! [`TreePlan`] or a refusal. The adapter then adds the workspace and records the tree. Recording
//! is best effort, as with `clone`: if it fails, the workspace stays.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::clone::RepoId;
use crate::error::{Code, ErrorDetails, RepoError, TreeError};
use crate::paths::Paths;
use crate::tree::{Lifetime, Role, TreePolicy, validate_purpose};

/// Where a new tree starts when no revision is given: a new change on top of trunk.
pub const DEFAULT_FROM: &str = "trunk()";

/// What someone asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// The repo.
    pub repo: RepoId,
    /// A short slug for the work, e.g. `fix-login`.
    pub task: String,
    /// Who the tree is for: an agent such as `claude`, or the person.
    pub owner: String,
    /// A lifetime instead of the policy's default.
    pub lifetime: Option<Lifetime>,
    /// The revision the new tree starts from (a jj revset).
    pub from: String,
    /// What the tree is for, if given.
    pub purpose: Option<String>,
}

/// What exists before `tree create` runs, as seen by an adapter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observed {
    /// The repo's directory under `trees/`, if mori manages the repo.
    pub tree_dir: Option<String>,
    /// The names of the trees mori recorded for the repo.
    pub recorded: BTreeSet<String>,
    /// The workspaces the VCS reports for the repo's clone.
    pub workspaces: BTreeSet<String>,
    /// The entries already in the repo's `trees/` directory.
    pub entries: BTreeSet<String>,
}

/// What `tree create` will do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreePlan {
    /// The repo.
    pub repo: RepoId,
    /// The tree's name, which is also its workspace name.
    pub name: String,
    /// Where the tree goes: `<root>/trees/<repo dir>/<name>`.
    pub path: PathBuf,
    /// Always [`Role::Task`]: `tree create` never makes a base tree.
    pub role: Role,
    /// Who the tree is for.
    pub owner: String,
    /// The task.
    pub task: String,
    /// When it may go.
    pub lifetime: Lifetime,
    /// The revision it starts from.
    pub from: String,
    /// What it is for, checked and trimmed.
    pub purpose: Option<String>,
}

/// The owner to use when none is given: the person's login name, lowercased, so it can be part
/// of a tree name.
///
/// # Errors
///
/// [`TreeError::OwnerUnknown`] when there is no login name to use.
pub fn default_owner(user: Option<&str>) -> Result<String, TreeError> {
    user.map(str::trim)
        .filter(|user| !user.is_empty())
        .map(str::to_ascii_lowercase)
        .ok_or(TreeError::OwnerUnknown)
}

/// Plans `tree create` from the request, the tree policy and what the adapter observed.
///
/// # Errors
///
/// [`RepoError::NotManaged`] if mori doesn't manage the repo; [`TreeError::NameInvalid`] if the
/// name made from the template can't be used; [`TreeError::TreeExists`] if mori already has a
/// tree of that name; [`TreeError::WorkspaceExists`] if the VCS has a workspace of that name that
/// mori didn't make; [`TreeError::PathExists`] if something is at the tree's path. Nothing should
/// be created on any of them.
pub fn plan(
    paths: &Paths,
    policy: &TreePolicy,
    request: Request,
    observed: &Observed,
) -> Result<TreePlan, PlanError> {
    let tree_dir = observed
        .tree_dir
        .as_ref()
        .ok_or_else(|| RepoError::NotManaged {
            repo: request.repo.to_string(),
        })?;
    let name = policy.name.render(&request.owner, &request.task)?;
    if observed.recorded.contains(&name) {
        return Err(TreeError::TreeExists { name }.into());
    }
    if observed.workspaces.contains(&name) {
        return Err(TreeError::WorkspaceExists { name }.into());
    }
    let path = paths.trees().join(tree_dir).join(&name);
    if observed.entries.contains(&name) {
        return Err(TreeError::PathExists { path }.into());
    }
    Ok(TreePlan {
        repo: request.repo,
        name,
        path,
        role: Role::Task,
        owner: request.owner,
        task: request.task,
        lifetime: policy.lifetime(Role::Task, request.lifetime),
        from: request.from,
        purpose: request
            .purpose
            .as_deref()
            .map(validate_purpose)
            .transpose()?,
    })
}

/// Why `tree create` can't go ahead: about the repo, or about the tree.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlanError {
    /// About the repo.
    #[error(transparent)]
    Repo(#[from] RepoError),
    /// About the tree.
    #[error(transparent)]
    Tree(#[from] TreeError),
}

impl ErrorDetails for PlanError {
    fn code(&self) -> Code {
        match self {
            Self::Repo(error) => error.code(),
            Self::Tree(error) => error.code(),
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            Self::Repo(error) => error.reason(),
            Self::Tree(error) => error.reason(),
        }
    }

    fn domain(&self) -> &'static str {
        match self {
            Self::Repo(error) => error.domain(),
            Self::Tree(error) => error.domain(),
        }
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::Repo(error) => error.metadata(),
            Self::Tree(error) => error.metadata(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clone::CloneUrl;
    use crate::paths::Env;

    fn paths() -> Paths {
        Paths::resolve(&Env {
            home: Some(PathBuf::from("/home/acme")),
            ..Env::default()
        })
        .unwrap()
    }

    fn request() -> Request {
        Request {
            repo: CloneUrl::parse("github.com/acme/widget").unwrap().repo,
            task: "fix-login".to_owned(),
            owner: "claude".to_owned(),
            lifetime: None,
            from: DEFAULT_FROM.to_owned(),
            purpose: None,
        }
    }

    fn managed() -> Observed {
        Observed {
            tree_dir: Some("widget".to_owned()),
            recorded: BTreeSet::from(["default".to_owned()]),
            workspaces: BTreeSet::from(["default".to_owned()]),
            entries: BTreeSet::new(),
        }
    }

    fn reason(error: &PlanError) -> &'static str {
        error.reason()
    }

    #[test]
    fn a_task_tree_goes_under_its_repos_tree_dir() {
        let plan = plan(&paths(), &TreePolicy::default(), request(), &managed()).unwrap();

        assert_eq!(plan.name, "claude-fix-login");
        assert_eq!(
            plan.path,
            PathBuf::from("/home/acme/mori/trees/widget/claude-fix-login")
        );
        assert_eq!(plan.role, Role::Task);
        assert_eq!(plan.lifetime, Lifetime::TaskDone);
        assert_eq!(plan.from, "trunk()");
    }

    #[test]
    fn a_requested_lifetime_wins() {
        let request = Request {
            lifetime: Some(Lifetime::Pinned),
            task: "lead".to_owned(),
            ..request()
        };

        let plan = plan(&paths(), &TreePolicy::default(), request, &managed()).unwrap();

        assert_eq!(plan.name, "claude-lead");
        assert_eq!(plan.lifetime, Lifetime::Pinned);
    }

    #[test]
    fn a_repo_mori_doesnt_manage_is_refused() {
        let error = plan(
            &paths(),
            &TreePolicy::default(),
            request(),
            &Observed::default(),
        )
        .unwrap_err();

        assert_eq!(reason(&error), "REPO_NOT_MANAGED");
    }

    #[test]
    fn a_bad_task_is_refused() {
        let request = Request {
            task: "../etc".to_owned(),
            ..request()
        };

        let error = plan(&paths(), &TreePolicy::default(), request, &managed()).unwrap_err();

        assert_eq!(reason(&error), "TREE_NAME_INVALID");
    }

    #[test]
    fn taken_names_are_refused() {
        for (observed, expected) in [
            (
                Observed {
                    recorded: BTreeSet::from(["claude-fix-login".to_owned()]),
                    ..managed()
                },
                "TREE_EXISTS",
            ),
            (
                Observed {
                    workspaces: BTreeSet::from(["claude-fix-login".to_owned()]),
                    ..managed()
                },
                "WORKSPACE_EXISTS",
            ),
            (
                Observed {
                    entries: BTreeSet::from(["claude-fix-login".to_owned()]),
                    ..managed()
                },
                "PATH_EXISTS",
            ),
        ] {
            let error = plan(&paths(), &TreePolicy::default(), request(), &observed).unwrap_err();

            assert_eq!(reason(&error), expected);
        }
    }

    #[test]
    fn the_owner_defaults_to_the_login_name() {
        assert_eq!(default_owner(Some("Nevin")).unwrap(), "nevin");

        let error = default_owner(Some(" ")).unwrap_err();
        assert_eq!(error.code(), Code::FailedPrecondition);
        assert_eq!(error.reason(), "OWNER_UNKNOWN");
    }
}

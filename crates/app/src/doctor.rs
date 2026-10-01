//! `mori doctor`: gather the facts read-only, let `mori-core` judge them, and report.

use std::collections::BTreeSet;
use std::path::Path;

use mori_api::v1alpha1::{DoctorResponse, Finding as ApiFinding, finding::Severity as ApiSeverity};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, clone_path};
use mori_core::doctor::{Facts, Finding, Known, RepoFacts, Severity, TreeFacts, check, summary};
use mori_core::error::{ErrorDetails, RepoError};
use mori_core::forest::{Entry, reconcile};
use mori_core::paths::Paths;
use mori_core::vcs::{Forge, VcsKind};
use mori_store::database::Database;
use mori_store::records::RepoRecord;

use crate::routed::kind_of;
use crate::state::{self, boxed};
use crate::{App, Backend, skills};

/// Runs `mori doctor` for the whole root, or only `repo`. Reads only.
///
/// # Errors
///
/// The plan's refusal, or a failure of the disk, the database or an adapter, with its code and
/// reason.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    repo: Option<&str>,
) -> Result<DoctorResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths(&app.host)?;
    let db = state::open_database(&paths)?;
    let mut records = db.repos().map_err(boxed)?;
    if let Some(repo) = repo {
        let wanted = CloneUrl::parse(repo).map_err(boxed)?.repo.to_string();
        records.retain(|record| record.remote == wanted);
        if records.is_empty() {
            return Err(boxed(RepoError::NotManaged { repo: wanted }));
        }
    }
    let repos = records
        .iter()
        .map(|record| repo_facts(&paths, &db, &app.vcs, record))
        .collect::<Result<Vec<_>, _>>()?;
    let needed: Vec<VcsKind> = records
        .iter()
        .filter_map(|record| {
            let clone = clone_path(&paths, &CloneUrl::parse(&record.remote).ok()?.repo);
            clone.exists().then(|| kind_of(&clone))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let unrecorded_clones = if repo.is_some() {
        Vec::new()
    } else {
        let managed = records.iter().map(|record| record.remote.clone()).collect();
        let clones = mori_store::init::find_clones(&paths.repos()).map_err(boxed)?;
        mori_core::init::unmanaged(&paths, &clones, &managed)
            .into_iter()
            .map(|unmanaged| (unmanaged.repo, unmanaged.path))
            .collect()
    };
    let facts = Facts {
        tools_missing: app.vcs.missing_tools(&needed),
        repos,
        unrecorded_clones,
        index_stale: skills::index_stale(&paths)?,
    };
    let findings = check(&facts);
    Ok(DoctorResponse {
        summary: summary(&findings),
        findings: findings.iter().map(to_api).collect(),
    })
}

/// What doctor needs to know about one recorded repo. A missing clone stops the reading there.
fn repo_facts(
    paths: &Paths,
    db: &Database,
    vcs: &impl Backend,
    record: &RepoRecord,
) -> Result<RepoFacts, Box<dyn ErrorDetails>> {
    let id = CloneUrl::parse(&record.remote).map_err(boxed)?.repo;
    let clone = clone_path(paths, &id);
    let mut facts = RepoFacts {
        repo: record.remote.clone(),
        clone: clone.clone(),
        clone_exists: clone.is_dir(),
        context_folder_exists: paths
            .root
            .join("context/projects")
            .join(&record.dir_name)
            .is_dir(),
        ..RepoFacts::default()
    };
    if !facts.clone_exists {
        return Ok(facts);
    }
    facts.backend_changed = clone.join(".jj").is_dir() && has_git_worktrees(&clone);
    let workspaces = vcs.list(&clone).map_err(boxed)?;
    let tree_dir = paths.trees().join(&record.dir_name);
    let in_use: BTreeSet<_> = workspaces
        .iter()
        .map(|workspace| workspace.root.clone())
        .collect();
    let records = db.trees(&record.id).map_err(boxed)?;
    let recorded: BTreeSet<String> = records.iter().map(|tree| tree.name.clone()).collect();
    facts.stray_dirs = subdirs(&tree_dir)
        .into_iter()
        .filter(|dir| {
            let name = dir
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            !in_use.iter().any(|root| same_path(root, dir))
                && !name.is_some_and(|name| recorded.contains(&name))
        })
        .collect();
    let expected = |name: &str| {
        if name == BASE_TREE_NAME {
            clone.clone()
        } else {
            tree_dir.join(name)
        }
    };
    facts.trees = reconcile(records, workspaces, |tree| tree.name.as_str())
        .into_iter()
        .map(|entry| {
            let (name, path, known) = match entry {
                Entry::Tree { record, workspace } => {
                    // jj forgets the root of a workspace whose directory was deleted.
                    let path = if workspace.root.as_os_str().is_empty() {
                        expected(&record.name)
                    } else {
                        workspace.root
                    };
                    (record.name, path, Known::Both)
                }
                Entry::Missing { record } => {
                    let path = expected(&record.name);
                    (record.name, path, Known::RecordOnly)
                }
                Entry::Foreign { workspace } => (workspace.name, workspace.root, Known::VcsOnly),
            };
            TreeFacts {
                dir_exists: path.is_dir(),
                name,
                path,
                known,
            }
        })
        .collect();
    facts.conflicted_bookmarks = vcs.conflicted_bookmarks(&clone).map_err(boxed)?;
    Ok(facts)
}

/// Whether the clone's git repository has linked worktrees: mori's git trees.
fn has_git_worktrees(clone: &Path) -> bool {
    !subdirs(&clone.join(".git/worktrees")).is_empty()
}

/// The directories directly in `dir`; none if it doesn't exist.
fn subdirs(dir: &Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default()
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn to_api(finding: &Finding) -> ApiFinding {
    ApiFinding {
        code: finding.code.as_str().to_owned(),
        severity: match finding.severity() {
            Severity::Info => ApiSeverity::Info,
            Severity::Warn => ApiSeverity::Warn,
            Severity::Problem => ApiSeverity::Problem,
        }
        .into(),
        subject: finding.subject.to_string(),
        message: finding.message.clone(),
        fix: finding.fix.clone(),
        auto_fixable: finding.auto_fixable(),
    }
}

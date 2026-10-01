//! `mori doctor`: gather the facts read-only, let `mori-core` judge them, and report.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mori_api::v1alpha1::{
    DoctorResponse, Finding as ApiFinding, Fixed, finding::Severity as ApiSeverity,
};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, clone_path};
use mori_core::doctor::{
    Code, Facts, Finding, Known, RepoFacts, Severity, Subject, TreeFacts, check, summary,
};
use mori_core::error::{DoctorError, ErrorDetails, RepoError};
use mori_core::forest::{Entry, reconcile};
use mori_core::paths::Paths;
use mori_core::skills::Mode;
use mori_core::vcs::{Forge, VcsKind};
use mori_store::database::Database;
use mori_store::gc::new_id;
use mori_store::records::RepoRecord;

use crate::gc::Context;
use crate::gc_apply;
use crate::routed::kind_of;
use crate::state::{self, boxed};
use crate::{App, Backend, Host, cache, skills};

/// What `--fix` was asked for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fix {
    /// Confirmed with `--yes`.
    pub yes: bool,
    /// Check everything and repair nothing.
    pub dry_run: bool,
}

/// Runs `mori doctor` for the whole root, or only `repo`. Reads only, unless `fix` asks it to
/// repair the auto-fixable findings: each removal pins the tree's commit and is journalled, as in
/// `mori gc --apply`, so `mori restore` brings it back.
///
/// # Errors
///
/// [`DoctorError::NotConfirmed`] for `--fix` without `--yes`, or a failure of the disk, the
/// database or an adapter, with its code and reason.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    repo: Option<&str>,
    fix: Option<Fix>,
) -> Result<DoctorResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths(&app.host)?;
    let mut db = state::open_database(&paths)?;
    let mut findings = check(&gather(app, &paths, &db, repo)?);
    let mut fixed = Vec::new();
    let validate_only = fix.is_some_and(|fix| fix.dry_run);
    if let Some(fix) = fix {
        let fixable: Vec<&Finding> = findings.iter().filter(|f| f.auto_fixable()).collect();
        if !fix.yes && !fix.dry_run && !fixable.is_empty() {
            return Err(boxed(DoctorError::NotConfirmed {
                count: fixable.len(),
            }));
        }
        let mut synced = false;
        for finding in fixable {
            let journal_entry = if fix.dry_run {
                String::new()
            } else {
                repair(app, &paths, &mut db, finding, &mut synced)?
            };
            fixed.push(Fixed {
                code: finding.code.as_str().to_owned(),
                subject: finding.subject.to_string(),
                journal_entry,
            });
        }
        if !fix.dry_run && !fixed.is_empty() {
            findings = check(&gather(app, &paths, &db, repo)?);
        }
    }
    Ok(DoctorResponse {
        summary: summary(&findings),
        findings: findings.iter().map(to_api).collect(),
        fixed,
        validate_only,
    })
}

/// Repairs one auto-fixable finding, and returns its journal entry if it removed anything.
fn repair<V: Backend, F: Forge>(
    app: &App<V, F>,
    paths: &Paths,
    db: &mut Database,
    finding: &Finding,
    synced: &mut bool,
) -> Result<String, Box<dyn ErrorDetails>> {
    match (&finding.code, &finding.subject) {
        (Code::TreeDirGone | Code::WorkspaceGone, Subject::Tree { repo, name }) => {
            let Some(repo) = db.repo(repo).map_err(boxed)? else {
                return Ok(String::new());
            };
            let Some(record) = db
                .trees(&repo.id)
                .map_err(boxed)?
                .into_iter()
                .find(|tree| &tree.name == name)
            else {
                return Ok(String::new());
            };
            let context = Context {
                paths,
                vcs: &app.vcs,
                forge: None,
                now: app.host.now,
                lru_max: None,
            };
            let entry_id = new_id("j");
            let has_workspace = finding.code == Code::TreeDirGone;
            let entry = gc_apply::remove(&context, db, &repo, &record, has_workspace, &entry_id)?;
            mori_store::gc::append_journal(paths, &entry).map_err(boxed)?;
            Ok(entry_id)
        }
        (Code::ContextFolderMissing | Code::ContextIndexStale, _) => {
            // One sync repairs every context folder and index at once.
            if !*synced {
                skills::run(paths, Mode::Sync, false)?;
                *synced = true;
            }
            Ok(String::new())
        }
        _ => Ok(String::new()),
    }
}

/// Everything doctor looks at, for the whole root or only `repo`. Reads only.
fn gather<V: Backend, F: Forge>(
    app: &App<V, F>,
    paths: &Paths,
    db: &Database,
    repo: Option<&str>,
) -> Result<Facts, Box<dyn ErrorDetails>> {
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
        .map(|record| repo_facts(&app.host, paths, db, &app.vcs, record))
        .collect::<Result<Vec<_>, _>>()?;
    let needed: Vec<VcsKind> = records
        .iter()
        .filter_map(|record| {
            let clone = clone_path(paths, &CloneUrl::parse(&record.remote).ok()?.repo);
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
        mori_core::init::unmanaged(paths, &clones, &managed)
            .into_iter()
            .map(|unmanaged| (unmanaged.repo, unmanaged.path))
            .collect()
    };
    Ok(Facts {
        tools_missing: app.vcs.missing_tools(&needed),
        repos,
        unrecorded_clones,
        index_stale: skills::index_stale(paths)?,
        ccache_without_base_dir: cache::ccache_lacks_base_dir(
            &app.host,
            &cache::SYSTEM_CCACHE_CONFS.map(PathBuf::from),
        ),
    })
}

/// What doctor needs to know about one recorded repo. A missing clone stops the reading there.
fn repo_facts(
    host: &Host,
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
    facts.cache_gaps = cache::repo_gaps(host, &clone, Path::new(cache::SYSTEM_BAZELRC));
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

//! `mori doctor`: drift between mori's records, the VCS and the disk, and what to do about it.
//!
//! The adapter gathers [`Facts`] in one read-only pass; [`check`] turns them into [`Finding`]s,
//! each with a stable code, a severity and the command that fixes it. A finding is auto-fixable
//! only when it concerns what mori created, the fix can't lose work (or pins and journals what it
//! removes), and there is exactly one right fix.

use std::path::PathBuf;

use crate::vcs::VcsKind;

/// How bad a finding is, in increasing order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Worth knowing; nothing is wrong.
    Info,
    /// Something is off, and mori still works.
    Warn,
    /// Something mori or the VCS will trip over.
    Problem,
}

/// What a finding is about.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subject {
    /// The root as a whole.
    Root,
    /// A repo, e.g. `github.com/acme/widget`.
    Repo(String),
    /// A tree of a repo.
    Tree {
        /// The repo.
        repo: String,
        /// The tree's name.
        name: String,
    },
    /// A path mori found but has no record of.
    Path(PathBuf),
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root => f.write_str("root"),
            Self::Repo(repo) => f.write_str(repo),
            Self::Tree { repo, name } => write!(f, "{repo} {name}"),
            Self::Path(path) => write!(f, "{}", path.display()),
        }
    }
}

/// Every kind of drift doctor knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    /// The VCS a clone needs isn't on `PATH`.
    ToolNotFound,
    /// A recorded clone's directory is gone.
    CloneMissing,
    /// A clone under `repos/` that mori has no record of.
    CloneNotRecorded,
    /// A git clone mori made now has a `.jj` folder, while its git worktrees still exist.
    BackendChanged,
    /// A recorded tree's workspace or worktree exists, but its directory doesn't.
    TreeDirGone,
    /// A recorded tree has no workspace or worktree.
    WorkspaceGone,
    /// A directory under `trees/` that no workspace uses.
    DirWithoutWorkspace,
    /// A workspace or worktree mori didn't create.
    ForeignWorkspace,
    /// A jj bookmark with several targets (`main??`).
    ConflictedBookmark,
    /// A clone without its `context/projects/<repo>/` folder.
    ContextFolderMissing,
    /// A generated `llms.txt` that differs from what mori would write.
    ContextIndexStale,
}

impl Code {
    /// The stable code shown in reports and JSON.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ToolNotFound => "TOOL_NOT_FOUND",
            Self::CloneMissing => "CLONE_MISSING",
            Self::CloneNotRecorded => "CLONE_NOT_RECORDED",
            Self::BackendChanged => "BACKEND_CHANGED",
            Self::TreeDirGone => "TREE_DIR_GONE",
            Self::WorkspaceGone => "WORKSPACE_GONE",
            Self::DirWithoutWorkspace => "DIR_WITHOUT_WORKSPACE",
            Self::ForeignWorkspace => "FOREIGN_WORKSPACE",
            Self::ConflictedBookmark => "CONFLICTED_BOOKMARK",
            Self::ContextFolderMissing => "CONTEXT_FOLDER_MISSING",
            Self::ContextIndexStale => "CONTEXT_INDEX_STALE",
        }
    }

    /// How bad it is.
    #[must_use]
    pub fn severity(self) -> Severity {
        match self {
            Self::ToolNotFound
            | Self::CloneMissing
            | Self::BackendChanged
            | Self::TreeDirGone
            | Self::ConflictedBookmark => Severity::Problem,
            Self::WorkspaceGone
            | Self::DirWithoutWorkspace
            | Self::ContextFolderMissing
            | Self::ContextIndexStale => Severity::Warn,
            Self::CloneNotRecorded | Self::ForeignWorkspace => Severity::Info,
        }
    }

    /// Whether `mori doctor --fix --yes` repairs it.
    #[must_use]
    pub fn auto_fixable(self) -> bool {
        matches!(
            self,
            Self::TreeDirGone
                | Self::WorkspaceGone
                | Self::ContextFolderMissing
                | Self::ContextIndexStale
        )
    }
}

/// One thing doctor found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// What kind of drift.
    pub code: Code,
    /// What it is about.
    pub subject: Subject,
    /// What is wrong, in a sentence.
    pub message: String,
    /// What fixes it: a command, or what to do by hand.
    pub fix: String,
}

impl Finding {
    /// How bad it is.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.code.severity()
    }

    /// Whether `mori doctor --fix --yes` repairs it.
    #[must_use]
    pub fn auto_fixable(&self) -> bool {
        self.code.auto_fixable()
    }
}

/// Everything doctor looks at, gathered read-only by the adapter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    /// The VCS tools some clone needs that aren't on `PATH`.
    pub tools_missing: Vec<VcsKind>,
    /// Every repo mori recorded.
    pub repos: Vec<RepoFacts>,
    /// Clones under `repos/` mori has no record of: the repo and its path.
    pub unrecorded_clones: Vec<(String, PathBuf)>,
    /// Whether a generated `llms.txt` differs from what mori would write now.
    pub index_stale: bool,
}

/// One recorded repo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RepoFacts {
    /// The repo, e.g. `github.com/acme/widget`.
    pub repo: String,
    /// Where its clone should be.
    pub clone: PathBuf,
    /// Whether the clone's directory exists. When it doesn't, nothing else here was read.
    pub clone_exists: bool,
    /// A `.jj` folder appeared in a clone that still has git worktrees.
    pub backend_changed: bool,
    /// Every tree mori recorded and every workspace the VCS reports, matched by name.
    pub trees: Vec<TreeFacts>,
    /// Directories under the repo's `trees/` folder that no workspace uses.
    pub stray_dirs: Vec<PathBuf>,
    /// Bookmarks with several targets.
    pub conflicted_bookmarks: Vec<String>,
    /// Whether `context/projects/<repo>/` exists.
    pub context_folder_exists: bool,
}

/// Who knows about a tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Known {
    /// mori recorded it and the VCS has it: the healthy case.
    #[default]
    Both,
    /// mori recorded it; the VCS has no workspace or worktree of that name.
    RecordOnly,
    /// The VCS has it; mori didn't create it.
    VcsOnly,
}

/// One tree: recorded, reported by the VCS, or both.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeFacts {
    /// Its name.
    pub name: String,
    /// Where it is, or should be.
    pub path: PathBuf,
    /// Who knows about it.
    pub known: Known,
    /// Its directory exists.
    pub dir_exists: bool,
}

const FIX: &str = "mori doctor --fix --yes";

/// Judges the facts. Findings come worst first, then by subject.
#[must_use]
pub fn check(facts: &Facts) -> Vec<Finding> {
    let mut findings = Vec::new();
    for tool in &facts.tools_missing {
        findings.push(Finding {
            code: Code::ToolNotFound,
            subject: Subject::Root,
            message: format!("{tool} isn't on PATH, and a clone mori manages needs it"),
            fix: format!("install {tool}, or put it on PATH"),
        });
    }
    if facts.index_stale {
        findings.push(Finding {
            code: Code::ContextIndexStale,
            subject: Subject::Root,
            message: "a generated llms.txt is out of date".to_owned(),
            fix: format!("mori skills sync (or {FIX})"),
        });
    }
    for (repo, path) in &facts.unrecorded_clones {
        findings.push(Finding {
            code: Code::CloneNotRecorded,
            subject: Subject::Path(path.clone()),
            message: format!("a clone of {repo} that mori didn't make; mori leaves it alone"),
            fix: "nothing, unless it shouldn't be under repos/: then move it by hand".to_owned(),
        });
    }
    for repo in &facts.repos {
        check_repo(repo, &mut findings);
    }
    findings.sort_by(|a, b| {
        b.severity()
            .cmp(&a.severity())
            .then_with(|| a.subject.cmp(&b.subject))
    });
    findings
}

fn check_repo(repo: &RepoFacts, findings: &mut Vec<Finding>) {
    let subject = || Subject::Repo(repo.repo.clone());
    let mut push = |code, subject, message: String, fix: String| {
        findings.push(Finding {
            code,
            subject,
            message,
            fix,
        });
    };
    if !repo.clone_exists {
        push(
            Code::CloneMissing,
            subject(),
            format!("its clone is gone from {}", repo.clone.display()),
            "put the clone back at that path, or ask the person; mori never re-clones by itself"
                .to_owned(),
        );
        return;
    }
    if repo.backend_changed {
        push(
            Code::BackendChanged,
            subject(),
            "a .jj folder appeared in this git clone, so mori now reads it as jj and its git \
             worktrees look missing"
                .to_owned(),
            format!(
                "if `jj git init` was a mistake, remove {}/.jj; otherwise ask the person",
                repo.clone.display()
            ),
        );
    }
    if !repo.context_folder_exists {
        push(
            Code::ContextFolderMissing,
            subject(),
            "its context folder under context/projects/ is missing".to_owned(),
            format!("mori skills sync (or {FIX})"),
        );
    }
    for bookmark in &repo.conflicted_bookmarks {
        push(
            Code::ConflictedBookmark,
            subject(),
            format!("the bookmark {bookmark:?} has several targets"),
            format!(
                "see them with `jj bookmark list {bookmark}`, then pick one: \
                 `jj bookmark set {bookmark} -r <revision>`"
            ),
        );
    }
    for dir in &repo.stray_dirs {
        push(
            Code::DirWithoutWorkspace,
            Subject::Path(dir.clone()),
            "a directory under trees/ that no workspace uses".to_owned(),
            "look inside; move or delete it by hand if nothing in it is needed".to_owned(),
        );
    }
    // A git clone that turned into a jj one reads its worktrees as missing; that is one problem,
    // already reported, not one per tree.
    if repo.backend_changed {
        return;
    }
    for tree in &repo.trees {
        check_tree(repo, tree, findings);
    }
}

fn check_tree(repo: &RepoFacts, tree: &TreeFacts, findings: &mut Vec<Finding>) {
    let mut push = |code, subject, message: String, fix: String| {
        findings.push(Finding {
            code,
            subject,
            message,
            fix,
        });
    };
    let subject = Subject::Tree {
        repo: repo.repo.clone(),
        name: tree.name.clone(),
    };
    match tree.known {
        Known::VcsOnly => push(
            Code::ForeignWorkspace,
            subject,
            "a workspace mori didn't create; mori leaves it alone".to_owned(),
            "nothing".to_owned(),
        ),
        Known::RecordOnly => push(
            Code::WorkspaceGone,
            subject,
            "mori has a record of it, but the VCS has no such tree".to_owned(),
            format!("{FIX} drops the record"),
        ),
        Known::Both if !tree.dir_exists => push(
            Code::TreeDirGone,
            subject,
            format!("its directory {} was deleted", tree.path.display()),
            format!("{FIX} pins its last commit and forgets it; `mori restore` brings it back"),
        ),
        Known::Both => {}
    }
}

/// The last line of a report: how many problems, and how many `--fix` repairs.
#[must_use]
pub fn summary(findings: &[Finding]) -> String {
    let problems = findings
        .iter()
        .filter(|finding| finding.severity() == Severity::Problem)
        .count();
    let fixable = findings
        .iter()
        .filter(|finding| finding.auto_fixable())
        .count();
    if findings.is_empty() {
        return "no problems".to_owned();
    }
    let noun = if problems == 1 { "problem" } else { "problems" };
    if fixable == 0 {
        format!(
            "{problems} {noun}, {} findings; none can be fixed automatically",
            findings.len()
        )
    } else {
        format!(
            "{problems} {noun}, {} findings; {fixable} can be fixed with `{FIX}`",
            findings.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> RepoFacts {
        RepoFacts {
            repo: "github.com/acme/widget".to_owned(),
            clone: PathBuf::from("/home/acme/mori/repos/github.com/acme/widget"),
            clone_exists: true,
            context_folder_exists: true,
            trees: vec![tree("default"), tree("claude-fix-login")],
            ..RepoFacts::default()
        }
    }

    fn tree(name: &str) -> TreeFacts {
        TreeFacts {
            name: name.to_owned(),
            path: PathBuf::from("/home/acme/mori/trees/widget").join(name),
            known: Known::Both,
            dir_exists: true,
        }
    }

    fn codes(facts: &Facts) -> Vec<&'static str> {
        check(facts)
            .iter()
            .map(|finding| finding.code.as_str())
            .collect()
    }

    fn one_repo(edit: impl FnOnce(&mut RepoFacts)) -> Facts {
        let mut repo = repo();
        edit(&mut repo);
        Facts {
            repos: vec![repo],
            ..Facts::default()
        }
    }

    #[test]
    fn a_healthy_root_has_no_findings() {
        let facts = one_repo(|_| {});

        assert!(check(&facts).is_empty());
        assert_eq!(summary(&check(&facts)), "no problems");
    }

    #[test]
    fn each_tree_drift_has_its_code() {
        let facts = one_repo(|repo| {
            repo.trees[1].dir_exists = false;
            repo.trees.push(TreeFacts {
                known: Known::RecordOnly,
                ..tree("claude-gone")
            });
            repo.trees.push(TreeFacts {
                known: Known::VcsOnly,
                ..tree("someone-elses")
            });
        });

        assert_eq!(
            codes(&facts),
            ["TREE_DIR_GONE", "WORKSPACE_GONE", "FOREIGN_WORKSPACE"]
        );
    }

    #[test]
    fn a_missing_clone_is_the_only_finding_for_its_repo() {
        let facts = one_repo(|repo| {
            repo.clone_exists = false;
            repo.context_folder_exists = false;
            repo.trees[1].dir_exists = false;
        });

        assert_eq!(codes(&facts), ["CLONE_MISSING"]);
    }

    #[test]
    fn a_changed_backend_is_one_problem_not_one_per_tree() {
        let facts = one_repo(|repo| {
            repo.backend_changed = true;
            repo.trees[1].known = Known::RecordOnly;
        });

        assert_eq!(codes(&facts), ["BACKEND_CHANGED"]);
    }

    #[test]
    fn repo_and_root_drift_have_their_codes() {
        let mut facts = one_repo(|repo| {
            repo.context_folder_exists = false;
            repo.conflicted_bookmarks = vec!["main".to_owned()];
            repo.stray_dirs = vec![PathBuf::from("/home/acme/mori/trees/widget/old")];
        });
        facts.tools_missing = vec![VcsKind::Git];
        facts.index_stale = true;
        facts.unrecorded_clones = vec![(
            "github.com/acme/other".to_owned(),
            PathBuf::from("/home/acme/mori/repos/github.com/acme/other"),
        )];

        let mut found = codes(&facts);
        found.sort_unstable();

        assert_eq!(
            found,
            [
                "CLONE_NOT_RECORDED",
                "CONFLICTED_BOOKMARK",
                "CONTEXT_FOLDER_MISSING",
                "CONTEXT_INDEX_STALE",
                "DIR_WITHOUT_WORKSPACE",
                "TOOL_NOT_FOUND"
            ]
        );
    }

    #[test]
    fn findings_come_worst_first() {
        let facts = one_repo(|repo| {
            repo.context_folder_exists = false;
            repo.trees[1].dir_exists = false;
            repo.trees.push(TreeFacts {
                known: Known::VcsOnly,
                ..tree("someone-elses")
            });
        });

        let severities: Vec<Severity> = check(&facts).iter().map(Finding::severity).collect();

        assert_eq!(
            severities,
            [Severity::Problem, Severity::Warn, Severity::Info]
        );
    }

    #[test]
    fn only_safe_single_answer_fixes_are_automatic() {
        assert!(Code::TreeDirGone.auto_fixable());
        assert!(Code::WorkspaceGone.auto_fixable());
        assert!(!Code::ConflictedBookmark.auto_fixable());
        assert!(!Code::ForeignWorkspace.auto_fixable());
        assert!(!Code::DirWithoutWorkspace.auto_fixable());
        assert!(!Code::BackendChanged.auto_fixable());
        assert!(!Code::CloneMissing.auto_fixable());
    }

    #[test]
    fn the_summary_counts_problems_and_fixes() {
        let facts = one_repo(|repo| {
            repo.trees[1].dir_exists = false;
            repo.conflicted_bookmarks = vec!["main".to_owned()];
        });

        assert_eq!(
            summary(&check(&facts)),
            "2 problems, 2 findings; 1 can be fixed with `mori doctor --fix --yes`"
        );
    }
}

//! The flows against a fake VCS and code host, on a throwaway root: every rule that needs no real
//! jj, checked without one.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mori_api::v1alpha1::gc_item::{Class as ItemClass, Outcome};
use mori_core::clone::RepoId;
use mori_core::error::{Code, ErrorDetails};
use mori_core::forest::{TreeState, Workspace, Workspaces};
use mori_core::paths::Env;
use mori_core::vcs::{Forge, Merged, RemoteBookmark, Vcs, VcsKind};
use tempfile::TempDir;

use crate::{App, Backend, Host, clone, gc, init, ls, restore, tree, tree_remove};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// A VCS failure, as the fake reports it.
#[derive(Debug)]
struct FakeError(String);

impl std::error::Error for FakeError {}

impl std::fmt::Display for FakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl ErrorDetails for FakeError {
    fn code(&self) -> Code {
        Code::Internal
    }

    fn reason(&self) -> &'static str {
        "FAKE_VCS_FAILED"
    }

    fn domain(&self) -> &'static str {
        "test.mori"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        Vec::new()
    }
}

/// One tree the fake knows.
#[derive(Clone, Debug)]
struct FakeTree {
    root: PathBuf,
    commit: String,
    changed: bool,
    unpushed: u32,
    bookmarks: Vec<RemoteBookmark>,
}

#[derive(Debug, Default)]
struct State {
    /// By clone and tree name.
    trees: BTreeMap<(PathBuf, String), FakeTree>,
    commits: BTreeSet<String>,
    pins: BTreeMap<String, String>,
}

/// An in-memory VCS that makes real directories, so the flows' own disk checks still apply.
#[derive(Debug, Default)]
struct FakeVcs {
    state: RefCell<State>,
}

impl FakeVcs {
    fn tree<T>(
        &self,
        clone: &Path,
        name: &str,
        f: impl FnOnce(&mut FakeTree) -> T,
    ) -> std::result::Result<T, FakeError> {
        let mut state = self.state.borrow_mut();
        let tree = state
            .trees
            .get_mut(&(clone.to_path_buf(), name.to_owned()))
            .ok_or_else(|| FakeError(format!("no tree {name} in {}", clone.display())))?;
        Ok(f(tree))
    }

    fn add(
        &self,
        clone: &Path,
        name: &str,
        root: &Path,
        commit: String,
    ) -> std::result::Result<(), FakeError> {
        if root.exists() {
            return Err(FakeError(format!("{} exists", root.display())));
        }
        std::fs::create_dir_all(root).map_err(|error| FakeError(error.to_string()))?;
        let mut state = self.state.borrow_mut();
        state.commits.insert(commit.clone());
        state.trees.insert(
            (clone.to_path_buf(), name.to_owned()),
            FakeTree {
                root: root.to_path_buf(),
                commit,
                changed: false,
                unpushed: 0,
                bookmarks: Vec::new(),
            },
        );
        Ok(())
    }
}

impl Workspaces for FakeVcs {
    type Error = FakeError;

    fn list(&self, clone: &Path) -> std::result::Result<Vec<Workspace>, FakeError> {
        Ok(self
            .state
            .borrow()
            .trees
            .iter()
            .filter(|((of, _), _)| of == clone)
            .map(|((_, name), tree)| Workspace {
                name: name.clone(),
                root: tree.root.clone(),
            })
            .collect())
    }

    fn state(&self, clone: &Path, name: &str) -> std::result::Result<TreeState, FakeError> {
        self.state_covering(clone, name, &[])
    }
}

impl Vcs for FakeVcs {
    fn clone_repo(
        &self,
        _url: &str,
        path: &Path,
        _colocate: bool,
    ) -> std::result::Result<(), FakeError> {
        self.add(path, "default", path, "c-base".to_owned())
    }

    fn add_tree(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        _from: &str,
    ) -> std::result::Result<(), FakeError> {
        self.add(clone, name, path, format!("c-{name}"))
    }

    fn add_tree_at(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        commit_id: &str,
    ) -> std::result::Result<(), FakeError> {
        if !self.state.borrow().commits.contains(commit_id) {
            return Err(FakeError(format!("no commit {commit_id}")));
        }
        self.add(clone, name, path, commit_id.to_owned())
    }

    fn snapshot(&self, _tree: &Path) -> std::result::Result<(), FakeError> {
        Ok(())
    }

    fn forget_tree(&self, clone: &Path, name: &str) -> std::result::Result<(), FakeError> {
        self.state
            .borrow_mut()
            .trees
            .remove(&(clone.to_path_buf(), name.to_owned()));
        Ok(())
    }

    fn state_covering(
        &self,
        clone: &Path,
        name: &str,
        landed: &[String],
    ) -> std::result::Result<TreeState, FakeError> {
        self.tree(clone, name, |tree| TreeState {
            change: tree.commit.clone(),
            changed: tree.changed,
            unpushed: if landed.contains(&tree.commit) {
                0
            } else {
                tree.unpushed
            },
        })
    }

    fn working_copy_commit(
        &self,
        clone: &Path,
        name: &str,
    ) -> std::result::Result<String, FakeError> {
        self.tree(clone, name, |tree| tree.commit.clone())
    }

    fn pushed_bookmarks(
        &self,
        clone: &Path,
        name: &str,
    ) -> std::result::Result<Vec<RemoteBookmark>, FakeError> {
        self.tree(clone, name, |tree| tree.bookmarks.clone())
    }

    fn last_change(&self, _clone: &Path, _name: &str) -> std::result::Result<u64, FakeError> {
        Ok(NOW)
    }

    fn pin(
        &self,
        _clone: &Path,
        name: &str,
        commit_id: &str,
    ) -> std::result::Result<(), FakeError> {
        self.state
            .borrow_mut()
            .pins
            .insert(name.to_owned(), commit_id.to_owned());
        Ok(())
    }

    fn commit_exists(
        &self,
        _clone: &Path,
        commit_id: &str,
    ) -> std::result::Result<bool, FakeError> {
        Ok(self.state.borrow().commits.contains(commit_id))
    }

    fn fetch(&self, _clone: &Path) -> std::result::Result<(), FakeError> {
        Ok(())
    }
}

impl Backend for FakeVcs {
    fn clone_as(
        &self,
        _kind: VcsKind,
        url: &str,
        path: &Path,
        colocate: bool,
    ) -> std::result::Result<(), FakeError> {
        self.clone_repo(url, path, colocate)
    }
}

/// A code host that never knows.
struct NoForge;

impl Forge for NoForge {
    fn pr_merged(&self, _repo: &RepoId, _bookmark: &str) -> Merged {
        Merged::Unknown
    }
}

const NOW: u64 = 1_800_000_000;
const REPO: &str = "github.com/acme/widget";

/// A set-up root with `acme/widget` cloned, under a temporary HOME.
struct Fixture {
    home: TempDir,
    app: App<FakeVcs, NoForge>,
}

impl Fixture {
    fn new() -> Result<Self> {
        let home = TempDir::new()?;
        let host = Host {
            env: Env {
                home: Some(home.path().to_path_buf()),
                xdg_config_home: Some(home.path().join(".config")),
                xdg_state_home: Some(home.path().join(".state")),
                xdg_cache_home: Some(home.path().join(".cache")),
                ..Env::default()
            },
            user: Some("tester".to_owned()),
            agent: None,
            now: NOW,
        };
        let app = App::new(host, FakeVcs::default(), NoForge);
        init::run(&app.host, false).map_err(to_std)?;
        clone::run(&app, REPO, false, false).map_err(to_std)?;
        Ok(Self { home, app })
    }

    fn clone_path(&self) -> PathBuf {
        self.home.path().join("mori/repos").join(REPO)
    }

    fn create(&self, task: &str) -> Result<String> {
        let created = tree::create(
            &self.app,
            tree::CreateArgs {
                repo: REPO.to_owned(),
                task: task.to_owned(),
                agent: Some("claude".to_owned()),
                lifetime: None,
                from: None,
                dry_run: false,
            },
        )
        .map_err(to_std)?;
        Ok(created.tree.map(|tree| tree.name).unwrap_or_default())
    }

    fn remove(&self, name: &str) -> std::result::Result<(), Box<dyn ErrorDetails>> {
        tree_remove::run(
            &self.app,
            tree_remove::RemoveArgs {
                repo: REPO.to_owned(),
                name: name.to_owned(),
                pinned: false,
                dry_run: false,
            },
        )
        .map(|_| ())
    }

    fn tree_names(&self) -> Result<Vec<String>> {
        let listed = ls::run(&self.app, Some(REPO)).map_err(to_std)?;
        Ok(listed
            .repos
            .into_iter()
            .flat_map(|repo| repo.trees)
            .filter_map(|row| row.tree.map(|tree| tree.name))
            .collect())
    }
}

fn to_std(error: Box<dyn ErrorDetails>) -> Box<dyn std::error::Error> {
    error
}

#[test]
fn a_created_tree_is_listed_under_its_owner() -> Result<()> {
    let fixture = Fixture::new()?;

    let name = fixture.create("fix-login")?;

    assert_eq!(name, "claude-fix-login");
    assert!(
        fixture
            .home
            .path()
            .join("mori/trees/widget/claude-fix-login")
            .is_dir()
    );
    assert_eq!(fixture.tree_names()?, ["claude-fix-login", "default"]);
    Ok(())
}

#[test]
fn the_owner_defaults_to_mori_agent_then_user() -> Result<()> {
    let mut host = Host {
        user: Some("tester".to_owned()),
        ..Host::default()
    };
    assert_eq!(tree::owner(&host, None).map_err(to_std)?, "tester");

    host.agent = Some("codex".to_owned());
    assert_eq!(tree::owner(&host, None).map_err(to_std)?, "codex");
    assert_eq!(
        tree::owner(&host, Some("claude".to_owned())).map_err(to_std)?,
        "claude"
    );
    Ok(())
}

#[test]
fn a_tree_with_unsaved_work_is_kept() -> Result<()> {
    let fixture = Fixture::new()?;
    let name = fixture.create("fix-login")?;
    fixture
        .app
        .vcs
        .tree(&fixture.clone_path(), &name, |tree| tree.changed = true)?;

    let error = fixture
        .remove(&name)
        .err()
        .ok_or("removed a tree with edits")?;

    assert_eq!(error.reason(), "TREE_HAS_UNSAVED_WORK");
    assert!(fixture.tree_names()?.contains(&name));
    Ok(())
}

#[test]
fn a_saved_tree_is_removed() -> Result<()> {
    let fixture = Fixture::new()?;
    let name = fixture.create("fix-login")?;

    fixture.remove(&name).map_err(to_std)?;

    assert_eq!(fixture.tree_names()?, ["default"]);
    assert!(
        !fixture
            .home
            .path()
            .join("mori/trees/widget")
            .join(&name)
            .exists()
    );
    Ok(())
}

/// Creates a tree whose pushed bookmark the remote then deleted: its work landed.
fn landed_tree(fixture: &Fixture) -> Result<String> {
    let name = fixture.create("fix-login")?;
    let clone = fixture.clone_path();
    fixture.app.vcs.tree(&clone, &name, |tree| {
        tree.unpushed = 1;
        tree.bookmarks = vec![RemoteBookmark {
            name: "claude/fix-login".to_owned(),
            remote: "origin".to_owned(),
            commit_id: tree.commit.clone(),
        }];
    })?;
    // Looking records the bookmark; then the remote deletes it after a squash merge.
    fixture.tree_names()?;
    fixture
        .app
        .vcs
        .tree(&clone, &name, |tree| tree.bookmarks.clear())?;
    Ok(name)
}

fn apply(fixture: &Fixture) -> Result<mori_api::v1alpha1::GcResponse> {
    gc::run(
        &fixture.app,
        &gc::GcArgs {
            repo: None,
            offline: true,
            apply: Some(gc::Apply {
                yes: true,
                names: Vec::new(),
                max: None,
                dry_run: false,
            }),
        },
    )
    .map_err(to_std)
}

#[test]
fn gc_removes_a_landed_tree_and_restore_brings_it_back() -> Result<()> {
    let fixture = Fixture::new()?;
    let name = landed_tree(&fixture)?;

    let report = apply(&fixture)?;

    let item = report
        .items
        .iter()
        .find(|item| item.name == name)
        .ok_or("tree not in the report")?;
    assert_eq!(item.class(), ItemClass::Remove);
    assert_eq!(item.outcome(), Outcome::Removed);
    assert_eq!(fixture.tree_names()?, ["default"]);
    let pins = fixture.app.vcs.state.borrow().pins.clone();
    assert_eq!(
        pins.get(&format!("refs/mori/removed/{}", item.entry_id)),
        Some(&format!("c-{name}"))
    );

    let restored = restore::run(&fixture.app, &item.entry_id).map_err(to_std)?;

    assert_eq!(restored.commit_id, format!("c-{name}"));
    assert!(fixture.tree_names()?.contains(&name));
    Ok(())
}

#[test]
fn restore_refuses_when_the_commit_is_gone() -> Result<()> {
    let fixture = Fixture::new()?;
    let name = landed_tree(&fixture)?;
    let entry_id = apply(&fixture)?
        .items
        .into_iter()
        .find(|item| item.name == name)
        .map(|item| item.entry_id)
        .ok_or("tree not in the report")?;
    fixture.app.vcs.state.borrow_mut().commits.clear();

    let error = restore::run(&fixture.app, &entry_id)
        .err()
        .ok_or("restored without its commit")?;

    assert_eq!(error.reason(), "RESTORE_COMMIT_GONE");
    assert!(!fixture.tree_names()?.contains(&name));
    Ok(())
}

#[test]
fn gc_without_yes_removes_nothing() -> Result<()> {
    let fixture = Fixture::new()?;
    let name = landed_tree(&fixture)?;

    let error = gc::run(
        &fixture.app,
        &gc::GcArgs {
            repo: None,
            offline: true,
            apply: Some(gc::Apply {
                yes: false,
                names: Vec::new(),
                max: None,
                dry_run: false,
            }),
        },
    )
    .err()
    .ok_or("removed without confirmation")?;

    assert_eq!(error.reason(), "CONFIRMATION_NEEDED");
    assert!(fixture.tree_names()?.contains(&name));
    Ok(())
}

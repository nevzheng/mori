//! One VCS over two backends: every call goes to jj or git by what the clone is.
//!
//! The backend isn't recorded anywhere: a clone or tree with a `.jj` directory is jj, anything
//! else is git. Only a new clone needs to be told which to make.

use std::fmt;
use std::path::Path;

use mori_core::error::{Code, ErrorDetails};
use mori_core::forest::{TreeState, Workspace, Workspaces};
use mori_core::vcs::{RemoteBookmark, Vcs, VcsKind};

use crate::Backend;

/// A single backend: any VCS whose errors carry codes and reasons.
pub trait Adapter: Vcs<Error: ErrorDetails + 'static> {}

impl<T: Vcs<Error: ErrorDetails + 'static>> Adapter for T {}

/// jj and git, picked per clone.
#[derive(Clone, Debug)]
pub struct Routed<J, G> {
    /// The jj backend.
    pub jj: J,
    /// The git backend.
    pub git: G,
}

/// Which backend owns `dir`, a clone or one of its trees.
#[must_use]
pub fn kind_of(dir: &Path) -> VcsKind {
    if dir.join(".jj").is_dir() {
        VcsKind::Jj
    } else {
        VcsKind::Git
    }
}

/// Runs `$call` on the backend that owns `$dir`, wrapping its error.
macro_rules! route {
    ($self:ident, $dir:expr, |$vcs:ident| $call:expr) => {
        match kind_of($dir) {
            VcsKind::Jj => {
                let $vcs = &$self.jj;
                $call.map_err(RoutedError::Jj)
            }
            VcsKind::Git => {
                let $vcs = &$self.git;
                $call.map_err(RoutedError::Git)
            }
        }
    };
}

impl<J: Adapter, G: Adapter> Workspaces for Routed<J, G> {
    type Error = RoutedError<J::Error, G::Error>;

    fn list(&self, clone: &Path) -> Result<Vec<Workspace>, Self::Error> {
        route!(self, clone, |vcs| vcs.list(clone))
    }

    fn state(&self, clone: &Path, name: &str) -> Result<TreeState, Self::Error> {
        route!(self, clone, |vcs| vcs.state(clone, name))
    }
}

impl<J: Adapter, G: Adapter> Vcs for Routed<J, G> {
    /// Without a clone to look at, this makes a jj clone; [`Backend::clone_as`] picks.
    fn clone_repo(&self, url: &str, path: &Path, colocate: bool) -> Result<(), Self::Error> {
        self.clone_as(VcsKind::Jj, url, path, colocate)
    }

    fn add_tree(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        from: &str,
    ) -> Result<(), Self::Error> {
        route!(self, clone, |vcs| vcs.add_tree(clone, name, path, from))
    }

    fn add_tree_at(
        &self,
        clone: &Path,
        name: &str,
        path: &Path,
        commit_id: &str,
    ) -> Result<(), Self::Error> {
        route!(self, clone, |vcs| vcs
            .add_tree_at(clone, name, path, commit_id))
    }

    fn snapshot(&self, tree: &Path) -> Result<(), Self::Error> {
        route!(self, tree, |vcs| vcs.snapshot(tree))
    }

    fn forget_tree(&self, clone: &Path, name: &str) -> Result<(), Self::Error> {
        route!(self, clone, |vcs| vcs.forget_tree(clone, name))
    }

    fn state_covering(
        &self,
        clone: &Path,
        name: &str,
        landed: &[String],
    ) -> Result<TreeState, Self::Error> {
        route!(self, clone, |vcs| vcs.state_covering(clone, name, landed))
    }

    fn working_copy_commit(&self, clone: &Path, name: &str) -> Result<String, Self::Error> {
        route!(self, clone, |vcs| vcs.working_copy_commit(clone, name))
    }

    fn pushed_bookmarks(
        &self,
        clone: &Path,
        name: &str,
    ) -> Result<Vec<RemoteBookmark>, Self::Error> {
        route!(self, clone, |vcs| vcs.pushed_bookmarks(clone, name))
    }

    fn last_change(&self, clone: &Path, name: &str) -> Result<u64, Self::Error> {
        route!(self, clone, |vcs| vcs.last_change(clone, name))
    }

    fn pin(&self, clone: &Path, name: &str, commit_id: &str) -> Result<(), Self::Error> {
        route!(self, clone, |vcs| vcs.pin(clone, name, commit_id))
    }

    fn commit_exists(&self, clone: &Path, commit_id: &str) -> Result<bool, Self::Error> {
        route!(self, clone, |vcs| vcs.commit_exists(clone, commit_id))
    }

    fn fetch(&self, clone: &Path) -> Result<(), Self::Error> {
        route!(self, clone, |vcs| vcs.fetch(clone))
    }
}

impl<J: Adapter, G: Adapter> Backend for Routed<J, G> {
    fn clone_as(
        &self,
        kind: VcsKind,
        url: &str,
        path: &Path,
        colocate: bool,
    ) -> Result<(), Self::Error> {
        match kind {
            VcsKind::Jj => self
                .jj
                .clone_repo(url, path, colocate)
                .map_err(RoutedError::Jj),
            VcsKind::Git => self
                .git
                .clone_repo(url, path, colocate)
                .map_err(RoutedError::Git),
        }
    }
}

/// An error from whichever backend handled the call; it reports as that backend's own error.
#[derive(Debug)]
pub enum RoutedError<J, G> {
    /// From jj.
    Jj(J),
    /// From git.
    Git(G),
}

impl<J: fmt::Display, G: fmt::Display> fmt::Display for RoutedError<J, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jj(error) => error.fmt(f),
            Self::Git(error) => error.fmt(f),
        }
    }
}

impl<J: ErrorDetails, G: ErrorDetails> std::error::Error for RoutedError<J, G> {}

impl<J: ErrorDetails, G: ErrorDetails> ErrorDetails for RoutedError<J, G> {
    fn code(&self) -> Code {
        match self {
            Self::Jj(error) => error.code(),
            Self::Git(error) => error.code(),
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            Self::Jj(error) => error.reason(),
            Self::Git(error) => error.reason(),
        }
    }

    fn domain(&self) -> &'static str {
        match self {
            Self::Jj(error) => error.domain(),
            Self::Git(error) => error.domain(),
        }
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::Jj(error) => error.metadata(),
            Self::Git(error) => error.metadata(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    /// A backend that only records the calls it gets; a jj one marks its directories with `.jj`.
    #[derive(Debug, Default)]
    struct Recorder {
        marks_jj: bool,
        calls: RefCell<Vec<String>>,
    }

    #[derive(Debug)]
    struct Never;

    impl fmt::Display for Never {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }

    impl std::error::Error for Never {}

    impl ErrorDetails for Never {
        fn code(&self) -> Code {
            Code::Internal
        }
        fn reason(&self) -> &'static str {
            "NEVER"
        }
        fn domain(&self) -> &'static str {
            "test.mori"
        }
        fn metadata(&self) -> Vec<(&'static str, String)> {
            Vec::new()
        }
    }

    impl Recorder {
        fn jj() -> Self {
            Self {
                marks_jj: true,
                ..Self::default()
            }
        }

        fn call(&self, what: &str) {
            self.calls.borrow_mut().push(what.to_owned());
        }

        fn make(&self, dir: &Path) -> Result<(), Never> {
            let dir = if self.marks_jj {
                dir.join(".jj")
            } else {
                dir.to_path_buf()
            };
            std::fs::create_dir_all(dir).map_err(|_| Never)
        }
    }

    impl Workspaces for Recorder {
        type Error = Never;
        fn list(&self, _: &Path) -> Result<Vec<Workspace>, Never> {
            self.call("list");
            Ok(Vec::new())
        }
        fn state(&self, _: &Path, _: &str) -> Result<TreeState, Never> {
            self.call("state");
            Err(Never)
        }
    }

    impl Vcs for Recorder {
        fn clone_repo(&self, _: &str, path: &Path, _: bool) -> Result<(), Never> {
            self.call("clone");
            self.make(path)
        }
        fn add_tree(&self, _: &Path, _: &str, path: &Path, _: &str) -> Result<(), Never> {
            self.call("add");
            self.make(path)
        }
        fn add_tree_at(&self, _: &Path, _: &str, path: &Path, _: &str) -> Result<(), Never> {
            self.call("add_at");
            self.make(path)
        }
        fn snapshot(&self, _: &Path) -> Result<(), Never> {
            self.call("snapshot");
            Ok(())
        }
        fn forget_tree(&self, _: &Path, _: &str) -> Result<(), Never> {
            self.call("forget");
            Ok(())
        }
        fn state_covering(&self, _: &Path, _: &str, _: &[String]) -> Result<TreeState, Never> {
            self.call("state");
            Err(Never)
        }
        fn working_copy_commit(&self, _: &Path, _: &str) -> Result<String, Never> {
            self.call("commit");
            Ok(String::new())
        }
        fn pushed_bookmarks(&self, _: &Path, _: &str) -> Result<Vec<RemoteBookmark>, Never> {
            self.call("pushed");
            Ok(Vec::new())
        }
        fn last_change(&self, _: &Path, _: &str) -> Result<u64, Never> {
            self.call("last_change");
            Ok(0)
        }
        fn pin(&self, _: &Path, _: &str, _: &str) -> Result<(), Never> {
            self.call("pin");
            Ok(())
        }
        fn commit_exists(&self, _: &Path, _: &str) -> Result<bool, Never> {
            self.call("exists");
            Ok(true)
        }
        fn fetch(&self, _: &Path) -> Result<(), Never> {
            self.call("fetch");
            Ok(())
        }
    }

    fn routed() -> Routed<Recorder, Recorder> {
        Routed {
            jj: Recorder::jj(),
            git: Recorder::default(),
        }
    }

    fn calls(recorder: &Recorder) -> Vec<String> {
        recorder.calls.borrow().clone()
    }

    #[test]
    fn a_new_clone_goes_to_the_backend_asked_for() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new()?;
        let vcs = routed();

        vcs.clone_as(VcsKind::Git, "url", &dir.path().join("g"), true)?;
        vcs.clone_as(VcsKind::Jj, "url", &dir.path().join("j"), true)?;

        assert_eq!(calls(&vcs.git), ["clone"]);
        assert_eq!(calls(&vcs.jj), ["clone"]);
        Ok(())
    }

    #[test]
    fn every_later_call_goes_to_the_clone_s_own_backend() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = TempDir::new()?;
        let vcs = routed();
        let git_clone: PathBuf = dir.path().join("g");
        let jj_clone: PathBuf = dir.path().join("j");
        vcs.clone_as(VcsKind::Git, "url", &git_clone, true)?;
        vcs.clone_as(VcsKind::Jj, "url", &jj_clone, true)?;

        for clone in [&git_clone, &jj_clone] {
            vcs.list(clone)?;
            vcs.add_tree(clone, "t", &clone.join("t"), "trunk()")?;
            vcs.pin(clone, "refs/mori/removed/j", "c")?;
            vcs.fetch(clone)?;
        }
        vcs.snapshot(&jj_clone.join("t"))?;

        let expected = ["clone", "list", "add", "pin", "fetch"];
        assert_eq!(calls(&vcs.git), expected);
        assert_eq!(calls(&vcs.jj), [&expected[..], &["snapshot"]].concat());
        Ok(())
    }

    #[test]
    fn an_error_reports_as_its_backend_s_own() {
        let vcs = routed();

        let error = vcs.state(Path::new("/nonexistent"), "t").unwrap_err();

        assert!(matches!(error, RoutedError::Git(Never)));
        assert_eq!(error.reason(), "NEVER");
        assert_eq!(error.domain(), "test.mori");
    }
}

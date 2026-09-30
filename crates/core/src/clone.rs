//! `mori clone`: a VCS clone plus a record of it.
//!
//! The adapter observes whether the clone's path exists and what mori has already recorded,
//! [`plan`] turns that into a [`ClonePlan`] or a refusal, and the adapter runs the clone, then
//! records the repo and its base tree. Recording is best effort: if it fails after the clone
//! succeeded, the clone stays, because mori never deletes work.

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

use crate::error::RepoError;
use crate::paths::Paths;

/// The base tree's name: jj's name for a clone's own workspace, so the record matches it.
pub const BASE_TREE_NAME: &str = "default";

/// Who the base tree belongs to: the person, never an agent.
pub const BASE_TREE_OWNER: &str = "you";

/// A repo's identity: its remote without the scheme, user, port or `.git`, all lowercase, such as
/// `github.com/acme/widget`. Two URLs for the same repo give the same identity, whatever their case.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoId {
    /// The host: `github.com`.
    pub host: String,
    /// The owner: `acme`.
    pub owner: String,
    /// The repo: `widget`.
    pub name: String,
}

impl fmt::Display for RepoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.host, self.owner, self.name)
    }
}

/// A URL mori can clone, and the repo it names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloneUrl {
    /// Who the repo is.
    pub repo: RepoId,
    /// What to fetch from: the URL as given, or `https://<host>/<owner>/<repo>.git` for the
    /// short form.
    pub fetch: String,
}

impl CloneUrl {
    /// Parses one of the forms people paste:
    ///
    /// - `https://github.com/acme/widget(.git)`
    /// - `ssh://git@github.com(:22)/acme/widget(.git)`
    /// - `git@github.com:acme/widget(.git)`
    /// - `github.com/acme/widget`
    ///
    /// Only `<host>/<owner>/<repo>` paths are accepted for now; nested groups are refused.
    ///
    /// # Errors
    ///
    /// [`RepoError::UrlInvalid`] saying what's wrong.
    pub fn parse(url: &str) -> Result<Self, RepoError> {
        let invalid = |why: &str| RepoError::UrlInvalid {
            url: url.to_owned(),
            why: why.to_owned(),
        };
        let (host, path, fetch) = if let Some(rest) = url.strip_prefix("https://") {
            let (host, path) = rest.split_once('/').ok_or_else(|| invalid("no path"))?;
            (host, path, url.to_owned())
        } else if let Some(rest) = url.strip_prefix("ssh://") {
            let (authority, path) = rest.split_once('/').ok_or_else(|| invalid("no path"))?;
            let host = authority
                .rsplit_once('@')
                .map_or(authority, |(_, host)| host);
            let host = host.split_once(':').map_or(host, |(host, _port)| host);
            (host, path, url.to_owned())
        } else if url.contains("://") {
            return Err(invalid(
                "use https://, ssh://, user@host:owner/repo or host/owner/repo",
            ));
        } else if let Some((authority, path)) = url.split_once(':') {
            // scp-like: user@host:owner/repo
            let host = authority
                .rsplit_once('@')
                .map_or(authority, |(_, host)| host);
            (host, path, url.to_owned())
        } else {
            let (host, path) = url.split_once('/').ok_or_else(|| invalid("no path"))?;
            let path = path.trim_end_matches('/');
            let path = path.strip_suffix(".git").unwrap_or(path);
            (host, path, format!("https://{host}/{path}.git"))
        };

        let path = path.trim_end_matches('/');
        let path = path.strip_suffix(".git").unwrap_or(path);
        let [owner, name] = path.split('/').collect::<Vec<_>>()[..] else {
            return Err(invalid("the path must be exactly <owner>/<repo>"));
        };
        let (host, owner, name) = (
            host.to_ascii_lowercase(),
            owner.to_ascii_lowercase(),
            name.to_ascii_lowercase(),
        );
        for (what, part) in [("host", &host), ("owner", &owner), ("repo", &name)] {
            check_component(what, part).map_err(|why| invalid(&why))?;
        }
        Ok(Self {
            repo: RepoId { host, owner, name },
            fetch,
        })
    }
}

/// A host, owner or repo becomes a directory name, so it must be one plain path component.
fn check_component(what: &str, part: &str) -> Result<(), String> {
    if part.is_empty() {
        return Err(format!("the {what} is empty"));
    }
    if part.starts_with('.') {
        return Err(format!("the {what} {part:?} starts with a dot"));
    }
    if let Some(bad) = part
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')))
    {
        return Err(format!("the {what} {part:?} has {bad:?}"));
    }
    Ok(())
}

/// What exists before `clone` runs, as seen by an adapter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observed {
    /// Whether anything exists at the clone's path.
    pub path_exists: bool,
    /// The repos mori has recorded.
    pub recorded: BTreeSet<RepoId>,
    /// The directory names under `trees/` that recorded repos already use.
    pub tree_dirs: BTreeSet<String>,
}

/// What `clone` will do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClonePlan {
    /// The repo and where to fetch it from.
    pub url: CloneUrl,
    /// Where the clone goes: `<root>/repos/<host>/<owner>/<repo>`.
    pub path: PathBuf,
    /// The repo's directory under `trees/`: its name, or `<owner>-<name>` if another repo
    /// already uses the name.
    pub tree_dir: String,
    /// Whether the clone is a git repo too (colocated), or jj only.
    pub colocate: bool,
}

/// The path a repo's clone gets.
#[must_use]
pub fn clone_path(paths: &Paths, repo: &RepoId) -> PathBuf {
    paths
        .repos()
        .join(&repo.host)
        .join(&repo.owner)
        .join(&repo.name)
}

/// Plans `clone` from what the adapter observed.
///
/// # Errors
///
/// [`RepoError::RepoExists`] if mori already recorded the repo, [`RepoError::PathExists`] if
/// something is at its path, and [`RepoError::TreeDirTaken`] if neither directory name under
/// `trees/` is free. Nothing should be created on any of them.
pub fn plan(
    paths: &Paths,
    url: CloneUrl,
    colocate: bool,
    observed: &Observed,
) -> Result<ClonePlan, RepoError> {
    let path = clone_path(paths, &url.repo);
    if observed.recorded.contains(&url.repo) {
        return Err(RepoError::RepoExists {
            repo: url.repo.to_string(),
        });
    }
    if observed.path_exists {
        return Err(RepoError::PathExists {
            repo: url.repo.to_string(),
            path,
        });
    }
    let tree_dir = [
        url.repo.name.clone(),
        format!("{}-{}", url.repo.owner, url.repo.name),
    ]
    .into_iter()
    .find(|dir| !observed.tree_dirs.contains(dir))
    .ok_or_else(|| RepoError::TreeDirTaken {
        repo: url.repo.to_string(),
    })?;
    Ok(ClonePlan {
        url,
        path,
        tree_dir,
        colocate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Code, ErrorDetails};
    use crate::paths::Env;

    fn widget() -> RepoId {
        RepoId {
            host: "github.com".to_owned(),
            owner: "acme".to_owned(),
            name: "widget".to_owned(),
        }
    }

    fn paths() -> Paths {
        Paths::resolve(&Env {
            home: Some(PathBuf::from("/home/acme")),
            ..Env::default()
        })
        .unwrap()
    }

    #[test]
    fn every_form_names_the_same_repo() {
        for url in [
            "https://github.com/acme/widget",
            "https://github.com/acme/widget.git",
            "https://github.com/acme/widget/",
            "https://GitHub.com/Acme/Widget",
            "git@github.com:ACME/widget.git",
            "ssh://git@github.com/acme/widget.git",
            "ssh://git@github.com:22/acme/widget",
            "git@github.com:acme/widget.git",
            "github.com/acme/widget",
            "github.com/acme/widget.git",
        ] {
            assert_eq!(CloneUrl::parse(url).unwrap().repo, widget(), "for {url:?}");
        }
    }

    #[test]
    fn the_given_url_is_fetched_as_written() {
        let url = CloneUrl::parse("git@github.com:Acme/Widget.git").unwrap();

        assert_eq!(url.fetch, "git@github.com:Acme/Widget.git");
    }

    #[test]
    fn the_short_form_fetches_over_https() {
        let url = CloneUrl::parse("github.com/acme/widget").unwrap();

        assert_eq!(url.fetch, "https://github.com/acme/widget.git");
    }

    #[test]
    fn bad_urls_are_refused() {
        for url in [
            "",
            "widget",
            "http://github.com/acme/widget",
            "file:///srv/widget",
            "https://github.com/acme",
            "https://gitlab.com/group/sub/widget",
            "github.com/acme/../widget",
            "github.com/.hidden/widget",
            "github.com/acme/wid get",
            "github.com//widget",
        ] {
            let error = CloneUrl::parse(url).unwrap_err();

            assert_eq!(error.code(), Code::InvalidArgument, "for {url:?}");
            assert_eq!(error.reason(), "CLONE_URL_INVALID");
        }
    }

    #[test]
    fn the_clone_goes_under_repos() {
        let url = CloneUrl::parse("github.com/acme/widget").unwrap();

        let plan = plan(&paths(), url, true, &Observed::default()).unwrap();

        assert_eq!(
            plan.path,
            PathBuf::from("/home/acme/mori/repos/github.com/acme/widget")
        );
        assert_eq!(plan.tree_dir, "widget");
        assert!(plan.colocate);
    }

    #[test]
    fn a_second_repo_of_the_same_name_gets_the_owner_in_its_tree_dir() {
        let url = CloneUrl::parse("github.com/other/widget").unwrap();
        let observed = Observed {
            tree_dirs: BTreeSet::from(["widget".to_owned()]),
            ..Observed::default()
        };

        let plan = plan(&paths(), url, true, &observed).unwrap();

        assert_eq!(plan.tree_dir, "other-widget");
    }

    #[test]
    fn a_recorded_repo_is_refused() {
        let url = CloneUrl::parse("github.com/acme/widget").unwrap();
        let observed = Observed {
            recorded: BTreeSet::from([widget()]),
            path_exists: true,
            ..Observed::default()
        };

        let error = plan(&paths(), url, true, &observed).unwrap_err();

        assert_eq!(error.code(), Code::AlreadyExists);
        assert_eq!(error.reason(), "REPO_EXISTS");
    }

    #[test]
    fn an_existing_path_is_refused() {
        let url = CloneUrl::parse("github.com/acme/widget").unwrap();
        let observed = Observed {
            path_exists: true,
            ..Observed::default()
        };

        let error = plan(&paths(), url, true, &observed).unwrap_err();

        assert_eq!(error.code(), Code::AlreadyExists);
        assert_eq!(error.reason(), "PATH_EXISTS");
    }

    #[test]
    fn no_free_tree_dir_is_refused() {
        let url = CloneUrl::parse("github.com/acme/widget").unwrap();
        let observed = Observed {
            tree_dirs: BTreeSet::from(["widget".to_owned(), "acme-widget".to_owned()]),
            ..Observed::default()
        };

        let error = plan(&paths(), url, true, &observed).unwrap_err();

        assert_eq!(error.reason(), "TREE_DIR_TAKEN");
    }
}

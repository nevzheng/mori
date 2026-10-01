//! Where a path is in the forest, from the root layout alone: no VCS, no database.
//!
//! `mori where` classifies a path with [`locate`], then the adapter fills in the records.

use std::path::{Component, Path};

use crate::clone::RepoId;
use crate::paths::Paths;

/// What part of the forest a path is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// The root itself, or a part of it that belongs to no repo.
    Root,
    /// Inside a repo's clone (its base tree).
    Clone(RepoId),
    /// Inside a task tree: the repo's directory under `trees/` and the tree's name.
    Tree {
        /// The repo's directory under `trees/`, e.g. `widget`.
        dir: String,
        /// The tree's name.
        name: String,
    },
    /// Inside the context folder; `Some(dir)` within `context/projects/<dir>/`.
    Context(Option<String>),
}

/// Where `path` is in the forest at `paths.root`, or `None` if it is outside. Both paths should
/// be absolute and already have symlinks resolved, so a link can't lead out of the root.
#[must_use]
pub fn locate(paths: &Paths, path: &Path) -> Option<Place> {
    let rest = path.strip_prefix(&paths.root).ok()?;
    let parts: Vec<String> = rest
        .components()
        .filter_map(|part| match part {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    Some(match parts.as_slice() {
        ["repos", host, owner, name, ..] => Place::Clone(RepoId {
            host: (*host).to_owned(),
            owner: (*owner).to_owned(),
            name: (*name).to_owned(),
        }),
        ["trees", dir, name, ..] => Place::Tree {
            dir: (*dir).to_owned(),
            name: (*name).to_owned(),
        },
        ["context", "projects", dir, ..] => Place::Context(Some((*dir).to_owned())),
        ["context", ..] => Place::Context(None),
        _ => Place::Root,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::paths::Env;

    fn paths() -> Paths {
        Paths::resolve(&Env {
            home: Some(PathBuf::from("/home/acme")),
            ..Env::default()
        })
        .unwrap()
    }

    fn at(path: &str) -> Option<Place> {
        locate(&paths(), Path::new(path))
    }

    #[test]
    fn every_part_of_the_layout_has_its_place() {
        assert_eq!(
            at("/home/acme/mori/repos/github.com/acme/widget/src/main.rs"),
            Some(Place::Clone(RepoId {
                host: "github.com".to_owned(),
                owner: "acme".to_owned(),
                name: "widget".to_owned(),
            }))
        );
        assert_eq!(
            at("/home/acme/mori/trees/widget/claude-auth/src"),
            Some(Place::Tree {
                dir: "widget".to_owned(),
                name: "claude-auth".to_owned(),
            })
        );
        assert_eq!(
            at("/home/acme/mori/context/projects/widget/notes.md"),
            Some(Place::Context(Some("widget".to_owned())))
        );
        assert_eq!(
            at("/home/acme/mori/context/skills"),
            Some(Place::Context(None))
        );
        assert_eq!(at("/home/acme/mori"), Some(Place::Root));
        assert_eq!(at("/home/acme/mori/repos/github.com"), Some(Place::Root));
        assert_eq!(at("/home/acme/mori/trees/widget"), Some(Place::Root));
    }

    #[test]
    fn outside_the_root_is_nowhere() {
        assert_eq!(at("/tmp/widget"), None);
        assert_eq!(at("/home/acme/mori-other/repos/x/y/z"), None);
    }
}

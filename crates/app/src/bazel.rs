//! Bazel output left by trees mori deleted. Bazel keeps each workspace's output base outside the
//! tree, so removing a tree leaves it behind; gc finds these leftovers and removes them. No hooks:
//! a leftover is recognized afterwards, whoever deleted the tree.

use std::path::{Path, PathBuf};

use mori_api::v1alpha1::GcItem;
use mori_api::v1alpha1::gc_item::{Class, Kind, Outcome};
use mori_core::disk::{OutputBase, default_bazel_output_user_root, is_leftover};
use mori_core::error::ErrorDetails;
use mori_core::paths::Paths;
use mori_store::records::RepoRecord;

use crate::Host;
use crate::state::{self, boxed};

/// The reason code a leftover is reported with.
const ORPHANED: &str = "ORPHANED";

/// Every leftover of a tree in `repos`, as a removable gc item with its size.
pub(crate) fn leftovers(host: &Host, paths: &Paths, repos: &[RepoRecord]) -> Vec<GcItem> {
    let Some(user_root) = user_root(host, paths) else {
        return Vec::new();
    };
    let trees_dirs = trees_dirs(paths);
    mori_store::disk::output_bases(&user_root)
        .into_iter()
        .filter(|base| is_leftover(base, &trees_dirs))
        .filter_map(|base| {
            let repo = repo_of(&base.workspace, &trees_dirs, repos)?;
            Some(item(&base, repo))
        })
        .collect()
}

/// Removes the leftover at `item.path` after checking again that it still is one.
pub(crate) fn remove(
    host: &Host,
    paths: &Paths,
    repos: &[RepoRecord],
    item: &GcItem,
    dry_run: bool,
) -> Result<Outcome, Box<dyn ErrorDetails>> {
    let still = leftovers(host, paths, repos)
        .iter()
        .any(|leftover| leftover.path == item.path);
    if !still {
        return Ok(Outcome::SkippedChanged);
    }
    if dry_run {
        return Ok(Outcome::WouldRemove);
    }
    mori_store::disk::remove_output_base(Path::new(&item.path)).map_err(boxed)?;
    Ok(Outcome::Removed)
}

/// `[disk] bazel_output_user_root`, else the platform default; none without a home and user.
fn user_root(host: &Host, paths: &Paths) -> Option<PathBuf> {
    if let Some(root) = state::disk_policy(paths).ok()?.bazel_output_user_root {
        return Some(root);
    }
    Some(default_bazel_output_user_root(
        host.env.home.as_deref()?,
        host.user.as_deref()?,
        cfg!(target_os = "macos"),
    ))
}

/// mori's `trees/` directory, as configured and with symlinks resolved: Bazel may write either.
fn trees_dirs(paths: &Paths) -> Vec<PathBuf> {
    let trees = paths.trees();
    let mut dirs = vec![trees.clone()];
    if let Ok(real) = std::fs::canonicalize(&trees)
        && real != trees
    {
        dirs.push(real);
    }
    dirs
}

/// The repo in `repos` whose trees directory holds `workspace`, if any.
fn repo_of<'a>(
    workspace: &Path,
    trees_dirs: &[PathBuf],
    repos: &'a [RepoRecord],
) -> Option<&'a RepoRecord> {
    let rest = trees_dirs
        .iter()
        .find_map(|dir| workspace.strip_prefix(dir).ok())?;
    let dir_name = rest.components().next()?.as_os_str();
    repos.iter().find(|repo| dir_name == repo.dir_name.as_str())
}

fn item(base: &OutputBase, repo: &RepoRecord) -> GcItem {
    let name = base
        .workspace
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    GcItem {
        repo: repo.remote.clone(),
        name,
        path: base.path.display().to_string(),
        class: Class::Remove.into(),
        reason: ORPHANED.to_owned(),
        facts: format!(
            "Bazel output of {}, which is gone; only cache",
            base.workspace.display()
        ),
        kind: Kind::BazelLeftover.into(),
        size_bytes: mori_store::disk::measure(&base.path).bytes,
        ..GcItem::default()
    }
}

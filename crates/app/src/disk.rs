//! Disk usage for the flows: free space under the root, and tree sizes, measured or reused.

use std::collections::BTreeMap;

use mori_core::clone::BASE_TREE_NAME;
use mori_core::disk::{
    RepoUsage, Space, limit_warnings, low_space_warning, rfc3339, size_is_recent,
};
use mori_core::paths::Paths;
use mori_store::database::Database;
use mori_store::disk;
use mori_store::records::TreeSize;

use crate::state;

/// Free and total space on the disk under the root; none if it can't be read.
pub(crate) fn space(paths: &Paths) -> Option<Space> {
    disk::free_space(&paths.root).ok()
}

/// The low-space warning, if free space is below `[disk] warn_below`. Never fails: a config or
/// disk that can't be read gives no warning.
pub(crate) fn low_space(paths: &Paths, space: Option<Space>) -> Option<String> {
    let floor = state::disk_policy(paths).ok()?.warn_below;
    low_space_warning(space?, floor)
}

/// The low-space warning and a warning for each `[disk]` limit mori is over. Tree counts come
/// from mori's records; sizes, per repo, only if `bytes` has them. Never fails: what can't be
/// read gives no warning.
pub(crate) fn warnings(
    paths: &Paths,
    db: &Database,
    space: Option<Space>,
    bytes: Option<&BTreeMap<String, u64>>,
) -> Vec<String> {
    let mut warnings: Vec<String> = low_space(paths, space).into_iter().collect();
    let Ok(policy) = state::disk_policy(paths) else {
        return warnings;
    };
    let usage: Vec<RepoUsage> = db
        .repos()
        .unwrap_or_default()
        .into_iter()
        .map(|repo| RepoUsage {
            task_trees: db.trees(&repo.id).map_or(0, |trees| {
                u32::try_from(
                    trees
                        .iter()
                        .filter(|tree| tree.name != BASE_TREE_NAME)
                        .count(),
                )
                .unwrap_or(u32::MAX)
            }),
            bytes: bytes.and_then(|bytes| bytes.get(&repo.remote).copied()),
            repo: repo.remote,
        })
        .collect();
    warnings.extend(limit_warnings(&policy, &usage));
    warnings
}

/// A tree's size: reused if it was measured recently (unless `skip_cache`), otherwise measured
/// and saved. A tree with no record (`tree_id` empty) is measured every time.
pub(crate) fn tree_size(
    db: &mut Database,
    tree_id: &str,
    path: &std::path::Path,
    now: u64,
    skip_cache: bool,
) -> TreeSize {
    if !tree_id.is_empty()
        && !skip_cache
        && let Ok(Some(saved)) = db.tree_size(tree_id)
        && size_is_recent(saved.measured_at, now)
    {
        return saved;
    }
    let measured = disk::measure(path);
    let size = TreeSize {
        bytes: measured.bytes,
        partial: measured.partial,
        measured_at: now,
    };
    if !tree_id.is_empty() {
        // Only a cache: failing to save means measuring again next time.
        let _ = db.record_tree_size(tree_id, &size);
    }
    size
}

/// When a size was measured, as the API shows it.
pub(crate) fn measured_at(size: &TreeSize) -> String {
    rfc3339(size.measured_at)
}

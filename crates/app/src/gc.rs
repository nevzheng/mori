//! `mori gc`: sort every tree into remove, blocked, keep or never, and save the report.
//!
//! The report changes no tree. It records the bookmarks it sees pushed from each task tree before
//! fetching, so a bookmark the remote deleted since is still known, then fetches (unless offline)
//! and asks `gh` about pull requests.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mori_api::v1alpha1::gc_item::{Class as ItemClass, Kind, Outcome};
use mori_api::v1alpha1::{Disk, GcItem, GcResponse};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, RepoId, clone_path};
use mori_core::disk::{Candidate, pick_to_free};
use mori_core::error::{CleanupError, ErrorDetails, RepoError};
use mori_core::forest::{Entry, reconcile};
use mori_core::gc::{Class, Facts, Reason, classify, over_cap};
use mori_core::paths::Paths;
use mori_core::tree::{Lifetime, Role};
use mori_core::vcs::Forge;
use mori_store::database::Database;
use mori_store::records::{RepoRecord, TreeRecord};

use crate::bazel;
use crate::disk;
use crate::gc_apply;
use crate::landing::{self, TreeRef};
use crate::state::{self, boxed};
use crate::{App, Backend};

/// What `mori gc` was asked for.
pub struct GcArgs {
    pub repo: Option<String>,
    pub offline: bool,
    /// Set with `--apply`.
    pub apply: Option<Apply>,
    /// `--free <size>`: pick removable trees until they free this many bytes.
    pub free: Option<u64>,
}

/// What `--apply` was asked for.
pub struct Apply {
    pub yes: bool,
    pub names: Vec<String>,
    pub max: Option<u32>,
    pub dry_run: bool,
}

/// The default batch size for `--apply`.
const DEFAULT_MAX: u32 = 10;

/// Runs `mori gc`: the report, and with `--apply` a confirmed batch of removals.
///
/// # Errors
///
/// The plan's refusal, or a failure of the disk, the database or an adapter, with its code and
/// reason.
pub fn run<V: Backend, F: Forge>(
    app: &App<V, F>,
    args: &GcArgs,
) -> Result<GcResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths(&app.host)?;
    let mut db = state::open_database(&paths)?;
    let mut repos = db.repos().map_err(boxed)?;
    if let Some(repo) = &args.repo {
        let wanted = CloneUrl::parse(repo).map_err(boxed)?.repo.to_string();
        repos.retain(|record| record.remote == wanted);
        if repos.is_empty() {
            return Err(boxed(RepoError::NotManaged { repo: wanted }));
        }
    }
    let lru_max = state::tree_policy(&paths)?.lru.max;
    let context = Context {
        paths: &paths,
        vcs: &app.vcs,
        forge: (!args.offline).then_some(&app.forge as &dyn Forge),
        now: app.host.now,
        lru_max,
    };
    let free = args.free.filter(|bytes| *bytes > 0);
    let mut items = Vec::new();
    let mut idle = Vec::new();
    for repo in &repos {
        let ids: BTreeMap<String, String> = db
            .trees(&repo.id)
            .map_err(boxed)?
            .into_iter()
            .map(|record| (record.name, record.id))
            .collect();
        for (mut item, idle_seconds) in judge_repo(&context, &mut db, repo, args.offline)? {
            // Picking by size needs sizes that are current, so `--free` measures removable trees
            // again.
            let fresh = free.is_some() && item.class() == ItemClass::Remove;
            let id = ids.get(&item.name).cloned();
            add_size(&mut db, &mut item, id.as_ref(), app.host.now, fresh);
            items.push(item);
            idle.push(idle_seconds);
        }
    }
    for leftover in bazel::leftovers(&app.host, &paths, &repos) {
        items.push(leftover);
        idle.push(None);
    }
    let picked = free.map(|target| pick(&items, &idle, target));
    if args.apply.is_none()
        && let Some(picked) = &picked
    {
        for &index in picked {
            items[index].set_outcome(Outcome::WouldRemove);
        }
    }
    if let Some(apply) = &args.apply {
        let batch = batch(&items, apply, picked.as_deref());
        if !apply.yes && !apply.dry_run && !batch.is_empty() {
            return Err(boxed(CleanupError::NotConfirmed { count: batch.len() }));
        }
        for index in batch {
            let item = &items[index];
            if item.kind() == Kind::BazelLeftover {
                let outcome = bazel::remove(&app.host, &paths, &repos, item, apply.dry_run)?;
                items[index].set_outcome(outcome);
                continue;
            }
            // This run already fetched; checking again works from what it saw.
            let (outcome, reason, entry_id) = gc_apply::apply_one(
                &context,
                &mut db,
                &item.repo.clone(),
                &item.name.clone(),
                apply.dry_run,
            )?;
            let item = &mut items[index];
            if outcome == Outcome::SkippedUnsaved {
                item.set_class(ItemClass::Blocked);
            }
            item.set_outcome(outcome);
            item.reason = reason;
            item.entry_id = entry_id;
        }
    }
    let space = disk::space(&paths);
    Ok(GcResponse {
        items,
        validate_only: args.apply.as_ref().is_some_and(|apply| apply.dry_run),
        disk: space.map(|space| Disk {
            free_bytes: space.free,
            total_bytes: space.total,
        }),
        warnings: disk::low_space(&paths, space).into_iter().collect(),
    })
}

/// The items `--apply` removes, as indexes into `items`: what `--free` picked, else the removable
/// ones (only those named by `--only`, which names trees, not leftovers), at most `--max`.
fn batch(items: &[GcItem], apply: &Apply, picked: Option<&[usize]>) -> Vec<usize> {
    let max = match (apply.max.filter(|max| *max > 0), picked) {
        (Some(max), _) => usize::try_from(max).unwrap_or(usize::MAX),
        (None, Some(_)) => usize::MAX,
        (None, None) => usize::try_from(DEFAULT_MAX).unwrap_or(usize::MAX),
    };
    match picked {
        Some(picked) => picked.iter().copied().take(max).collect(),
        None => items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.class() == ItemClass::Remove)
            .filter(|(_, item)| {
                apply.names.is_empty()
                    || (item.kind() != Kind::BazelLeftover && apply.names.contains(&item.name))
            })
            .map(|(index, _)| index)
            .take(max)
            .collect(),
    }
}

/// Measures a tree that may go and exists on disk, or reuses its recent size.
fn add_size(db: &mut Database, item: &mut GcItem, tree_id: Option<&String>, now: u64, fresh: bool) {
    let may_go = matches!(
        item.class(),
        ItemClass::Remove | ItemClass::Blocked | ItemClass::Keep
    );
    if !may_go || item.reason == Reason::Missing.code() {
        return;
    }
    let id = tree_id.map_or("", String::as_str);
    item.size_bytes = disk::tree_size(db, id, std::path::Path::new(&item.path), now, fresh).bytes;
}

/// The removable items `--free` takes to free `target` bytes, as indexes into `items`.
fn pick(items: &[GcItem], idle: &[Option<u64>], target: u64) -> Vec<usize> {
    let removable: Vec<usize> = (0..items.len())
        .filter(|&index| items[index].class() == ItemClass::Remove)
        .collect();
    let candidates: Vec<Candidate> = removable
        .iter()
        .map(|&index| Candidate {
            bytes: items[index].size_bytes,
            idle_seconds: idle[index],
            cache_only: items[index].kind() == Kind::BazelLeftover,
        })
        .collect();
    pick_to_free(&candidates, target)
        .into_iter()
        .map(|picked| removable[picked])
        .collect()
}

/// What judging a repo's trees needs.
pub(crate) struct Context<'a, V> {
    pub paths: &'a Paths,
    pub vcs: &'a V,
    pub forge: Option<&'a dyn Forge>,
    pub now: u64,
    /// `[trees.lru] max`.
    pub lru_max: Option<u32>,
}

/// What one tree looks like before it is classified.
struct Row {
    name: String,
    path: PathBuf,
    facts: Facts,
    landed_count: usize,
}

/// Judges every tree of `repo`, fetching first unless `offline`. Reads only, apart from the
/// fetch and recording the bookmarks it sees.
pub(crate) fn report_repo<V: Backend>(
    context: &Context<V>,
    db: &mut Database,
    repo: &RepoRecord,
    offline: bool,
) -> Result<Vec<GcItem>, Box<dyn ErrorDetails>> {
    Ok(judge_repo(context, db, repo, offline)?
        .into_iter()
        .map(|(item, _)| item)
        .collect())
}

/// A tree in the report, and seconds since its latest change (none when unknown).
type Judged = (GcItem, Option<u64>);

/// As [`report_repo`], with each tree's seconds since its latest change.
fn judge_repo<V: Backend>(
    context: &Context<V>,
    db: &mut Database,
    repo: &RepoRecord,
    offline: bool,
) -> Result<Vec<Judged>, Box<dyn ErrorDetails>> {
    let id = CloneUrl::parse(&repo.remote).map_err(boxed)?.repo;
    let clone = clone_path(context.paths, &id);
    let records = db.trees(&repo.id).map_err(boxed)?;
    let workspaces = if clone.exists() {
        context.vcs.list(&clone).map_err(boxed)?
    } else {
        Vec::new()
    };
    // Record what is pushed now, before a fetch can make a deleted bookmark disappear.
    for record in records.iter().filter(|record| is_task(record)) {
        if workspaces
            .iter()
            .any(|workspace| workspace.name == record.name)
        {
            landing::observe(db, context.vcs, None, &tree_ref(&id, &clone, record))?;
        }
    }
    if !offline && clone.exists() {
        // A failed fetch leaves the clone's view as it was; the report still works from it.
        let _ = context.vcs.fetch(&clone);
    }
    let mut rows = Vec::new();
    for entry in reconcile(records, workspaces, |record| record.name.as_str()) {
        rows.push(row(context, db, repo, &id, &clone, entry)?);
    }
    let idle: Vec<(String, Option<u64>)> = rows
        .iter()
        .filter(|row| matches!(row.facts.recorded, Some((Role::Task, _))))
        .map(|row| (row.name.clone(), row.facts.idle_seconds))
        .collect();
    let beyond = over_cap(&idle, context.lru_max);
    Ok(rows
        .into_iter()
        .map(|mut row| {
            row.facts.over_cap = beyond.contains(&row.name);
            let (class, reason) = classify(&row.facts);
            let idle_seconds = row.facts.idle_seconds;
            let item = GcItem {
                repo: repo.remote.clone(),
                facts: describe(&row, reason),
                name: row.name,
                path: row.path.display().to_string(),
                class: item_class(class).into(),
                reason: reason.code().to_owned(),
                kind: Kind::Tree.into(),
                ..GcItem::default()
            };
            (item, idle_seconds)
        })
        .collect())
}

fn row<V: Backend>(
    context: &Context<V>,
    db: &mut Database,
    repo: &RepoRecord,
    id: &RepoId,
    clone: &std::path::Path,
    entry: Entry<TreeRecord>,
) -> Result<Row, Box<dyn ErrorDetails>> {
    let path_of = |record: &TreeRecord| {
        if record.name == BASE_TREE_NAME {
            clone.to_path_buf()
        } else {
            context
                .paths
                .trees()
                .join(&repo.dir_name)
                .join(&record.name)
        }
    };
    Ok(match entry {
        Entry::Foreign { workspace } => Row {
            name: workspace.name,
            path: workspace.root,
            facts: Facts {
                recorded: None,
                state: None,
                landed: None,
                idle_seconds: None,
                over_cap: false,
            },
            landed_count: 0,
        },
        Entry::Missing { record } => Row {
            name: record.name.clone(),
            path: path_of(&record),
            facts: Facts {
                recorded: Some((role(&record), lifetime(&record))),
                state: None,
                landed: None,
                idle_seconds: None,
                over_cap: false,
            },
            landed_count: 0,
        },
        Entry::Tree { record, workspace } => {
            let (landed, commits) = if is_task(&record) {
                let landing = landing::observe(
                    db,
                    context.vcs,
                    context.forge,
                    &tree_ref(id, clone, &record),
                )?;
                (landing.landed, landing.commits)
            } else {
                (None, Vec::new())
            };
            let state = context
                .vcs
                .state_covering(clone, &workspace.name, &commits)
                .map_err(boxed)?;
            let idle_seconds = context
                .vcs
                .last_change(clone, &workspace.name)
                .ok()
                .map(|changed| context.now.saturating_sub(changed));
            Row {
                name: record.name.clone(),
                path: workspace.root,
                facts: Facts {
                    recorded: Some((role(&record), lifetime(&record))),
                    state: Some(state),
                    landed,
                    idle_seconds,
                    over_cap: false,
                },
                landed_count: commits.len(),
            }
        }
    })
}

fn tree_ref<'a>(id: &'a RepoId, clone: &'a std::path::Path, record: &'a TreeRecord) -> TreeRef<'a> {
    TreeRef {
        repo: id,
        clone,
        id: &record.id,
        name: &record.name,
    }
}

pub(crate) fn is_task(record: &TreeRecord) -> bool {
    Role::of(&record.name) == Role::Task
}

fn role(record: &TreeRecord) -> Role {
    Role::of(&record.name)
}

/// An unreadable stored lifetime counts as pinned: the choice that never removes.
pub(crate) fn lifetime(record: &TreeRecord) -> Lifetime {
    record.lifetime.parse().unwrap_or(Lifetime::Pinned)
}

/// The facts behind a reason, in a few words.
fn describe(row: &Row, reason: Reason) -> String {
    let days = |seconds: u64| seconds / (24 * 60 * 60);
    match reason {
        Reason::Base => "the clone itself".to_owned(),
        Reason::Foreign => "a workspace mori didn't make".to_owned(),
        Reason::Pinned => "lifetime pinned".to_owned(),
        Reason::Missing => "its workspace is gone; only the record goes".to_owned(),
        Reason::Landed => format!("{} landed bookmark(s)", row.landed_count),
        Reason::OverCap => "beyond the repo's lru cap".to_owned(),
        Reason::Idle => format!(
            "no change for {} days",
            days(row.facts.idle_seconds.unwrap_or_default())
        ),
        Reason::NotYet => "its lifetime doesn't let it go yet".to_owned(),
        Reason::Unknown => {
            "can't tell yet (offline, gh not installed or not logged in, or no change time)"
                .to_owned()
        }
        Reason::Unsaved => {
            let state = row
                .facts
                .state
                .clone()
                .unwrap_or_else(|| mori_core::forest::TreeState {
                    change: String::new(),
                    changed: false,
                    unpushed: 0,
                });
            let mut parts = Vec::new();
            if state.changed {
                parts.push("edited".to_owned());
            }
            if state.unpushed > 0 {
                parts.push(format!("{} unpushed", state.unpushed));
            }
            parts.join(", ")
        }
    }
}

fn item_class(class: Class) -> ItemClass {
    match class {
        Class::Remove => ItemClass::Remove,
        Class::Blocked => ItemClass::Blocked,
        Class::Keep => ItemClass::Keep,
        Class::Never => ItemClass::Never,
    }
}

/// The class's name in reports and text.
#[must_use]
pub fn class_name(class: ItemClass) -> &'static str {
    match class {
        ItemClass::Remove => "remove",
        ItemClass::Blocked => "blocked",
        ItemClass::Keep => "keep",
        ItemClass::Never | ItemClass::Unspecified => "never",
    }
}

/// Counts the items per class, for the summary line.
#[must_use]
pub fn counts(items: &[GcItem]) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts.entry(class_name(item.class())).or_insert(0) += 1;
    }
    counts
}

//! `mori gc`: sort every tree into remove, blocked, keep or never, and save the report.
//!
//! The report changes no tree. It records the bookmarks it sees pushed from each task tree before
//! fetching, so a bookmark the remote deleted since is still known, then fetches (unless offline)
//! and asks `gh` about pull requests.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mori_api::v1alpha1::gc_item::{Class as ItemClass, Outcome};
use mori_api::v1alpha1::{GcItem, GcResponse};
use mori_core::clone::{BASE_TREE_NAME, CloneUrl, RepoId, clone_path};
use mori_core::error::{CleanupError, ErrorDetails, RepoError};
use mori_core::forest::{Entry, Workspaces, reconcile};
use mori_core::gc::{Class, Facts, Reason, classify, over_cap};
use mori_core::paths::Paths;
use mori_core::tree::{Lifetime, Role};
use mori_github::GhCli;
use mori_jj::JjCli;
use mori_store::database::Database;
use mori_store::gc::now;
use mori_store::records::{RepoRecord, TreeRecord};

use crate::gc_apply;
use crate::landing::{self, TreeRef};
use crate::state::{self, boxed};

/// The `gh` to use: `$MORI_GH` if set, else `gh` on `PATH`.
pub fn gh() -> GhCli {
    std::env::var_os("MORI_GH").map_or_else(GhCli::from_path, GhCli::new)
}

/// What `mori gc` was asked for.
pub struct GcArgs {
    pub repo: Option<String>,
    pub offline: bool,
    /// Set with `--apply`.
    pub apply: Option<Apply>,
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
pub fn run(args: &GcArgs) -> Result<GcResponse, Box<dyn ErrorDetails>> {
    let paths = state::paths()?;
    let mut db = state::open_database(&paths)?;
    let jj = JjCli::from_path();
    let gh = (!args.offline).then(gh);
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
        jj: &jj,
        gh: gh.as_ref(),
        now: now(),
        lru_max,
    };
    let mut items = Vec::new();
    for repo in &repos {
        items.extend(report_repo(&context, &mut db, repo, args.offline)?);
    }
    if let Some(apply) = &args.apply {
        let max = usize::try_from(apply.max.filter(|max| *max > 0).unwrap_or(DEFAULT_MAX))
            .unwrap_or(usize::MAX);
        let batch: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.class() == ItemClass::Remove)
            .filter(|(_, item)| apply.names.is_empty() || apply.names.contains(&item.name))
            .map(|(index, _)| index)
            .take(max)
            .collect();
        if !apply.yes && !apply.dry_run && !batch.is_empty() {
            return Err(boxed(CleanupError::NotConfirmed { count: batch.len() }));
        }
        for index in batch {
            let item = &items[index];
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
    Ok(GcResponse {
        items,
        validate_only: args.apply.as_ref().is_some_and(|apply| apply.dry_run),
    })
}

/// What judging a repo's trees needs.
pub struct Context<'a> {
    pub paths: &'a Paths,
    pub jj: &'a JjCli,
    pub gh: Option<&'a GhCli>,
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
pub fn report_repo(
    context: &Context,
    db: &mut Database,
    repo: &RepoRecord,
    offline: bool,
) -> Result<Vec<GcItem>, Box<dyn ErrorDetails>> {
    let id = CloneUrl::parse(&repo.remote).map_err(boxed)?.repo;
    let clone = clone_path(context.paths, &id);
    let records = db.trees(&repo.id).map_err(boxed)?;
    let workspaces = if clone.exists() {
        context.jj.list(&clone).map_err(boxed)?
    } else {
        Vec::new()
    };
    // Record what is pushed now, before a fetch can make a deleted bookmark disappear.
    for record in records.iter().filter(|record| is_task(record)) {
        if workspaces
            .iter()
            .any(|workspace| workspace.name == record.name)
        {
            landing::observe(db, context.jj, None, &tree_ref(&id, &clone, record))?;
        }
    }
    if !offline && clone.exists() {
        // A failed fetch leaves the clone's view as it was; the report still works from it.
        let _ = context.jj.fetch(&clone);
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
            GcItem {
                repo: repo.remote.clone(),
                facts: describe(&row, reason),
                name: row.name,
                path: row.path.display().to_string(),
                class: item_class(class).into(),
                reason: reason.code().to_owned(),
                ..GcItem::default()
            }
        })
        .collect())
}

fn row(
    context: &Context,
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
                let landing =
                    landing::observe(db, context.jj, context.gh, &tree_ref(id, clone, &record))?;
                (landing.landed, landing.commits)
            } else {
                (None, Vec::new())
            };
            let state = context
                .jj
                .state_covering(clone, &workspace.name, &commits)
                .map_err(boxed)?;
            let idle_seconds = context
                .jj
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

pub fn is_task(record: &TreeRecord) -> bool {
    record.role == Role::Task.as_str()
}

fn role(record: &TreeRecord) -> Role {
    if record.role == Role::Base.as_str() {
        Role::Base
    } else {
        Role::Task
    }
}

/// An unreadable stored lifetime counts as pinned: the choice that never removes.
pub fn lifetime(record: &TreeRecord) -> Lifetime {
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
        Reason::Unknown => "can't tell yet (offline, no gh, or no change time)".to_owned(),
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
pub fn class_name(class: ItemClass) -> &'static str {
    match class {
        ItemClass::Remove => "remove",
        ItemClass::Blocked => "blocked",
        ItemClass::Keep => "keep",
        ItemClass::Never | ItemClass::Unspecified => "never",
    }
}

/// Counts the items per class, for the summary line.
pub fn counts(items: &[GcItem]) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts.entry(class_name(item.class())).or_insert(0) += 1;
    }
    counts
}

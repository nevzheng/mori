//! Cleanup's files in mori's state directory: saved reports, so `mori gc apply` acts on exactly
//! what the person saw, and an append-only journal of every removal, so `mori restore` can undo
//! it. Both are plain JSON, readable without mori.

use std::io::{BufRead, ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mori_core::paths::Paths;
use serde::{Deserialize, Serialize};

use crate::StoreError;

/// One tree in a report.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ReportItem {
    /// The repo, e.g. `github.com/acme/widget`.
    pub repo: String,
    /// The tree's name.
    pub name: String,
    /// `remove`, `blocked`, `keep` or `never`.
    pub class: String,
    /// The reason code, e.g. `LANDED`.
    pub reason: String,
    /// The facts it was judged on, for people to read.
    pub facts: String,
}

/// A saved cleanup report. Never changed after it is saved.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Report {
    /// Its ID, e.g. `gc-1790532359-4f2a0001`.
    pub id: String,
    /// When it was made (seconds since the Unix epoch).
    pub created_at: u64,
    /// Every tree, sorted by repo and name.
    pub items: Vec<ReportItem>,
}

/// A bookmark a removed tree's work was reachable from.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct JournalBookmark {
    /// The remote.
    pub remote: String,
    /// The bookmark.
    pub bookmark: String,
    /// Where it pointed when last seen.
    pub commit_id: String,
}

/// One removal: everything needed to understand and undo it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct JournalEntry {
    /// Its ID, e.g. `j-1790532359-4f2a0001`.
    pub id: String,
    /// When the tree was removed (seconds since the Unix epoch).
    pub removed_at: u64,
    /// The report the removal came from, if any.
    pub report_id: Option<String>,
    /// The repo.
    pub repo: String,
    /// The tree's record ID, reused if it is restored.
    pub tree_id: String,
    /// The tree's name.
    pub name: String,
    /// Where it was.
    pub path: String,
    /// Who it was for.
    pub owner: String,
    /// Its task.
    pub task: Option<String>,
    /// Its lifetime.
    pub lifetime: String,
    /// The commit its working copy was on.
    pub commit_id: String,
    /// The ref that pins that commit, e.g. `refs/mori/removed/j-1790532359-4f2a0001`.
    pub pin: String,
    /// The bookmarks its work was reachable from.
    pub bookmarks: Vec<JournalBookmark>,
}

/// IDs made so far by this process, so two made in the same clock tick still differ.
static MADE: AtomicU32 = AtomicU32::new(0);

/// A new ID with `prefix`: the time in seconds, then hex digits from the process ID and a
/// per-process counter, so IDs differ and sort by time.
#[must_use]
pub fn new_id(prefix: &str) -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let count = MADE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{prefix}-{seconds}-{:04x}{:04x}",
        std::process::id() & 0xffff,
        count & 0xffff
    )
}

/// Seconds since the Unix epoch, now.
#[must_use]
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|now| now.as_secs())
        .unwrap_or_default()
}

fn reports_dir(paths: &Paths) -> PathBuf {
    paths.state_dir.join("reports")
}

/// The journal file.
#[must_use]
pub fn journal_path(paths: &Paths) -> PathBuf {
    paths.state_dir.join("journal.jsonl")
}

/// Saves `report` as `reports/<id>.json`; refuses to replace one.
///
/// # Errors
///
/// [`StoreError::AlreadyExists`] if that report exists, otherwise I/O errors.
pub fn save_report(paths: &Paths, report: &Report) -> Result<PathBuf, StoreError> {
    let dir = reports_dir(paths);
    std::fs::create_dir_all(&dir).map_err(|source| io(&dir, source))?;
    let path = dir.join(format!("{}.json", report.id));
    let text = serde_json::to_string_pretty(report).map_err(|error| io(&path, error.into()))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .map_err(|source| match source.kind() {
            ErrorKind::AlreadyExists => StoreError::AlreadyExists { path: path.clone() },
            _ => io(&path, source),
        })?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
        .map_err(|source| io(&path, source))?;
    Ok(path)
}

/// Loads the report with `id`.
///
/// # Errors
///
/// I/O errors (including "not found") and a report that isn't valid JSON.
pub fn load_report(paths: &Paths, id: &str) -> Result<Report, StoreError> {
    let path = reports_dir(paths).join(format!("{id}.json"));
    let text = std::fs::read_to_string(&path).map_err(|source| io(&path, source))?;
    serde_json::from_str(&text).map_err(|error| io(&path, error.into()))
}

/// Appends one entry to the journal, as one line, and syncs it to disk before returning.
///
/// # Errors
///
/// I/O errors.
pub fn append_journal(paths: &Paths, entry: &JournalEntry) -> Result<(), StoreError> {
    let path = journal_path(paths);
    let line = serde_json::to_string(entry).map_err(|error| io(&path, error.into()))?;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .mode(0o600)
        .open(&path)
        .map_err(|source| io(&path, source))?;
    file.write_all(format!("{line}\n").as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| io(&path, source))
}

/// Every journal entry, oldest first. A line that doesn't parse (a crash mid-write) is skipped.
///
/// # Errors
///
/// I/O errors other than "not found".
pub fn read_journal(paths: &Paths) -> Result<Vec<JournalEntry>, StoreError> {
    let path = journal_path(paths);
    let file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(io(&path, source)),
    };
    let mut entries = Vec::new();
    for line in std::io::BufReader::new(file).lines() {
        let line = line.map_err(|source| io(&path, source))?;
        if let Ok(entry) = serde_json::from_str(&line) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn io(path: &Path, source: std::io::Error) -> StoreError {
    StoreError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use mori_core::paths::Env;

    use super::*;

    fn paths(home: &Path) -> Paths {
        let paths = Paths::resolve(&Env {
            home: Some(home.to_path_buf()),
            ..Env::default()
        })
        .unwrap();
        std::fs::create_dir_all(&paths.state_dir).unwrap();
        paths
    }

    fn entry(id: &str) -> JournalEntry {
        JournalEntry {
            id: id.to_owned(),
            removed_at: 1,
            report_id: Some("gc-1-000000".to_owned()),
            repo: "github.com/acme/widget".to_owned(),
            tree_id: "tree_1".to_owned(),
            name: "claude-fix-login".to_owned(),
            path: "/home/acme/mori/trees/widget/claude-fix-login".to_owned(),
            owner: "claude".to_owned(),
            task: Some("fix-login".to_owned()),
            lifetime: "task-done".to_owned(),
            commit_id: "aaaa".to_owned(),
            pin: format!("refs/mori/removed/{id}"),
            bookmarks: vec![JournalBookmark {
                remote: "origin".to_owned(),
                bookmark: "claude/fix-login".to_owned(),
                commit_id: "aaaa".to_owned(),
            }],
        }
    }

    #[test]
    fn a_report_round_trips_and_is_never_replaced() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        let report = Report {
            id: new_id("gc"),
            created_at: now(),
            items: vec![ReportItem {
                repo: "github.com/acme/widget".to_owned(),
                name: "claude-fix-login".to_owned(),
                class: "remove".to_owned(),
                reason: "LANDED".to_owned(),
                facts: "bookmark claude/fix-login gone from origin".to_owned(),
            }],
        };

        save_report(&paths, &report).unwrap();

        assert_eq!(load_report(&paths, &report.id).unwrap(), report);
        assert!(matches!(
            save_report(&paths, &report).unwrap_err(),
            StoreError::AlreadyExists { .. }
        ));
    }

    #[test]
    fn the_journal_appends_and_reads_back_in_order() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        assert_eq!(read_journal(&paths).unwrap(), []);

        append_journal(&paths, &entry("j-1")).unwrap();
        append_journal(&paths, &entry("j-2")).unwrap();

        let ids: Vec<String> = read_journal(&paths)
            .unwrap()
            .into_iter()
            .map(|entry| entry.id)
            .collect();
        assert_eq!(ids, ["j-1", "j-2"]);
    }

    #[test]
    fn a_torn_journal_line_is_skipped() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        append_journal(&paths, &entry("j-1")).unwrap();
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(journal_path(&paths))
            .unwrap();
        file.write_all(b"{\"id\": \"j-2\", \"remo").unwrap();

        assert_eq!(read_journal(&paths).unwrap().len(), 1);
    }

    #[test]
    fn ids_have_their_prefix_and_differ() {
        let a = new_id("j");
        let b = new_id("j");

        assert!(a.starts_with("j-"), "{a}");
        assert_ne!(a, b);
    }
}

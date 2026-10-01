//! mori's records of the repos and trees it created. Only what the VCS can't tell mori: who made a
//! tree, for what, and how long it may live. IDs and timestamps come from SQLite, so callers need
//! no clock or random source.

use rusqlite::ffi::{SQLITE_CONSTRAINT_PRIMARYKEY, SQLITE_CONSTRAINT_UNIQUE};
use rusqlite::{OptionalExtension, params};

use crate::StoreError;
use crate::database::Database;

/// A repo to record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewRepo<'a> {
    /// Its identity, e.g. `github.com/acme/widget`.
    pub remote: &'a str,
    /// Its directory under `trees/`.
    pub dir_name: &'a str,
}

/// A tree to record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewTree<'a> {
    /// Its name, which is also its workspace name.
    pub name: &'a str,
    /// Who made it: `you`, or an agent such as `claude`.
    pub owner: &'a str,
    /// Its task, if any.
    pub task: Option<&'a str>,
    /// Its lifetime, in the text form `mori_core::tree::Lifetime` prints.
    pub lifetime: &'a str,
}

/// A recorded repo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoRecord {
    /// Its opaque ID.
    pub id: String,
    /// Its identity.
    pub remote: String,
    /// Its directory under `trees/`.
    pub dir_name: String,
    /// When mori recorded it (RFC 3339, UTC).
    pub created_at: String,
}

/// A recorded tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRecord {
    /// Its opaque ID.
    pub id: String,
    /// The ID of its repo.
    pub repo_id: String,
    /// Its name, which is also its workspace name.
    pub name: String,
    /// Who made it.
    pub owner: String,
    /// Its task, if any.
    pub task: Option<String>,
    /// Its lifetime.
    pub lifetime: String,
    /// When mori recorded it (RFC 3339, UTC).
    pub created_at: String,
    /// When mori last saw it used (RFC 3339, UTC).
    pub last_used_at: String,
}

/// A remote bookmark mori saw pointing into a tree's own history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeBookmark {
    /// The remote, e.g. `origin`.
    pub remote: String,
    /// The bookmark, e.g. `claude/fix-login`.
    pub bookmark: String,
    /// Where it pointed when last seen.
    pub commit_id: String,
    /// When mori last saw it (RFC 3339, UTC).
    pub seen_at: String,
}

/// A tree's size as last measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeSize {
    /// Bytes on disk.
    pub bytes: u64,
    /// True if some files couldn't be read.
    pub partial: bool,
    /// When it was measured (seconds since the Unix epoch).
    pub measured_at: u64,
}

// Opaque, random, never reused: a prefix and 128 random bits.
const NEW_ID: &str = "lower(hex(randomblob(16)))";
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

impl Database {
    /// Records a repo mori just cloned, and its base tree, all or nothing. Returns the new
    /// records.
    ///
    /// # Errors
    ///
    /// [`StoreError::AlreadyRecorded`] if the repo, its `trees/` directory name or the tree is
    /// already recorded (nothing is written), otherwise a SQLite error.
    pub fn record_clone(
        &mut self,
        repo: &NewRepo,
        base: &NewTree,
    ) -> Result<(RepoRecord, TreeRecord), StoreError> {
        let path = self.path.clone();
        let sqlite = |source: rusqlite::Error| {
            if is_key_clash(&source) {
                StoreError::AlreadyRecorded {
                    path: path.clone(),
                    what: format!(
                        "repo {} or its trees/ directory {}",
                        repo.remote, repo.dir_name
                    ),
                }
            } else {
                StoreError::Sqlite {
                    path: path.clone(),
                    source,
                }
            }
        };
        let tx = self.conn.transaction().map_err(sqlite)?;
        let repo_id: String = tx
            .query_row(
                &format!(
                    "INSERT INTO repos (id, remote, dir_name, created_at)
                     VALUES ('repo_' || {NEW_ID}, ?1, ?2, {NOW}) RETURNING id"
                ),
                params![repo.remote, repo.dir_name],
                |row| row.get(0),
            )
            .map_err(sqlite)?;
        tx.execute(
            &format!(
                "INSERT INTO trees
                     (id, repo_id, name, owner, task, lifetime, created_at, last_used_at)
                 VALUES ('tree_' || {NEW_ID}, ?1, ?2, ?3, ?4, ?5, {NOW}, {NOW})"
            ),
            params![repo_id, base.name, base.owner, base.task, base.lifetime],
        )
        .map_err(sqlite)?;
        tx.commit().map_err(sqlite)?;

        let recorded = self.repo(repo.remote)?.ok_or_else(|| StoreError::NotMori {
            path: self.path.clone(),
        })?;
        let trees = self.trees(&recorded.id)?;
        let base = trees
            .into_iter()
            .next()
            .ok_or_else(|| StoreError::NotMori {
                path: self.path.clone(),
            })?;
        Ok((recorded, base))
    }

    /// Records a task tree mori just created in the repo with ID `repo_id`, and returns it.
    ///
    /// # Errors
    ///
    /// [`StoreError::AlreadyRecorded`] if the repo already has a tree of that name (nothing is
    /// written), otherwise a SQLite error, including when no repo has that ID.
    pub fn record_tree(&mut self, repo_id: &str, tree: &NewTree) -> Result<TreeRecord, StoreError> {
        let path = self.path.clone();
        let sqlite = |source: rusqlite::Error| {
            if is_key_clash(&source) {
                StoreError::AlreadyRecorded {
                    path: path.clone(),
                    what: format!("tree {}", tree.name),
                }
            } else {
                StoreError::Sqlite {
                    path: path.clone(),
                    source,
                }
            }
        };
        self.conn
            .query_row(
                &format!(
                    "INSERT INTO trees
                         (id, repo_id, name, owner, task, lifetime, created_at, last_used_at)
                     VALUES ('tree_' || {NEW_ID}, ?1, ?2, ?3, ?4, ?5, {NOW}, {NOW})
                     RETURNING {TREE_COLUMNS}"
                ),
                params![repo_id, tree.name, tree.owner, tree.task, tree.lifetime],
                tree_record,
            )
            .map_err(sqlite)
    }

    /// Drops the record of the tree named `name` in the repo with ID `repo_id`. Returns whether
    /// there was one. Only records change: the tree's files are the caller's to remove first.
    ///
    /// # Errors
    ///
    /// A SQLite error.
    pub fn delete_tree(&mut self, repo_id: &str, name: &str) -> Result<bool, StoreError> {
        let path = self.path.clone();
        let sqlite = |source| StoreError::Sqlite {
            path: path.clone(),
            source,
        };
        let tx = self.conn.transaction().map_err(sqlite)?;
        for table in ["tree_bookmarks", "tree_sizes"] {
            tx.execute(
                &format!(
                    "DELETE FROM {table} WHERE tree_id IN
                         (SELECT id FROM trees WHERE repo_id = ?1 AND name = ?2)"
                ),
                params![repo_id, name],
            )
            .map_err(sqlite)?;
        }
        let deleted = tx
            .execute(
                "DELETE FROM trees WHERE repo_id = ?1 AND name = ?2",
                params![repo_id, name],
            )
            .map_err(sqlite)?;
        tx.commit().map_err(sqlite)?;
        Ok(deleted > 0)
    }

    /// Notes that `bookmark` on `remote` points into the tree with ID `tree_id`, at `commit_id`.
    /// A new sighting adds a row; a moved bookmark updates it.
    ///
    /// # Errors
    ///
    /// A SQLite error, including when no tree has that ID.
    pub fn record_tree_bookmark(
        &mut self,
        tree_id: &str,
        remote: &str,
        bookmark: &str,
        commit_id: &str,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                &format!(
                    "INSERT INTO tree_bookmarks (tree_id, remote, bookmark, commit_id, seen_at)
                     VALUES (?1, ?2, ?3, ?4, {NOW})
                     ON CONFLICT (tree_id, remote, bookmark)
                     DO UPDATE SET commit_id = excluded.commit_id, seen_at = excluded.seen_at"
                ),
                params![tree_id, remote, bookmark, commit_id],
            )
            .map(|_| ())
            .map_err(|source| self.sqlite(source))
    }

    /// Forgets that `bookmark` on `remote` was pushed from the tree with ID `tree_id`. Forgetting
    /// one that isn't recorded does nothing.
    ///
    /// # Errors
    ///
    /// A SQLite error.
    pub fn forget_tree_bookmark(
        &mut self,
        tree_id: &str,
        remote: &str,
        bookmark: &str,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM tree_bookmarks WHERE tree_id = ?1 AND remote = ?2 AND bookmark = ?3",
                params![tree_id, remote, bookmark],
            )
            .map(|_| ())
            .map_err(|source| self.sqlite(source))
    }

    /// Every bookmark mori has seen pushed from the tree with ID `tree_id`, sorted.
    ///
    /// # Errors
    ///
    /// A SQLite error.
    pub fn tree_bookmarks(&self, tree_id: &str) -> Result<Vec<TreeBookmark>, StoreError> {
        let sqlite = |source| self.sqlite(source);
        let mut statement = self
            .conn
            .prepare(
                "SELECT remote, bookmark, commit_id, seen_at FROM tree_bookmarks
                 WHERE tree_id = ?1 ORDER BY remote, bookmark",
            )
            .map_err(sqlite)?;
        let rows = statement
            .query_map([tree_id], |row| {
                Ok(TreeBookmark {
                    remote: row.get(0)?,
                    bookmark: row.get(1)?,
                    commit_id: row.get(2)?,
                    seen_at: row.get(3)?,
                })
            })
            .map_err(sqlite)?;
        rows.collect::<Result<_, _>>().map_err(sqlite)
    }

    /// Saves the size just measured for the tree with ID `tree_id`, at `measured_at` (seconds
    /// since the Unix epoch).
    ///
    /// # Errors
    ///
    /// A SQLite error, including when no tree has that ID.
    pub fn record_tree_size(&mut self, tree_id: &str, size: &TreeSize) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO tree_sizes (tree_id, bytes, partial, measured_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (tree_id) DO UPDATE SET bytes = excluded.bytes,
                     partial = excluded.partial, measured_at = excluded.measured_at",
                params![
                    tree_id,
                    i64::try_from(size.bytes).unwrap_or(i64::MAX),
                    size.partial,
                    i64::try_from(size.measured_at).unwrap_or(i64::MAX),
                ],
            )
            .map(|_| ())
            .map_err(|source| self.sqlite(source))
    }

    /// The last size saved for the tree with ID `tree_id`, if any.
    ///
    /// # Errors
    ///
    /// A SQLite error.
    pub fn tree_size(&self, tree_id: &str) -> Result<Option<TreeSize>, StoreError> {
        self.conn
            .query_row(
                "SELECT bytes, partial, measured_at FROM tree_sizes WHERE tree_id = ?1",
                [tree_id],
                |row| {
                    Ok(TreeSize {
                        bytes: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                        partial: row.get(1)?,
                        measured_at: u64::try_from(row.get::<_, i64>(2)?).unwrap_or(0),
                    })
                },
            )
            .optional()
            .map_err(|source| self.sqlite(source))
    }

    /// Records a restored tree under its old ID, so it is the same tree coming back.
    ///
    /// # Errors
    ///
    /// [`StoreError::AlreadyRecorded`] if the ID or the name is taken (nothing is written),
    /// otherwise a SQLite error.
    pub fn restore_tree(
        &mut self,
        repo_id: &str,
        tree_id: &str,
        tree: &NewTree,
    ) -> Result<TreeRecord, StoreError> {
        let path = self.path.clone();
        let sqlite = |source: rusqlite::Error| {
            if is_key_clash(&source) {
                StoreError::AlreadyRecorded {
                    path: path.clone(),
                    what: format!("tree {} ({tree_id})", tree.name),
                }
            } else {
                StoreError::Sqlite {
                    path: path.clone(),
                    source,
                }
            }
        };
        self.conn
            .query_row(
                &format!(
                    "INSERT INTO trees
                         (id, repo_id, name, owner, task, lifetime, created_at, last_used_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, {NOW}, {NOW})
                     RETURNING {TREE_COLUMNS}"
                ),
                params![
                    tree_id,
                    repo_id,
                    tree.name,
                    tree.owner,
                    tree.task,
                    tree.lifetime
                ],
                tree_record,
            )
            .map_err(sqlite)
    }

    /// The recorded repo with identity `remote`, if any.
    ///
    /// # Errors
    ///
    /// A SQLite error.
    pub fn repo(&self, remote: &str) -> Result<Option<RepoRecord>, StoreError> {
        self.conn
            .query_row(
                "SELECT id, remote, dir_name, created_at FROM repos WHERE remote = ?1",
                [remote],
                repo_record,
            )
            .optional()
            .map_err(|source| self.sqlite(source))
    }

    /// Every recorded repo, sorted by identity.
    ///
    /// # Errors
    ///
    /// A SQLite error.
    pub fn repos(&self) -> Result<Vec<RepoRecord>, StoreError> {
        let sqlite = |source| self.sqlite(source);
        let mut statement = self
            .conn
            .prepare("SELECT id, remote, dir_name, created_at FROM repos ORDER BY remote")
            .map_err(sqlite)?;
        let rows = statement.query_map([], repo_record).map_err(sqlite)?;
        rows.collect::<Result<_, _>>().map_err(sqlite)
    }

    /// Every recorded tree of the repo with ID `repo_id`, sorted by name.
    ///
    /// # Errors
    ///
    /// A SQLite error.
    pub fn trees(&self, repo_id: &str) -> Result<Vec<TreeRecord>, StoreError> {
        let sqlite = |source| self.sqlite(source);
        let mut statement = self
            .conn
            .prepare(&format!(
                "SELECT {TREE_COLUMNS} FROM trees WHERE repo_id = ?1 ORDER BY name"
            ))
            .map_err(sqlite)?;
        let rows = statement
            .query_map([repo_id], tree_record)
            .map_err(sqlite)?;
        rows.collect::<Result<_, _>>().map_err(sqlite)
    }
}

/// Only a clash on a key means "already recorded"; any other constraint failure is a bug.
fn is_key_clash(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(failure, _)
            if matches!(
                failure.extended_code,
                SQLITE_CONSTRAINT_UNIQUE | SQLITE_CONSTRAINT_PRIMARYKEY
            )
    )
}

const TREE_COLUMNS: &str = "id, repo_id, name, owner, task, lifetime, created_at, last_used_at";

fn tree_record(row: &rusqlite::Row) -> rusqlite::Result<TreeRecord> {
    Ok(TreeRecord {
        id: row.get(0)?,
        repo_id: row.get(1)?,
        name: row.get(2)?,
        owner: row.get(3)?,
        task: row.get(4)?,
        lifetime: row.get(5)?,
        created_at: row.get(6)?,
        last_used_at: row.get(7)?,
    })
}

fn repo_record(row: &rusqlite::Row) -> rusqlite::Result<RepoRecord> {
    Ok(RepoRecord {
        id: row.get(0)?,
        remote: row.get(1)?,
        dir_name: row.get(2)?,
        created_at: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use mori_core::error::{Code, ErrorDetails};

    use super::*;

    fn database() -> Result<(tempfile::TempDir, Database), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let db = Database::create(&dir.path().join("mori.db"), "/home/acme/mori", 0o600)?;
        Ok((dir, db))
    }

    fn widget() -> NewRepo<'static> {
        NewRepo {
            remote: "github.com/acme/widget",
            dir_name: "widget",
        }
    }

    fn base() -> NewTree<'static> {
        NewTree {
            name: "default",
            owner: "you",
            task: None,
            lifetime: "pinned",
        }
    }

    #[test]
    fn a_clone_is_recorded_with_its_base_tree() {
        let (_dir, mut db) = database().unwrap();

        let (repo, tree) = db.record_clone(&widget(), &base()).unwrap();

        assert_eq!(repo.remote, "github.com/acme/widget");
        assert_eq!(repo.dir_name, "widget");
        assert!(
            repo.id.starts_with("repo_") && repo.id.len() == 37,
            "{}",
            repo.id
        );
        assert_eq!(tree.repo_id, repo.id);
        assert!(tree.id.starts_with("tree_"), "{}", tree.id);
        assert_eq!(
            (tree.name.as_str(), tree.lifetime.as_str()),
            ("default", "pinned")
        );
        assert_eq!(tree.created_at, tree.last_used_at);
        assert!(tree.created_at.ends_with('Z'), "{}", tree.created_at);
        assert_eq!(db.repos().unwrap(), [repo]);
    }

    #[test]
    fn a_repo_is_recorded_once() {
        let (_dir, mut db) = database().unwrap();
        db.record_clone(&widget(), &base()).unwrap();

        let error = db.record_clone(&widget(), &base()).unwrap_err();

        assert_eq!(error.code(), Code::AlreadyExists);
        assert_eq!(error.reason(), "ALREADY_RECORDED");
        assert_eq!(db.repos().unwrap().len(), 1);
    }

    #[test]
    fn a_second_repo_cannot_take_the_same_tree_dir() {
        let (_dir, mut db) = database().unwrap();
        db.record_clone(&widget(), &base()).unwrap();
        let other = NewRepo {
            remote: "github.com/other/widget",
            dir_name: "widget",
        };

        let error = db.record_clone(&other, &base()).unwrap_err();

        assert_eq!(error.reason(), "ALREADY_RECORDED");
    }

    #[test]
    fn a_failed_record_writes_nothing() {
        let (_dir, mut db) = database().unwrap();
        let bad_tree = NewTree {
            lifetime: "forever",
            ..base()
        };

        let error = db.record_clone(&widget(), &bad_tree).unwrap_err();

        assert_eq!(
            error.reason(),
            "DATABASE_ERROR",
            "a bad lifetime is a bug, not a duplicate"
        );
        assert_eq!(db.repos().unwrap(), []);
    }

    #[test]
    fn records_survive_reopening() {
        let (dir, mut db) = database().unwrap();
        let (repo, tree) = db.record_clone(&widget(), &base()).unwrap();
        drop(db);

        let db = Database::open(&PathBuf::from(dir.path()).join("mori.db")).unwrap();

        assert_eq!(
            db.repo("github.com/acme/widget").unwrap(),
            Some(repo.clone())
        );
        assert_eq!(db.trees(&repo.id).unwrap(), [tree]);
        assert_eq!(db.repo("github.com/acme/gadget").unwrap(), None);
    }

    fn task_tree(name: &'static str) -> NewTree<'static> {
        NewTree {
            name,
            owner: "claude",
            task: Some("fix-login"),
            lifetime: "task-done",
        }
    }

    #[test]
    fn a_task_tree_is_recorded_beside_the_base_tree() {
        let (_dir, mut db) = database().unwrap();
        let (repo, _) = db.record_clone(&widget(), &base()).unwrap();

        let tree = db
            .record_tree(&repo.id, &task_tree("claude-fix-login"))
            .unwrap();

        assert!(tree.id.starts_with("tree_"), "{}", tree.id);
        assert_eq!(tree.repo_id, repo.id);
        assert_eq!(tree.task.as_deref(), Some("fix-login"));
        let names: Vec<_> = db
            .trees(&repo.id)
            .unwrap()
            .into_iter()
            .map(|tree| tree.name)
            .collect();
        assert_eq!(names, ["claude-fix-login", "default"]);
    }

    #[test]
    fn a_tree_name_is_recorded_once_per_repo() {
        let (_dir, mut db) = database().unwrap();
        let (repo, _) = db.record_clone(&widget(), &base()).unwrap();
        db.record_tree(&repo.id, &task_tree("claude-fix-login"))
            .unwrap();

        let error = db
            .record_tree(&repo.id, &task_tree("claude-fix-login"))
            .unwrap_err();

        assert_eq!(error.code(), Code::AlreadyExists);
        assert_eq!(error.reason(), "ALREADY_RECORDED");
    }

    #[test]
    fn a_tree_needs_a_recorded_repo() {
        let (_dir, mut db) = database().unwrap();

        let error = db
            .record_tree("repo_missing", &task_tree("claude-fix-login"))
            .unwrap_err();

        assert_eq!(error.reason(), "DATABASE_ERROR");
    }

    #[test]
    fn a_tree_record_is_dropped_once() {
        let (_dir, mut db) = database().unwrap();
        let (repo, _) = db.record_clone(&widget(), &base()).unwrap();
        db.record_tree(&repo.id, &task_tree("claude-fix-login"))
            .unwrap();

        assert!(db.delete_tree(&repo.id, "claude-fix-login").unwrap());
        assert!(!db.delete_tree(&repo.id, "claude-fix-login").unwrap());

        let names: Vec<_> = db
            .trees(&repo.id)
            .unwrap()
            .into_iter()
            .map(|tree| tree.name)
            .collect();
        assert_eq!(names, ["default"]);
    }

    #[test]
    fn pushed_bookmarks_are_recorded_updated_and_dropped_with_the_tree() {
        let (_dir, mut db) = database().unwrap();
        let (repo, _) = db.record_clone(&widget(), &base()).unwrap();
        let tree = db
            .record_tree(&repo.id, &task_tree("claude-fix-login"))
            .unwrap();

        db.record_tree_bookmark(&tree.id, "origin", "claude/fix-login", "aaaa")
            .unwrap();
        db.record_tree_bookmark(&tree.id, "origin", "claude/fix-login", "bbbb")
            .unwrap();

        let seen = db.tree_bookmarks(&tree.id).unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].bookmark, "claude/fix-login");
        assert_eq!(seen[0].commit_id, "bbbb");

        db.record_tree_bookmark(&tree.id, "origin", "claude/other", "cccc")
            .unwrap();
        db.forget_tree_bookmark(&tree.id, "origin", "claude/other")
            .unwrap();
        db.forget_tree_bookmark(&tree.id, "origin", "claude/never-seen")
            .unwrap();
        assert_eq!(db.tree_bookmarks(&tree.id).unwrap(), seen);

        assert!(db.delete_tree(&repo.id, "claude-fix-login").unwrap());
        assert_eq!(db.tree_bookmarks(&tree.id).unwrap(), []);
    }

    #[test]
    fn a_tree_size_is_saved_replaced_and_dropped_with_the_tree() {
        let (_dir, mut db) = database().unwrap();
        let (repo, _) = db.record_clone(&widget(), &base()).unwrap();
        let tree = db
            .record_tree(&repo.id, &task_tree("claude-fix-login"))
            .unwrap();
        let size = |bytes, measured_at| TreeSize {
            bytes,
            partial: false,
            measured_at,
        };
        assert_eq!(db.tree_size(&tree.id).unwrap(), None);

        db.record_tree_size(&tree.id, &size(100, 1)).unwrap();
        db.record_tree_size(&tree.id, &size(200, 2)).unwrap();

        assert_eq!(db.tree_size(&tree.id).unwrap(), Some(size(200, 2)));
        assert!(db.delete_tree(&repo.id, "claude-fix-login").unwrap());
        assert_eq!(db.tree_size(&tree.id).unwrap(), None);
    }

    #[test]
    fn a_restored_tree_keeps_its_old_id() {
        let (_dir, mut db) = database().unwrap();
        let (repo, _) = db.record_clone(&widget(), &base()).unwrap();
        let tree = db
            .record_tree(&repo.id, &task_tree("claude-fix-login"))
            .unwrap();
        db.delete_tree(&repo.id, "claude-fix-login").unwrap();

        let back = db
            .restore_tree(&repo.id, &tree.id, &task_tree("claude-fix-login"))
            .unwrap();

        assert_eq!(back.id, tree.id);
        let again = db
            .restore_tree(&repo.id, &tree.id, &task_tree("claude-fix-login"))
            .unwrap_err();
        assert_eq!(again.reason(), "ALREADY_RECORDED");
    }
}

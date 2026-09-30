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
    /// `base` or `task`.
    pub role: &'a str,
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
    /// `base` or `task`.
    pub role: String,
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
        // Only a clash on a key means "already recorded"; any other constraint is a bug.
        let sqlite = |source: rusqlite::Error| match &source {
            rusqlite::Error::SqliteFailure(error, _)
                if matches!(
                    error.extended_code,
                    SQLITE_CONSTRAINT_UNIQUE | SQLITE_CONSTRAINT_PRIMARYKEY
                ) =>
            {
                StoreError::AlreadyRecorded {
                    path: path.clone(),
                    what: format!(
                        "repo {} or its trees/ directory {}",
                        repo.remote, repo.dir_name
                    ),
                }
            }
            _ => StoreError::Sqlite {
                path: path.clone(),
                source,
            },
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
                     (id, repo_id, name, role, owner, task, lifetime, created_at, last_used_at)
                 VALUES ('tree_' || {NEW_ID}, ?1, ?2, ?3, ?4, ?5, ?6, {NOW}, {NOW})"
            ),
            params![
                repo_id,
                base.name,
                base.role,
                base.owner,
                base.task,
                base.lifetime
            ],
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
            .prepare(
                "SELECT id, repo_id, name, role, owner, task, lifetime, created_at, last_used_at
                 FROM trees WHERE repo_id = ?1 ORDER BY name",
            )
            .map_err(sqlite)?;
        let rows = statement
            .query_map([repo_id], |row| {
                Ok(TreeRecord {
                    id: row.get(0)?,
                    repo_id: row.get(1)?,
                    name: row.get(2)?,
                    role: row.get(3)?,
                    owner: row.get(4)?,
                    task: row.get(5)?,
                    lifetime: row.get(6)?,
                    created_at: row.get(7)?,
                    last_used_at: row.get(8)?,
                })
            })
            .map_err(sqlite)?;
        rows.collect::<Result<_, _>>().map_err(sqlite)
    }
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
            role: "base",
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
            (
                tree.name.as_str(),
                tree.role.as_str(),
                tree.lifetime.as_str()
            ),
            ("default", "base", "pinned")
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
            role: "lead",
            ..base()
        };

        let error = db.record_clone(&widget(), &bad_tree).unwrap_err();

        assert_eq!(
            error.reason(),
            "DATABASE_ERROR",
            "a bad role is a bug, not a duplicate"
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
}

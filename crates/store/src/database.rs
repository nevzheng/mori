//! The state database: one SQLite file that only its owner can read.
//!
//! Its schema version is SQLite's `user_version`, and its `application_id` marks it as mori's, so
//! mori never writes to another program's database by mistake. Opening a database an older mori
//! wrote upgrades it in place, all or nothing.
//!
//! It holds only what the VCS can't tell mori: who made a tree, for what, and how long it may live.
//! Branches, HEAD and dirty or pushed state are always read from the VCS, never stored here.

use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use rusqlite::{
    Connection, ErrorCode, OpenFlags, OptionalExtension, Transaction, TransactionBehavior,
};

use crate::StoreError;

/// The `application_id` of a mori database: "mori" in ASCII.
const APPLICATION_ID: i32 = 0x6d6f_7269;

/// How long to wait for another mori process to finish writing before giving up.
const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// The schema version this mori writes, and the newest it reads.
pub const SCHEMA_VERSION: i32 = 3;

/// The steps from an empty file to each version: `MIGRATIONS[n]` takes version `n` to `n + 1`.
/// Append only: a released step never changes.
const MIGRATIONS: [&str; 3] = [SCHEMA_V1, SCHEMA_V2, SCHEMA_V3];

const SCHEMA_V1: &str = "
    CREATE TABLE meta (
        key   TEXT PRIMARY KEY NOT NULL,
        value TEXT NOT NULL
    ) STRICT;
";

// IDs are opaque and never reused. A tree's name is also its workspace name in the VCS, which is
// how a row is matched to a workspace.
const SCHEMA_V2: &str = "
    CREATE TABLE repos (
        id         TEXT PRIMARY KEY NOT NULL,
        remote     TEXT NOT NULL UNIQUE,
        dir_name   TEXT NOT NULL UNIQUE,
        created_at TEXT NOT NULL
    ) STRICT;

    CREATE TABLE trees (
        id           TEXT PRIMARY KEY NOT NULL,
        repo_id      TEXT NOT NULL REFERENCES repos (id),
        name         TEXT NOT NULL,
        role         TEXT NOT NULL CHECK (role IN ('base', 'task')),
        owner        TEXT NOT NULL,
        task         TEXT,
        lifetime     TEXT NOT NULL
            CHECK (lifetime IN ('pinned', 'task-done', 'lru') OR lifetime GLOB 'ttl:*'),
        created_at   TEXT NOT NULL,
        last_used_at TEXT NOT NULL,
        UNIQUE (repo_id, name)
    ) STRICT;
";

// The remote bookmarks mori has seen pointing into a tree's own history, and where each pointed
// when last seen. A row stays after the bookmark disappears from the remote: after a squash merge,
// that disappearance and the recorded commit are what show the tree's work landed.
const SCHEMA_V3: &str = "
    CREATE TABLE tree_bookmarks (
        tree_id   TEXT NOT NULL REFERENCES trees (id),
        remote    TEXT NOT NULL,
        bookmark  TEXT NOT NULL,
        commit_id TEXT NOT NULL,
        seen_at   TEXT NOT NULL,
        PRIMARY KEY (tree_id, remote, bookmark)
    ) STRICT;
";

/// An open mori database.
#[derive(Debug)]
pub struct Database {
    pub(crate) path: PathBuf,
    pub(crate) conn: Connection,
}

impl Database {
    /// Creates a database at `path` with permission bits `mode` and records `root` in it.
    ///
    /// The file is created with `mode` before SQLite writes anything, so it is never readable by
    /// others, even briefly. If setting it up fails, the new file is removed again.
    ///
    /// # Errors
    ///
    /// [`StoreError::AlreadyExists`] if `path` exists (it is left untouched), otherwise an I/O or
    /// SQLite error.
    pub fn create(path: &Path, root: &str, mode: u32) -> Result<Self, StoreError> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(path)
            .map_err(|source| match source.kind() {
                ErrorKind::AlreadyExists => StoreError::AlreadyExists {
                    path: path.to_path_buf(),
                },
                _ => StoreError::Io {
                    path: path.to_path_buf(),
                    source,
                },
            })?;
        let created = Self::connect(path).and_then(|mut db| {
            db.initialize(root)?;
            Ok(db)
        });
        if created.is_err() {
            // Best effort, and safe: this call created the file, so no one else's data is in it.
            let _ = std::fs::remove_file(path);
        }
        created
    }

    /// Opens an existing mori database.
    ///
    /// # Errors
    ///
    /// [`StoreError::NotMori`] if the file isn't a mori database, [`StoreError::SchemaTooNew`] if a
    /// newer mori wrote it, otherwise an I/O or SQLite error. A failed upgrade changes nothing.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let mut db = Self::connect(path)?;
        let not_mori = || StoreError::NotMori {
            path: path.to_path_buf(),
        };
        let application_id = db.pragma("application_id").map_err(|error| match error {
            StoreError::Sqlite { source, .. }
                if source.sqlite_error_code() == Some(ErrorCode::NotADatabase) =>
            {
                not_mori()
            }
            other => other,
        })?;
        if application_id != APPLICATION_ID {
            return Err(not_mori());
        }
        match db.pragma("user_version")? {
            SCHEMA_VERSION => Ok(db),
            found if found > SCHEMA_VERSION => Err(StoreError::SchemaTooNew {
                path: path.to_path_buf(),
                found,
                supported: SCHEMA_VERSION,
            }),
            // Version 0 is a file mori never finished setting up, not an old schema.
            found if found >= 1 => {
                db.upgrade(usize::try_from(found).map_err(|_| not_mori())?)?;
                Ok(db)
            }
            _ => Err(not_mori()),
        }
    }

    /// The root recorded when the database was created.
    ///
    /// # Errors
    ///
    /// [`StoreError::NotMori`] if no root is recorded, otherwise a SQLite error.
    pub fn root(&self) -> Result<PathBuf, StoreError> {
        self.conn
            .query_row("SELECT value FROM meta WHERE key = 'root'", [], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(|source| self.sqlite(source))?
            .map(PathBuf::from)
            .ok_or_else(|| StoreError::NotMori {
                path: self.path.clone(),
            })
    }

    /// Opens `path` without creating it: a missing file is an error, never a new database.
    fn connect(path: &Path) -> Result<Self, StoreError> {
        let sqlite = |source| StoreError::Sqlite {
            path: path.to_path_buf(),
            source,
        };
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(sqlite)?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(sqlite)?;
        conn.busy_timeout(BUSY_TIMEOUT).map_err(sqlite)?;
        Ok(Self {
            path: path.to_path_buf(),
            conn,
        })
    }

    /// Writes the whole schema and the root, all or nothing.
    fn initialize(&mut self, root: &str) -> Result<(), StoreError> {
        let path = self.path.clone();
        let sqlite = |source| StoreError::Sqlite {
            path: path.clone(),
            source,
        };
        let tx = self.conn.transaction().map_err(sqlite)?;
        tx.pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(sqlite)?;
        apply_migrations(&tx, 0).map_err(sqlite)?;
        tx.execute("INSERT INTO meta (key, value) VALUES ('root', ?1)", [root])
            .map_err(sqlite)?;
        tx.commit().map_err(sqlite)
    }

    /// Upgrades a database from schema version `from` to [`SCHEMA_VERSION`], all or nothing.
    fn upgrade(&mut self, from: usize) -> Result<(), StoreError> {
        let path = self.path.clone();
        let sqlite = |source| StoreError::Sqlite {
            path: path.clone(),
            source,
        };
        // Take the write lock first, then look again: another mori may have upgraded meanwhile.
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sqlite)?;
        let current: i32 = tx
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(sqlite)?;
        if usize::try_from(current).ok() == Some(from) {
            apply_migrations(&tx, from).map_err(sqlite)?;
        }
        tx.commit().map_err(sqlite)
    }

    fn pragma(&self, name: &str) -> Result<i32, StoreError> {
        self.conn
            .pragma_query_value(None, name, |row| row.get(0))
            .map_err(|source| self.sqlite(source))
    }

    pub(crate) fn sqlite(&self, source: rusqlite::Error) -> StoreError {
        StoreError::Sqlite {
            path: self.path.clone(),
            source,
        }
    }
}

/// Runs the migrations from schema version `from` on, then records the new version.
fn apply_migrations(tx: &Transaction, from: usize) -> rusqlite::Result<()> {
    for step in MIGRATIONS.iter().skip(from) {
        tx.execute_batch(step)?;
    }
    tx.pragma_update(None, "user_version", SCHEMA_VERSION)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use mori_core::error::{Code, ErrorDetails};

    use super::*;

    const ROOT: &str = "/home/acme/mori";

    /// Writes a database the way the first mori release did: schema version 1.
    fn create_v1(path: &Path) -> rusqlite::Result<()> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "application_id", APPLICATION_ID)?;
        conn.execute_batch(SCHEMA_V1)?;
        conn.execute("INSERT INTO meta (key, value) VALUES ('root', ?1)", [ROOT])?;
        conn.pragma_update(None, "user_version", 1)
    }

    fn tables(db: &Database) -> rusqlite::Result<Vec<String>> {
        let mut statement = db
            .conn
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")?;
        let names = statement.query_map([], |row| row.get(0))?;
        names.collect()
    }

    fn insert_repo(db: &Database, id: &str) -> rusqlite::Result<usize> {
        db.conn.execute(
            "INSERT INTO repos (id, remote, dir_name, created_at)
             VALUES (?1, 'github.com/acme/' || ?1, ?1, '2026-01-01T00:00:00Z')",
            [id],
        )
    }

    fn insert_tree(
        db: &Database,
        repo: &str,
        name: &str,
        role: &str,
        lifetime: &str,
    ) -> rusqlite::Result<usize> {
        db.conn.execute(
            "INSERT INTO trees (id, repo_id, name, role, owner, task, lifetime, created_at, last_used_at)
             VALUES (lower(hex(randomblob(8))), ?1, ?2, ?3, 'claude', 'fix-login', ?4,
                     '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [repo, name, role, lifetime],
        )
    }

    fn database_path() -> std::io::Result<(tempfile::TempDir, PathBuf)> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("mori.db");
        Ok((dir, path))
    }

    #[test]
    fn create_then_open_reads_the_root() {
        let (_dir, path) = database_path().unwrap();
        Database::create(&path, ROOT, 0o600).unwrap();

        let db = Database::open(&path).unwrap();
        assert_eq!(db.root().unwrap(), PathBuf::from(ROOT));
        assert_eq!(db.pragma("user_version").unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn create_uses_the_mode() {
        let (_dir, path) = database_path().unwrap();
        Database::create(&path, ROOT, 0o600).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn create_never_overwrites() {
        let (_dir, path) = database_path().unwrap();
        std::fs::write(&path, "someone else's file").unwrap();

        let error = Database::create(&path, ROOT, 0o600).unwrap_err();
        assert!(matches!(error, StoreError::AlreadyExists { .. }));
        assert_eq!(error.code(), Code::AlreadyExists);
        assert_eq!(error.reason(), "FILE_EXISTS");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "someone else's file"
        );
    }

    #[test]
    fn open_never_creates() {
        let (_dir, path) = database_path().unwrap();

        let error = Database::open(&path).unwrap_err();
        assert!(matches!(error, StoreError::Sqlite { .. }));
        assert!(!path.exists());
    }

    #[test]
    fn open_refuses_a_file_that_isnt_a_database() {
        let (_dir, path) = database_path().unwrap();
        std::fs::write(&path, "not a database, but long enough to have a header").unwrap();

        let error = Database::open(&path).unwrap_err();
        assert!(matches!(error, StoreError::NotMori { .. }));
        assert_eq!(error.reason(), "DATABASE_NOT_MORI");
    }

    #[test]
    fn open_refuses_another_programs_database() {
        let (_dir, path) = database_path().unwrap();
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE notes (body TEXT);")
            .unwrap();

        let error = Database::open(&path).unwrap_err();
        assert!(matches!(error, StoreError::NotMori { .. }));
    }

    #[test]
    fn open_refuses_a_newer_schema() {
        let (_dir, path) = database_path().unwrap();
        Database::create(&path, ROOT, 0o600).unwrap();
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .unwrap();

        let error = Database::open(&path).unwrap_err();
        assert!(matches!(
            error,
            StoreError::SchemaTooNew {
                found: 4,
                supported: 3,
                ..
            }
        ));
        assert_eq!(error.code(), Code::FailedPrecondition);
        assert_eq!(
            error.metadata(),
            vec![
                ("path", path.display().to_string()),
                ("foundVersion", "4".to_owned()),
                ("supportedVersion", "3".to_owned()),
            ]
        );
    }

    #[test]
    fn create_writes_the_latest_schema() {
        let (_dir, path) = database_path().unwrap();
        let db = Database::create(&path, ROOT, 0o600).unwrap();

        assert_eq!(
            tables(&db).unwrap(),
            ["meta", "repos", "tree_bookmarks", "trees"]
        );
    }

    #[test]
    fn open_upgrades_a_version_1_database() {
        let (_dir, path) = database_path().unwrap();
        create_v1(&path).unwrap();

        let db = Database::open(&path).unwrap();
        assert_eq!(db.pragma("user_version").unwrap(), SCHEMA_VERSION);
        assert_eq!(db.root().unwrap(), PathBuf::from(ROOT));
        assert_eq!(
            tables(&db).unwrap(),
            ["meta", "repos", "tree_bookmarks", "trees"]
        );
    }

    #[test]
    fn an_upgrade_another_mori_already_did_is_skipped() {
        let (_dir, path) = database_path().unwrap();
        create_v1(&path).unwrap();
        // Both saw version 1; this one upgrades first.
        let mut late = Database::connect(&path).unwrap();
        Database::open(&path).unwrap();

        late.upgrade(1).unwrap();
        assert_eq!(late.pragma("user_version").unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn a_failed_upgrade_changes_nothing() {
        let (_dir, path) = database_path().unwrap();
        create_v1(&path).unwrap();
        // Version 2 creates `trees` after `repos`; this makes that second step fail.
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE trees (x TEXT);")
            .unwrap();

        let error = Database::open(&path).unwrap_err();
        assert!(matches!(error, StoreError::Sqlite { .. }));
        let conn = Connection::open(&path).unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
        let repos: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name = 'repos'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(repos, 0);
    }

    #[test]
    fn trees_accept_the_known_roles_and_lifetimes() {
        let (_dir, path) = database_path().unwrap();
        let db = Database::create(&path, ROOT, 0o600).unwrap();
        insert_repo(&db, "widget").unwrap();

        for (name, role, lifetime) in [
            ("base", "base", "pinned"),
            ("a", "task", "task-done"),
            ("b", "task", "ttl:14d"),
            ("c", "task", "lru"),
        ] {
            insert_tree(&db, "widget", name, role, lifetime).unwrap();
        }
    }

    #[test]
    fn trees_refuse_bad_rows() {
        let (_dir, path) = database_path().unwrap();
        let db = Database::create(&path, ROOT, 0o600).unwrap();
        insert_repo(&db, "widget").unwrap();
        insert_tree(&db, "widget", "claude-fix-login", "task", "pinned").unwrap();

        // An unknown role or lifetime, a second tree with the same name, a repo that isn't there.
        assert!(insert_tree(&db, "widget", "x", "lead", "pinned").is_err());
        assert!(insert_tree(&db, "widget", "x", "task", "forever").is_err());
        assert!(insert_tree(&db, "widget", "claude-fix-login", "task", "pinned").is_err());
        assert!(insert_tree(&db, "gadget", "x", "task", "pinned").is_err());
    }
}

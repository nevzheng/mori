//! The state database: one SQLite file that only its owner can read.
//!
//! Its schema version is SQLite's `user_version`, and its `application_id` marks it as mori's, so
//! mori never writes to another program's database by mistake.

use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, ErrorCode, OpenFlags, OptionalExtension};

use crate::StoreError;

/// The `application_id` of a mori database: "mori" in ASCII.
const APPLICATION_ID: i32 = 0x6d6f_7269;

/// The schema version this mori writes and reads. Later versions add migrations from this one.
pub const SCHEMA_VERSION: i32 = 1;

const SCHEMA_V1: &str = "
    CREATE TABLE meta (
        key   TEXT PRIMARY KEY NOT NULL,
        value TEXT NOT NULL
    ) STRICT;
";

/// An open mori database.
#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    conn: Connection,
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
    /// newer mori wrote it, otherwise an I/O or SQLite error.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let db = Self::connect(path)?;
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
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|source| StoreError::Sqlite {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self {
            path: path.to_path_buf(),
            conn,
        })
    }

    /// Writes schema version 1 and the root, all or nothing.
    fn initialize(&mut self, root: &str) -> Result<(), StoreError> {
        let path = self.path.clone();
        let sqlite = |source| StoreError::Sqlite {
            path: path.clone(),
            source,
        };
        let tx = self.conn.transaction().map_err(sqlite)?;
        tx.pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(sqlite)?;
        tx.execute_batch(SCHEMA_V1).map_err(sqlite)?;
        tx.execute("INSERT INTO meta (key, value) VALUES ('root', ?1)", [root])
            .map_err(sqlite)?;
        tx.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(sqlite)?;
        tx.commit().map_err(sqlite)
    }

    fn pragma(&self, name: &str) -> Result<i32, StoreError> {
        self.conn
            .pragma_query_value(None, name, |row| row.get(0))
            .map_err(|source| self.sqlite(source))
    }

    fn sqlite(&self, source: rusqlite::Error) -> StoreError {
        StoreError::Sqlite {
            path: self.path.clone(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use mori_core::error::{Code, ErrorDetails};

    use super::*;

    const ROOT: &str = "/home/acme/mori";

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
                found: 2,
                supported: 1,
                ..
            }
        ));
        assert_eq!(error.code(), Code::FailedPrecondition);
        assert_eq!(
            error.metadata(),
            vec![
                ("path", path.display().to_string()),
                ("foundVersion", "2".to_owned()),
                ("supportedVersion", "1".to_owned()),
            ]
        );
    }
}

//! Errors from mori's own files (domain `store.mori`).

use std::path::{Path, PathBuf};

use mori_core::error::Code;

/// Errors from mori's own files.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// A file mori would create already exists. mori never overwrites it.
    #[error("{} already exists", path.display())]
    AlreadyExists {
        /// The file.
        path: PathBuf,
    },

    /// Something other than a directory is where mori needs a directory.
    #[error("{} is not a directory", path.display())]
    NotADirectory {
        /// The path.
        path: PathBuf,
    },

    /// The file isn't a mori database, or is missing what every mori database has.
    #[error("{} is not a mori database", path.display())]
    NotMori {
        /// The file.
        path: PathBuf,
    },

    /// The database was written by a newer mori.
    #[error(
        "{} has schema version {found}, but this mori supports up to {supported}: upgrade mori",
        path.display()
    )]
    SchemaTooNew {
        /// The database.
        path: PathBuf,
        /// The version in the file.
        found: i32,
        /// The newest version this mori supports.
        supported: i32,
    },

    /// The operating system refused or failed.
    #[error("{}: {source}", path.display())]
    Io {
        /// The path being worked on.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },

    /// SQLite failed.
    #[error("{}: {source}", path.display())]
    Sqlite {
        /// The database.
        path: PathBuf,
        /// The underlying error.
        source: rusqlite::Error,
    },
}

impl StoreError {
    /// The AIP-193 `ErrorInfo.domain`.
    pub const DOMAIN: &'static str = "store.mori";

    /// The canonical code.
    #[must_use]
    pub fn code(&self) -> Code {
        match self {
            Self::AlreadyExists { .. } => Code::AlreadyExists,
            Self::NotADirectory { .. } | Self::NotMori { .. } | Self::SchemaTooNew { .. } => {
                Code::FailedPrecondition
            }
            Self::Io { .. } | Self::Sqlite { .. } => Code::Internal,
        }
    }

    /// The AIP-193 `ErrorInfo.reason`: stable, `UPPER_SNAKE_CASE`.
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::AlreadyExists { .. } => "FILE_EXISTS",
            Self::NotADirectory { .. } => "NOT_A_DIRECTORY",
            Self::NotMori { .. } => "DATABASE_NOT_MORI",
            Self::SchemaTooNew { .. } => "DATABASE_SCHEMA_TOO_NEW",
            Self::Io { .. } => "IO_ERROR",
            Self::Sqlite { .. } => "DATABASE_ERROR",
        }
    }

    /// The AIP-193 `ErrorInfo.metadata`, with lowerCamelCase keys.
    #[must_use]
    pub fn metadata(&self) -> Vec<(&'static str, String)> {
        let mut metadata = vec![("path", self.path().display().to_string())];
        if let Self::SchemaTooNew {
            found, supported, ..
        } = self
        {
            metadata.push(("foundVersion", found.to_string()));
            metadata.push(("supportedVersion", supported.to_string()));
        }
        metadata
    }

    /// The path the error is about.
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::AlreadyExists { path }
            | Self::NotADirectory { path }
            | Self::NotMori { path }
            | Self::SchemaTooNew { path, .. }
            | Self::Io { path, .. }
            | Self::Sqlite { path, .. } => path,
        }
    }
}

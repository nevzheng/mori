//! Errors carry what Google AIP-193 needs: a canonical code, a stable machine-readable reason, a
//! domain and metadata. The edges (CLI, MCP) render them; this crate only describes them.

use std::path::PathBuf;

/// The canonical codes mori's errors use: a subset of `google.rpc.Code`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    /// The caller passed something invalid.
    InvalidArgument,
    /// Something the operation would create already exists.
    AlreadyExists,
    /// The system isn't in a state where the operation can run.
    FailedPrecondition,
    /// Something broke that the caller can't fix, e.g. an I/O error.
    Internal,
}

impl Code {
    /// The `google.rpc.Code` name, e.g. `FAILED_PRECONDITION`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::Internal => "INTERNAL",
        }
    }

    /// The `google.rpc.Code` number.
    #[must_use]
    pub fn number(self) -> i32 {
        match self {
            Self::InvalidArgument => 3,
            Self::AlreadyExists => 6,
            Self::FailedPrecondition => 9,
            Self::Internal => 13,
        }
    }
}

/// Where a root was recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootSource {
    /// `config.toml`.
    Config,
    /// The state database.
    Database,
}

impl RootSource {
    fn name(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Database => "database",
        }
    }
}

/// Errors about mori's configuration and environment (domain `config.mori`).
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigError {
    /// `HOME` isn't set (or isn't absolute), and a path needs it.
    #[error("HOME is not set to an absolute path, so mori can't find its default paths")]
    HomeNotSet,

    /// A mori override variable holds a relative path.
    #[error("{var} must be an absolute path, got {value:?}")]
    NotAbsolute {
        /// The variable, e.g. `MORI_ROOT`.
        var: &'static str,
        /// Its value.
        value: PathBuf,
    },

    /// A path mori would use isn't valid UTF-8.
    #[error("{path:?} is not valid UTF-8; mori needs UTF-8 paths")]
    NotUtf8 {
        /// The path.
        path: PathBuf,
    },

    /// `config.toml` doesn't parse, or has an unsupported schema.
    #[error("{path:?} is invalid: {message}")]
    ConfigInvalid {
        /// The config file.
        path: PathBuf,
        /// What's wrong.
        message: String,
    },

    /// The root on record differs from the effective root: a misconfiguration.
    #[error(
        "misconfiguration: mori's root is recorded as {recorded:?} in the {} but is now {effective:?}. \
         Moving a root needs a migration, which isn't defined yet",
        recorded_in.name()
    )]
    RootMismatch {
        /// The root on record.
        recorded: PathBuf,
        /// The root this run would use.
        effective: PathBuf,
        /// Where the recorded root came from. (Not `source`: thiserror reserves that name.)
        recorded_in: RootSource,
    },
}

impl ConfigError {
    /// The AIP-193 `ErrorInfo.domain`.
    pub const DOMAIN: &'static str = "config.mori";

    /// The canonical code.
    #[must_use]
    pub fn code(&self) -> Code {
        match self {
            Self::NotAbsolute { .. } | Self::NotUtf8 { .. } | Self::ConfigInvalid { .. } => {
                Code::InvalidArgument
            }
            Self::HomeNotSet | Self::RootMismatch { .. } => Code::FailedPrecondition,
        }
    }

    /// The AIP-193 `ErrorInfo.reason`: stable, `UPPER_SNAKE_CASE`.
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::HomeNotSet => "HOME_NOT_SET",
            Self::NotAbsolute { .. } => "PATH_NOT_ABSOLUTE",
            Self::NotUtf8 { .. } => "PATH_NOT_UTF8",
            Self::ConfigInvalid { .. } => "CONFIG_INVALID",
            Self::RootMismatch { .. } => "ROOT_MISMATCH",
        }
    }

    /// The AIP-193 `ErrorInfo.metadata`, with lowerCamelCase keys.
    #[must_use]
    pub fn metadata(&self) -> Vec<(&'static str, String)> {
        let path = |p: &PathBuf| p.display().to_string();
        match self {
            Self::HomeNotSet => vec![],
            Self::NotAbsolute { var, value } => {
                vec![("variable", (*var).to_owned()), ("value", path(value))]
            }
            Self::NotUtf8 { path: p } | Self::ConfigInvalid { path: p, .. } => {
                vec![("path", path(p))]
            }
            Self::RootMismatch {
                recorded,
                effective,
                recorded_in,
            } => vec![
                ("recordedRoot", path(recorded)),
                ("effectiveRoot", path(effective)),
                ("recordedIn", recorded_in.name().to_owned()),
            ],
        }
    }
}

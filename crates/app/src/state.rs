//! mori's own state, as every command after `init` needs it: the paths, the database and the
//! `[trees]` policy from `config.toml`.

use mori_core::config::Config;
use mori_core::error::{ConfigError, ErrorDetails, RootSource};
use mori_core::paths::Paths;
use mori_core::tree::TreePolicy;
use mori_store::StoreError;
use mori_store::database::Database;

use crate::Host;

/// The paths for this run, from the host's environment.
pub fn paths(host: &Host) -> Result<Paths, Box<dyn ErrorDetails>> {
    Paths::resolve(&host.env).map_err(boxed)
}

/// Opens the database `mori init` made, checking that it records this run's root.
pub fn open_database(paths: &Paths) -> Result<Database, Box<dyn ErrorDetails>> {
    if paths.database.symlink_metadata().is_err() {
        return Err(boxed(ConfigError::NotInitialized {
            database: paths.database.clone(),
        }));
    }
    let db = Database::open(&paths.database).map_err(boxed)?;
    let recorded = db.root().map_err(boxed)?;
    if recorded != paths.root {
        return Err(boxed(ConfigError::RootMismatch {
            recorded,
            effective: paths.root.clone(),
            recorded_in: RootSource::Database,
        }));
    }
    Ok(db)
}

/// The `[trees]` policy from `config.toml`: every key has a default.
pub fn tree_policy(paths: &Paths) -> Result<TreePolicy, Box<dyn ErrorDetails>> {
    let text = std::fs::read_to_string(&paths.config_file).map_err(|source| {
        boxed(StoreError::Io {
            path: paths.config_file.clone(),
            source,
        })
    })?;
    Ok(Config::parse(&text, &paths.config_file)
        .map_err(boxed)?
        .trees)
}

/// Boxes an error for the edges to render.
pub fn boxed(error: impl ErrorDetails + 'static) -> Box<dyn ErrorDetails> {
    Box::new(error)
}

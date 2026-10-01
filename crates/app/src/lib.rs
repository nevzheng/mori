//! mori's commands as flows: each observes through the adapters, asks `mori-core` for a plan,
//! and carries it out.
//!
//! A flow gets everything from outside through an [`App`]: the environment and clock as plain
//! data, and the VCS and code host as traits. The binary builds one from the real process; a test
//! builds one with fakes.

mod bazel;
mod cache;
pub mod clone;
mod disk;
pub mod doctor;
pub mod gc;
mod gc_apply;
pub mod init;
mod landing;
pub mod ls;
pub mod place;
pub mod restore;
pub mod routed;
pub mod skills;
mod state;
pub mod tree;
pub mod tree_remove;
pub mod tree_set;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::time::{SystemTime, UNIX_EPOCH};

use mori_core::error::ErrorDetails;
use mori_core::paths::Env;
use std::path::Path;

use mori_core::vcs::{Forge, Vcs, VcsKind};

/// What a command reads from the process: environment variables and the clock.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Host {
    /// The variables that locate mori's root and state.
    pub env: Env,
    /// `USER`: the default owner of trees and clones.
    pub user: Option<String>,
    /// `MORI_AGENT`: the default owner of trees, ahead of `USER`.
    pub agent: Option<String>,
    /// Now, in seconds since the Unix epoch.
    pub now: u64,
    /// The variables the cache checks read ([`mori_core::cache::ENV_VARS`]) that are set and
    /// not empty.
    pub cache_vars: BTreeMap<String, OsString>,
}

impl Host {
    /// Reads the real process environment and clock.
    #[must_use]
    pub fn from_process() -> Self {
        let var = |name: &str| std::env::var(name).ok();
        Self {
            env: Env::from_vars(|name| std::env::var_os(name)),
            user: var("USER"),
            agent: var("MORI_AGENT").filter(|agent| !agent.trim().is_empty()),
            now: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cache_vars: mori_core::cache::ENV_VARS
                .iter()
                .filter_map(|name| {
                    let value = std::env::var_os(name).filter(|value| !value.is_empty())?;
                    Some(((*name).to_owned(), value))
                })
                .collect(),
        }
    }
}

/// A VCS the flows can use: its errors carry codes and reasons, and it can make a clone of
/// either kind ([`routed::Routed`] in the binary).
pub trait Backend: Vcs<Error: ErrorDetails + 'static> {
    /// Clones `url` into `path` as a `kind` clone; `colocate` applies to jj.
    ///
    /// # Errors
    ///
    /// As [`Vcs::clone_repo`].
    fn clone_as(
        &self,
        kind: VcsKind,
        url: &str,
        path: &Path,
        colocate: bool,
    ) -> Result<(), Self::Error>;

    /// Which of `needed` can't be run on this machine. By default, none.
    fn missing_tools(&self, _needed: &[VcsKind]) -> Vec<VcsKind> {
        Vec::new()
    }
}

/// The API's name for `kind`.
fn api_vcs(kind: VcsKind) -> mori_api::v1alpha1::Vcs {
    match kind {
        VcsKind::Jj => mori_api::v1alpha1::Vcs::Jj,
        VcsKind::Git => mori_api::v1alpha1::Vcs::Git,
    }
}

/// Everything a command gets from outside.
pub struct App<V, F> {
    /// The environment and clock.
    pub host: Host,
    /// The VCS clones and trees are made with.
    pub vcs: V,
    /// The code host asked about pull requests, unless a command runs offline.
    pub forge: F,
}

impl<V: Backend, F: Forge> App<V, F> {
    /// An app from its parts.
    pub fn new(host: Host, vcs: V, forge: F) -> Self {
        Self { host, vcs, forge }
    }
}

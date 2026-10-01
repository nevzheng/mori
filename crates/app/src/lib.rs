//! mori's commands as flows: each observes through the adapters, asks `mori-core` for a plan,
//! and carries it out.
//!
//! A flow gets everything from outside through an [`App`]: the environment and clock as plain
//! data, and the VCS and code host as traits. The binary builds one from the real process; a test
//! builds one with fakes.

pub mod clone;
pub mod gc;
mod gc_apply;
pub mod init;
mod landing;
pub mod ls;
pub mod restore;
pub mod skills;
mod state;
pub mod tree;
pub mod tree_remove;

use std::time::{SystemTime, UNIX_EPOCH};

use mori_core::error::ErrorDetails;
use mori_core::paths::Env;
use mori_core::vcs::{Forge, Vcs};

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
        }
    }
}

/// A VCS whose errors the flows can report.
pub trait Backend: Vcs<Error: ErrorDetails + 'static> {}

impl<T: Vcs<Error: ErrorDetails + 'static>> Backend for T {}

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

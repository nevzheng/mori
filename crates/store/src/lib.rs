//! Everything mori keeps on this machine: its directories, `config.toml` and the state database.
//! This is an adapter: it carries out what `mori-core` decides, and does no deciding itself.

pub mod database;
mod error;
pub mod init;

pub use error::StoreError;

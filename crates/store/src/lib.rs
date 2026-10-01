//! Everything mori keeps on this machine: its directories, `config.toml` and the state database.
//! This is an adapter: it carries out what `mori-core` decides, and does no deciding itself.

pub mod database;
pub mod disk;
mod error;
pub mod gc;
pub mod init;
pub mod records;
pub mod skills;

pub use error::StoreError;

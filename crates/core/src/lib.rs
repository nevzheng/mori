//! mori's domain model and rules.
//!
//! Pure and synchronous: no I/O. Operations follow observe → plan → execute. An adapter gathers
//! the facts an operation needs as plain data, this crate turns them into a plan or a refusal,
//! and an adapter carries the plan out.

pub mod cache;
pub mod clone;
pub mod config;
pub mod disk;
pub mod doctor;
pub mod error;
pub mod forest;
pub mod gc;
pub mod init;
pub mod paths;
pub mod skills;
pub mod tree;
pub mod tree_create;
pub mod tree_remove;
pub mod vcs;

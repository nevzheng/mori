//! mori's domain model and rules.
//!
//! Pure and synchronous: no I/O. Operations follow observe → plan → execute. An adapter gathers
//! the facts an operation needs as plain data, this crate turns them into a plan or a refusal,
//! and an adapter carries the plan out.

pub mod config;
pub mod error;
pub mod init;
pub mod paths;

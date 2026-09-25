//! mori's domain model and rules: trees, refs, owners, lifetimes, cleanup.
//!
//! Pure and synchronous. Every side effect sits behind a trait (a port) defined here and
//! implemented by an adapter crate.

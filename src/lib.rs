//! Library root. Splitting this out from `main.rs` (which becomes a thin
//! binary entry point) is what lets `cargo test` actually run doctests -
//! they only execute against a library target, not a bare binary crate.

pub mod core;
pub mod db;
pub mod error;
pub mod models;
pub mod routers;
pub mod schemas;

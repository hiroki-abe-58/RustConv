//! Core module - conversion engine, query, validation
//!
//! This module includes:
//! - converter.rs: Cross-format conversion engine
//!
//! Future phases will add:
//! - query.rs: JSONPath and jq-compatible queries
//! - validator.rs: Schema validation
//! - differ.rs: Diff calculation
//! - merger.rs: Merge logic

pub mod converter;

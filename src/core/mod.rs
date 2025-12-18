//! Core module - conversion engine, query, validation
//!
//! This module includes:
//! - converter.rs: Cross-format conversion engine
//! - query.rs: JSONPath and data transformation queries
//!
//! Future phases will add:
//! - validator.rs: Schema validation
//! - differ.rs: Diff calculation
//! - merger.rs: Merge logic

pub mod converter;
pub mod query;

//! Core module - conversion engine, query, validation
//!
//! This module includes:
//! - converter.rs: Cross-format conversion engine
//! - query.rs: JSONPath and data transformation queries
//! - validator.rs: Schema validation and linting
//! - differ.rs: Diff calculation
//! - schema.rs: JSON Schema generation
//!
//! Future phases will add:
//! - merger.rs: Merge logic

pub mod converter;
pub mod differ;
pub mod query;
pub mod schema;
pub mod validator;

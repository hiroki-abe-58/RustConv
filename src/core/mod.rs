//! Core module - conversion engine, query, validation
//!
//! This module includes:
//! - converter.rs: Cross-format conversion engine
//! - query.rs: JSONPath and data transformation queries
//! - validator.rs: Schema validation and linting
//! - differ.rs: Diff calculation
//! - schema.rs: JSON Schema generation
//! - merger.rs: Merge logic
//! - patcher.rs: JSON Patch (RFC 6902)
//! - template.rs: Template variable substitution
//! - batch.rs: Batch processing

pub mod batch;
pub mod converter;
pub mod differ;
pub mod merger;
pub mod patcher;
pub mod query;
pub mod schema;
pub mod template;
pub mod validator;

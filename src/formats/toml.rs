//! TOML format handling

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use toml::Value;

/// Read input from file or stdin
pub fn read_input(path: Option<&Path>) -> Result<String> {
    match path {
        Some(p) => {
            fs::read_to_string(p).with_context(|| format!("Failed to read file: {}", p.display()))
        }
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            Ok(buffer)
        }
    }
}

/// Parse TOML string into Value
pub fn parse(content: &str) -> Result<Value> {
    content.parse::<Value>().context("Failed to parse TOML")
}

/// Convert Value to pretty-printed TOML string
pub fn to_pretty(value: &Value) -> Result<String> {
    toml::to_string_pretty(value).context("Failed to serialize TOML")
}

/// Convert Value to compact TOML string
pub fn to_compact(value: &Value) -> Result<String> {
    toml::to_string(value).context("Failed to serialize TOML")
}

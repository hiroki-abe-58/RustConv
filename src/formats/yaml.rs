//! YAML format handling

use anyhow::{Context, Result};
use serde_yaml::Value;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

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

/// Parse YAML string into Value
pub fn parse(content: &str) -> Result<Value> {
    serde_yaml::from_str(content).context("Failed to parse YAML")
}

/// Convert Value to pretty-printed YAML string
pub fn to_pretty(value: &Value) -> Result<String> {
    serde_yaml::to_string(value).context("Failed to serialize YAML")
}

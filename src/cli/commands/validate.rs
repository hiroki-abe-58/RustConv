//! Validate subcommand implementation

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cli::args::ValidateArgs;
use crate::cli::output::write_output;
use crate::core::validator;
use crate::formats::detect::{detect, Format};

/// Execute the validate subcommand
pub fn execute(args: ValidateArgs) -> Result<()> {
    // Read input
    let content = read_input(args.input.as_deref())?;

    // Detect format
    let format = if let Some(ref fmt) = args.format {
        parse_format(fmt)?
    } else {
        detect(args.input.as_deref(), &content)
            .context("Could not detect format. Use --format to specify.")?
    };

    let result = if let Some(ref schema_path) = args.schema {
        // Validate against JSON Schema
        let schema_content = fs::read_to_string(schema_path)
            .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;
        let schema: serde_json::Value = serde_json::from_str(&schema_content)
            .context("Failed to parse schema as JSON")?;
        let data: serde_json::Value = parse_to_json(&content, format)?;
        validator::validate_json_schema(&data, &schema)?
    } else {
        // Lint the format
        match format {
            Format::Json => validator::lint_json(&content)?,
            Format::Yaml => validator::lint_yaml(&content)?,
            Format::Toml => validator::lint_toml(&content)?,
            Format::Csv => validator::validate_csv(&content, !args.no_headers)?,
            Format::Xml => {
                // For XML, just validate it can be parsed
                crate::formats::xml::validate(&content)?;
                let mut result = validator::ValidationResult::new();
                result.valid = true;
                result
            }
        }
    };

    let output = result.format_output();
    write_output(&output)?;

    if !result.valid {
        std::process::exit(1);
    }

    Ok(())
}

fn read_input(path: Option<&Path>) -> Result<String> {
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

fn parse_format(s: &str) -> Result<Format> {
    match s.to_lowercase().as_str() {
        "json" => Ok(Format::Json),
        "yaml" | "yml" => Ok(Format::Yaml),
        "toml" => Ok(Format::Toml),
        "csv" => Ok(Format::Csv),
        "xml" => Ok(Format::Xml),
        _ => anyhow::bail!("Unknown format: {}", s),
    }
}

fn parse_to_json(content: &str, format: Format) -> Result<serde_json::Value> {
    match format {
        Format::Json => serde_json::from_str(content).context("Failed to parse JSON"),
        Format::Yaml => {
            let yaml: serde_yaml::Value =
                serde_yaml::from_str(content).context("Failed to parse YAML")?;
            let json_str = serde_json::to_string(&yaml)?;
            serde_json::from_str(&json_str).context("Failed to convert YAML to JSON")
        }
        _ => anyhow::bail!("Schema validation only supports JSON and YAML"),
    }
}


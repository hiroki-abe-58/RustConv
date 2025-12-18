//! Query subcommand implementation

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cli::args::QueryArgs;
use crate::cli::output::write_output;
use crate::core::query;
use crate::formats::detect::{detect, Format};
use crate::formats::{json as json_format, yaml as yaml_format};
use crate::utils::highlight;

/// Execute the query subcommand
pub fn execute(args: QueryArgs) -> Result<()> {
    // Read input
    let content = read_input(args.input.as_deref())?;

    // Detect format and parse to JSON
    let format = detect(args.input.as_deref(), &content).unwrap_or(Format::Json);

    let mut value = parse_to_json(&content, format)?;

    // Apply JSONPath query if provided
    if let Some(ref path) = args.query {
        value = query::jsonpath_query(&value, path)?;
    }

    // Apply transformations
    if args.keys {
        value = query::extract_keys(&value, args.recursive);
    }

    if args.values {
        value = query::extract_values(&value, args.recursive);
    }

    if args.flatten {
        let separator = args.separator.as_deref().unwrap_or(".");
        value = query::flatten(&value, separator);
    }

    if args.sort_keys {
        value = query::sort_keys(&value);
    }

    if let Some(ref expr) = args.filter {
        value = query::filter_array(&value, expr)?;
    }

    if let Some(ref fields) = args.select {
        let field_list: Vec<String> = fields.split(',').map(|s| s.trim().to_string()).collect();
        value = query::select_fields(&value, &field_list)?;
    }

    if args.unique {
        value = query::unique(&value)?;
    }

    if args.count {
        value = query::count(&value);
    }

    if args.reverse {
        value = query::reverse(&value)?;
    }

    if let Some(n) = args.first {
        value = query::first(&value, n)?;
    }

    if let Some(n) = args.last {
        value = query::last(&value, n)?;
    }

    // Output
    let output = if args.compact {
        serde_json::to_string(&value)?
    } else {
        serde_json::to_string_pretty(&value)?
    };

    let highlighted = if args.raw {
        output
    } else {
        highlight::highlight_json(&output)
    };

    write_output(&highlighted)?;

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

fn parse_to_json(content: &str, format: Format) -> Result<serde_json::Value> {
    match format {
        Format::Json => json_format::parse(content)
            .map(|v| serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap()),
        Format::Yaml => {
            let yaml_value = yaml_format::parse(content)?;
            let json_str = serde_json::to_string(&yaml_value)?;
            serde_json::from_str(&json_str).context("Failed to convert YAML to JSON")
        }
        _ => {
            // For other formats, try JSON first, then YAML
            if let Ok(v) = json_format::parse(content) {
                Ok(serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap())
            } else {
                let yaml_value = yaml_format::parse(content)?;
                let json_str = serde_json::to_string(&yaml_value)?;
                serde_json::from_str(&json_str).context("Failed to parse input")
            }
        }
    }
}

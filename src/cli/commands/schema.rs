//! Schema subcommand implementation

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cli::args::SchemaArgs;
use crate::cli::output::write_output;
use crate::core::schema;
use crate::formats::detect::{detect, Format};
use crate::utils::highlight;

/// Execute the schema subcommand
pub fn execute(args: SchemaArgs) -> Result<()> {
    // Read input
    let content = read_input(args.input.as_deref())?;

    // Detect format and parse to JSON
    let format = detect(args.input.as_deref(), &content).unwrap_or(Format::Json);
    let value = parse_to_json(&content, format)?;

    // Generate schema
    let json_schema = schema::generate_schema(&value);

    // Output based on format
    let output = if args.typescript {
        let name = args
            .name
            .as_deref()
            .unwrap_or_else(|| {
                args.input
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str())
                    .unwrap_or("Data")
            });
        // Capitalize first letter
        let name = capitalize_first(name);
        schema::schema_to_typescript(&json_schema, &name)
    } else {
        let json_str = serde_json::to_string_pretty(&json_schema)?;
        if args.raw {
            json_str
        } else {
            highlight::highlight_json(&json_str)
        }
    };

    // Write output
    if let Some(ref output_path) = args.output {
        fs::write(output_path, &output)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
    } else {
        write_output(&output)?;
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

fn parse_to_json(content: &str, format: Format) -> Result<serde_json::Value> {
    match format {
        Format::Json => serde_json::from_str(content).context("Failed to parse JSON"),
        Format::Yaml => {
            let yaml: serde_yaml::Value =
                serde_yaml::from_str(content).context("Failed to parse YAML")?;
            let json_str = serde_json::to_string(&yaml)?;
            serde_json::from_str(&json_str).context("Failed to convert to JSON")
        }
        Format::Csv => {
            let data = crate::formats::csv::parse(content, true)?;
            let headers = data.headers.as_ref().context("CSV must have headers")?;

            let mut records = Vec::new();
            for row in &data.rows {
                let mut obj = serde_json::Map::new();
                for (i, cell) in row.iter().enumerate() {
                    let key = headers.get(i).cloned().unwrap_or_else(|| format!("col{}", i));
                    // Try to infer type
                    let value = if let Ok(n) = cell.parse::<i64>() {
                        serde_json::Value::Number(n.into())
                    } else if let Ok(f) = cell.parse::<f64>() {
                        serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::String(cell.clone()))
                    } else if cell == "true" || cell == "false" {
                        serde_json::Value::Bool(cell == "true")
                    } else {
                        serde_json::Value::String(cell.clone())
                    };
                    obj.insert(key, value);
                }
                records.push(serde_json::Value::Object(obj));
            }
            Ok(serde_json::Value::Array(records))
        }
        _ => anyhow::bail!("Schema generation supports JSON, YAML, and CSV"),
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}


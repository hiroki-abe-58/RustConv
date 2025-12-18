//! Merge subcommand implementation

use anyhow::{Context, Result};
use std::fs;

use crate::cli::args::MergeArgs;
use crate::cli::output::write_output;
use crate::core::converter;
use crate::core::merger::{self, MergeStrategy};
use crate::formats::detect::{detect, Format};
use crate::utils::highlight;

/// Execute the merge subcommand
pub fn execute(args: MergeArgs) -> Result<()> {
    // Read all input files
    let mut values = Vec::new();

    for input_path in &args.files {
        let content = fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read file: {}", input_path.display()))?;

        let format = detect(Some(input_path.as_path()), &content)
            .with_context(|| format!("Could not detect format of: {}", input_path.display()))?;

        // Convert to JSON for merging
        let json_str = converter::convert(&content, format, Format::Json)?;
        let value: serde_json::Value = serde_json::from_str(&json_str)?;
        values.push(value);
    }

    // Determine merge strategy
    let strategy = match args.strategy.as_deref() {
        Some("shallow") => MergeStrategy::Shallow,
        Some("concat") => MergeStrategy::ConcatArrays,
        Some("union") => MergeStrategy::UnionArrays,
        Some("deep") | None => MergeStrategy::Deep,
        Some(s) => anyhow::bail!("Unknown merge strategy: {}. Use: deep, shallow, concat, union", s),
    };

    // Merge all values
    let merged = merger::merge_all(&values, strategy)?;

    // Determine output format
    let output_format = if let Some(ref fmt) = args.format {
        parse_format(fmt)?
    } else if let Some(ref output_path) = args.output {
        detect(Some(output_path.as_path()), "").unwrap_or(Format::Json)
    } else {
        Format::Json
    };

    // Convert to output format
    let output = format_output(&merged, output_format)?;

    // Write output
    if let Some(ref output_path) = args.output {
        fs::write(output_path, &output)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        if !args.quiet {
            eprintln!("Merged {} files -> {}", args.files.len(), output_path.display());
        }
    } else {
        let highlighted = match output_format {
            Format::Json => highlight::highlight_json(&output),
            Format::Yaml => highlight::highlight_yaml(&output),
            Format::Toml => highlight::highlight_toml(&output),
            _ => output.clone(),
        };
        write_output(&highlighted)?;
    }

    Ok(())
}

fn parse_format(s: &str) -> Result<Format> {
    match s.to_lowercase().as_str() {
        "json" => Ok(Format::Json),
        "yaml" | "yml" => Ok(Format::Yaml),
        "toml" => Ok(Format::Toml),
        _ => anyhow::bail!("Unsupported output format: {}. Use: json, yaml, toml", s),
    }
}

fn format_output(value: &serde_json::Value, format: Format) -> Result<String> {
    match format {
        Format::Json => serde_json::to_string_pretty(value).context("Failed to serialize JSON"),
        Format::Yaml => serde_yaml::to_string(value).context("Failed to serialize YAML"),
        Format::Toml => toml::to_string_pretty(value).context("Failed to serialize TOML"),
        _ => anyhow::bail!("Unsupported output format for merge"),
    }
}


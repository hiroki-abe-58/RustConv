//! Convert subcommand implementation

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::cli::args::ConvertArgs;
use crate::cli::output::write_output;
use crate::core::converter;
use crate::formats::detect::{detect, Format};
use crate::utils::highlight;

/// Execute the convert subcommand
pub fn execute(args: ConvertArgs) -> Result<()> {
    // Read input
    let content = read_input(args.input.as_deref())?;

    // Detect source format
    let from_format = if let Some(ref from) = args.from {
        parse_format(from)?
    } else {
        detect(args.input.as_deref(), &content)
            .context("Could not detect source format. Use --from to specify.")?
    };

    // Parse target formats
    let to_formats = parse_target_formats(&args.to)?;

    if to_formats.is_empty() {
        bail!("No target format specified. Use --to to specify output format(s).");
    }

    // Perform conversion(s)
    for to_format in &to_formats {
        let result = converter::convert(&content, from_format, *to_format)?;

        if let Some(ref output_path) = args.output {
            // Write to file
            let output_file = if to_formats.len() > 1 {
                // Multiple outputs: add extension
                let stem = output_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let parent = output_path.parent().unwrap_or(Path::new("."));
                parent.join(format!("{}.{}", stem, to_format.as_str()))
            } else {
                output_path.clone()
            };

            fs::write(&output_file, &result)
                .with_context(|| format!("Failed to write to {}", output_file.display()))?;

            if !args.quiet {
                eprintln!(
                    "{} {} -> {}",
                    "Converted:".green(),
                    from_format.as_str().cyan(),
                    output_file.display().to_string().cyan()
                );
            }
        } else {
            // Output to stdout
            if to_formats.len() > 1 && !args.quiet {
                eprintln!(
                    "{} {}",
                    "--- Output format:".dimmed(),
                    to_format.as_str().cyan()
                );
            }

            let highlighted = highlight_output(&result, *to_format);
            write_output(&highlighted)?;

            if to_formats.len() > 1 {
                println!(); // Separator between outputs
            }
        }
    }

    Ok(())
}

fn read_input(path: Option<&Path>) -> Result<String> {
    match path {
        Some(p) => {
            fs::read_to_string(p).with_context(|| format!("Failed to read file: {}", p.display()))
        }
        None => {
            use std::io::Read;
            let mut buffer = String::new();
            std::io::stdin()
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
        _ => bail!(
            "Unknown format: {}. Supported: json, yaml, toml, csv, xml",
            s
        ),
    }
}

fn parse_target_formats(to: &str) -> Result<Vec<Format>> {
    let mut formats = Vec::new();

    for part in to.split(',') {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            formats.push(parse_format(trimmed)?);
        }
    }

    Ok(formats)
}

fn highlight_output(content: &str, format: Format) -> String {
    match format {
        Format::Json => highlight::highlight_json(content),
        Format::Yaml => highlight::highlight_yaml(content),
        Format::Toml => highlight::highlight_toml(content),
        Format::Csv => highlight::highlight_csv(content, true),
        Format::Xml => highlight::highlight_xml(content),
    }
}

/// Get output file path with appropriate extension
pub fn get_output_path(
    input: Option<&Path>,
    format: Format,
    output: Option<&Path>,
) -> Option<std::path::PathBuf> {
    if let Some(out) = output {
        return Some(out.to_path_buf());
    }

    input.map(|p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let parent = p.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}.{}", stem, format.as_str()))
    })
}

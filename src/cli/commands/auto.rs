//! Auto-detect format subcommand implementation

use anyhow::{bail, Result};
use colored::Colorize;

use crate::cli::args::AutoArgs;
use crate::cli::output::write_output;
use crate::formats::detect::{detect, Format};
use crate::formats::{
    csv as csv_format, json as json_format, toml as toml_format, xml as xml_format,
    yaml as yaml_format,
};
use crate::utils::highlight;

/// Execute the auto subcommand
pub fn execute(args: AutoArgs) -> Result<()> {
    // Read content first
    let content = match &args.input {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            buf
        }
    };

    // Detect format
    let format = detect(args.input.as_deref(), &content);

    match format {
        Some(Format::Json) => {
            if !args.quiet {
                eprintln!("{} {}", "Detected format:".dimmed(), "JSON".cyan());
            }
            let value = json_format::parse(&content)?;
            let output = json_format::to_pretty(&value)?;
            let highlighted = highlight::highlight_json(&output);
            write_output(&highlighted)?;
        }
        Some(Format::Yaml) => {
            if !args.quiet {
                eprintln!("{} {}", "Detected format:".dimmed(), "YAML".cyan());
            }
            let value = yaml_format::parse(&content)?;
            let output = yaml_format::to_pretty(&value)?;
            let highlighted = highlight::highlight_yaml(&output);
            write_output(&highlighted)?;
        }
        Some(Format::Toml) => {
            if !args.quiet {
                eprintln!("{} {}", "Detected format:".dimmed(), "TOML".cyan());
            }
            let value = toml_format::parse(&content)?;
            let output = toml_format::to_pretty(&value)?;
            let highlighted = highlight::highlight_toml(&output);
            write_output(&highlighted)?;
        }
        Some(Format::Csv) => {
            if !args.quiet {
                eprintln!("{} {}", "Detected format:".dimmed(), "CSV".cyan());
            }
            let data = csv_format::parse(&content, true)?;
            let output = csv_format::to_table(&data)?;
            let highlighted = highlight::highlight_csv(&output, false);
            write_output(&highlighted)?;
        }
        Some(Format::Xml) => {
            if !args.quiet {
                eprintln!("{} {}", "Detected format:".dimmed(), "XML".cyan());
            }
            xml_format::validate(&content)?;
            let output = xml_format::to_pretty(&content)?;
            let highlighted = highlight::highlight_xml(&output);
            write_output(&highlighted)?;
        }
        None => {
            bail!("Could not detect format. Please specify the format explicitly using a subcommand (json, yaml, toml, csv, xml).");
        }
    }

    Ok(())
}

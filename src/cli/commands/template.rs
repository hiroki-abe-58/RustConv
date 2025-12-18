//! Template subcommand implementation

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cli::args::TemplateArgs;
use crate::cli::output::write_output;
use crate::core::template::{self, TemplateOptions};
use crate::formats::detect::{detect, Format};
use crate::utils::highlight;

/// Execute the template subcommand
pub fn execute(args: TemplateArgs) -> Result<()> {
    // Read template
    let template_content = read_input(args.template.as_deref())?;

    // Detect template format
    let template_format = detect(args.template.as_deref(), &template_content)
        .unwrap_or(Format::Json);

    // Parse template as JSON value for processing
    let template_value: serde_json::Value = match template_format {
        Format::Json => serde_json::from_str(&template_content)
            .context("Template must be valid JSON")?,
        Format::Yaml => {
            let yaml: serde_yaml::Value = serde_yaml::from_str(&template_content)
                .context("Template must be valid YAML")?;
            serde_json::to_value(yaml)?
        }
        _ => anyhow::bail!("Template must be JSON or YAML"),
    };

    // Load variables
    let mut vars = serde_json::Map::new();

    // Add environment variables if requested
    if args.env {
        if let serde_json::Value::Object(env_vars) = template::env_to_json() {
            for (k, v) in env_vars {
                vars.insert(k, v);
            }
        }
    }

    // Load variables from file
    if let Some(ref vars_path) = args.vars {
        let vars_content = fs::read_to_string(vars_path)
            .with_context(|| format!("Failed to read vars file: {}", vars_path.display()))?;
        let vars_format = detect(Some(vars_path.as_path()), &vars_content)
            .context("Could not detect vars file format")?;

        let file_vars: serde_json::Value = match vars_format {
            Format::Json => serde_json::from_str(&vars_content)?,
            Format::Yaml => {
                let yaml: serde_yaml::Value = serde_yaml::from_str(&vars_content)?;
                serde_json::to_value(yaml)?
            }
            _ => anyhow::bail!("Variables file must be JSON or YAML"),
        };

        if let serde_json::Value::Object(obj) = file_vars {
            for (k, v) in obj {
                vars.insert(k, v);
            }
        }
    }

    // Add inline variables
    for var_str in &args.set {
        let parts: Vec<&str> = var_str.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            // Try to parse as JSON, otherwise treat as string
            let json_value: serde_json::Value = serde_json::from_str(value)
                .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
            vars.insert(key.to_string(), json_value);
        } else {
            anyhow::bail!("Invalid variable format: '{}'. Use: key=value", var_str);
        }
    }

    let vars_value = serde_json::Value::Object(vars);

    // Configure template options
    let options = TemplateOptions {
        strict: args.strict,
        ..Default::default()
    };

    // Validate template if requested
    if args.validate {
        let missing = template::validate_template(&template_value, &vars_value, &options)?;
        if missing.is_empty() {
            eprintln!("Template validation passed. All variables are defined.");
            return Ok(());
        } else {
            eprintln!("Missing variables:");
            for var in &missing {
                eprintln!("  - {}", var);
            }
            std::process::exit(1);
        }
    }

    // Render template
    let rendered = template::render_value(&template_value, &vars_value, &options)?;

    // Format output
    let output_format = if let Some(ref fmt) = args.format {
        parse_format(fmt)?
    } else if let Some(ref output_path) = args.output {
        detect(Some(output_path.as_path()), "").unwrap_or(template_format)
    } else {
        template_format
    };

    let output = format_output(&rendered, output_format)?;

    // Write output
    if let Some(ref output_path) = args.output {
        fs::write(output_path, &output)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        if !args.quiet {
            eprintln!("Rendered template written to {}", output_path.display());
        }
    } else {
        let highlighted = if args.raw {
            output
        } else {
            match output_format {
                Format::Json => highlight::highlight_json(&output),
                Format::Yaml => highlight::highlight_yaml(&output),
                _ => output.clone(),
            }
        };
        write_output(&highlighted)?;
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
        _ => anyhow::bail!("Unsupported output format: {}. Use: json, yaml", s),
    }
}

fn format_output(value: &serde_json::Value, format: Format) -> Result<String> {
    match format {
        Format::Json => serde_json::to_string_pretty(value).context("Failed to serialize JSON"),
        Format::Yaml => serde_yaml::to_string(value).context("Failed to serialize YAML"),
        _ => serde_json::to_string_pretty(value).context("Failed to serialize"),
    }
}


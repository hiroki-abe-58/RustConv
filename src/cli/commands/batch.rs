//! Batch subcommand implementation

use anyhow::{Context, Result};
use std::fs;

use crate::cli::args::BatchArgs;
use crate::cli::output::write_output;
use crate::core::batch::{self, BatchConfig};
use crate::formats::detect::detect;

/// Execute the batch subcommand
pub fn execute(args: BatchArgs) -> Result<()> {
    // Read config file
    let config_content = fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config file: {}", args.config.display()))?;

    // Detect config format
    let config_format = detect(Some(args.config.as_path()), &config_content)
        .context("Could not detect config file format")?;

    // Parse config
    let mut config: BatchConfig = batch::parse_config(&config_content, config_format)?;

    // Override continue_on_error if specified
    if args.continue_on_error {
        config.continue_on_error = true;
    }

    // Merge variables from command line
    if !args.set.is_empty() {
        let mut vars = config.variables.clone().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(ref mut map) = vars {
            for var_str in &args.set {
                let parts: Vec<&str> = var_str.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    let json_value: serde_json::Value = serde_json::from_str(value)
                        .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
                    map.insert(key.to_string(), json_value);
                }
            }
        }
        config.variables = Some(vars);
    }

    // Get base directory for relative paths
    let base_dir = args
        .config
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    if !args.quiet {
        eprintln!("Running batch with {} jobs...", config.jobs.len());
    }

    // Execute batch
    let results = batch::execute_batch(&config, &base_dir);

    // Format and output results
    let output = batch::format_results(&results);
    write_output(&output)?;

    // Exit with error if any job failed
    let has_failures = results.iter().any(|r| !r.success);
    if has_failures {
        std::process::exit(1);
    }

    Ok(())
}


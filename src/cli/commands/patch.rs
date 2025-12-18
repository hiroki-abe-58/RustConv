//! Patch subcommand implementation

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cli::args::PatchArgs;
use crate::cli::output::write_output;
use crate::core::patcher;
use crate::utils::highlight;

/// Execute the patch subcommand
pub fn execute(args: PatchArgs) -> Result<()> {
    // Read input document
    let doc_content = read_input(args.input.as_deref())?;
    let doc: serde_json::Value = serde_json::from_str(&doc_content)
        .context("Input must be valid JSON")?;

    // Read patch
    let patch_content = fs::read_to_string(&args.patch)
        .with_context(|| format!("Failed to read patch file: {}", args.patch.display()))?;
    let patch_value: serde_json::Value = serde_json::from_str(&patch_content)
        .context("Patch must be valid JSON")?;

    // Parse patch operations
    let operations = patcher::parse_patch(&patch_value)?;

    // Apply patch
    let result = patcher::apply_patch(&doc, &operations)?;

    // Format output
    let output = serde_json::to_string_pretty(&result)?;

    // Write output
    if let Some(ref output_path) = args.output {
        fs::write(output_path, &output)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        if !args.quiet {
            eprintln!("Patched output written to {}", output_path.display());
        }
    } else {
        let highlighted = if args.raw {
            output
        } else {
            highlight::highlight_json(&output)
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


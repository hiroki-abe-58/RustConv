//! Diff subcommand implementation

use anyhow::{Context, Result};
use std::fs;

use crate::cli::args::DiffArgs;
use crate::cli::output::write_output;
use crate::core::differ::{self, DiffFormat};
use crate::formats::detect::detect;

/// Execute the diff subcommand
pub fn execute(args: DiffArgs) -> Result<()> {
    // Read both files
    let content1 = fs::read_to_string(&args.file1)
        .with_context(|| format!("Failed to read file: {}", args.file1.display()))?;
    let content2 = fs::read_to_string(&args.file2)
        .with_context(|| format!("Failed to read file: {}", args.file2.display()))?;

    // Detect formats
    let format1 = detect(Some(args.file1.as_path()), &content1)
        .context("Could not detect format of first file")?;
    let format2 = detect(Some(args.file2.as_path()), &content2)
        .context("Could not detect format of second file")?;

    // Determine output format
    let diff_format = if args.patch {
        DiffFormat::JsonPatch
    } else if args.side_by_side {
        DiffFormat::SideBySide
    } else {
        DiffFormat::Unified
    };

    // Generate diff
    let output = if args.summary {
        differ::diff_summary(&content1, &content2, format1, format2)?
    } else {
        differ::diff(&content1, &content2, format1, format2, diff_format)?
    };

    write_output(&output)?;

    Ok(())
}


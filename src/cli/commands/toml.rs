//! TOML subcommand implementation

use anyhow::Result;

use crate::cli::args::TomlArgs;
use crate::cli::output::write_output;
use crate::formats::toml as toml_format;
use crate::utils::highlight;

/// Execute the toml subcommand
pub fn execute(args: TomlArgs) -> Result<()> {
    let content = toml_format::read_input(args.input.as_deref())?;
    let value = toml_format::parse(&content)?;

    let output = if args.compact {
        toml_format::to_compact(&value)?
    } else {
        toml_format::to_pretty(&value)?
    };

    let highlighted = highlight::highlight_toml(&output);
    write_output(&highlighted)?;

    Ok(())
}

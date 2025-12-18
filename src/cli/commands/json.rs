//! JSON subcommand implementation

use anyhow::Result;

use crate::cli::args::JsonArgs;
use crate::cli::output::write_output;
use crate::formats::json as json_format;
use crate::utils::highlight;

/// Execute the json subcommand
pub fn execute(args: JsonArgs) -> Result<()> {
    let content = json_format::read_input(args.input.as_deref())?;
    let value = json_format::parse(&content)?;

    let output = if args.compact {
        json_format::to_compact(&value)?
    } else {
        json_format::to_pretty(&value)?
    };

    let highlighted = highlight::highlight_json(&output);
    write_output(&highlighted)?;

    Ok(())
}

//! XML subcommand implementation

use anyhow::Result;

use crate::cli::args::XmlArgs;
use crate::cli::output::write_output;
use crate::formats::xml as xml_format;
use crate::utils::highlight;

/// Execute the xml subcommand
pub fn execute(args: XmlArgs) -> Result<()> {
    let content = xml_format::read_input(args.input.as_deref())?;

    // Validate XML first
    xml_format::validate(&content)?;

    let output = if args.compact {
        xml_format::to_compact(&content)?
    } else {
        xml_format::to_pretty(&content)?
    };

    let highlighted = highlight::highlight_xml(&output);
    write_output(&highlighted)?;

    Ok(())
}

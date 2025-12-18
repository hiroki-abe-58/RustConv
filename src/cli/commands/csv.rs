//! CSV subcommand implementation

use anyhow::Result;

use crate::cli::args::CsvArgs;
use crate::cli::output::write_output;
use crate::formats::csv as csv_format;
use crate::utils::highlight;

/// Execute the csv subcommand
pub fn execute(args: CsvArgs) -> Result<()> {
    let content = csv_format::read_input(args.input.as_deref())?;
    let data = csv_format::parse(&content, !args.no_headers)?;

    let output = if args.raw {
        csv_format::to_csv(&data)?
    } else {
        csv_format::to_table(&data)?
    };

    let highlighted = highlight::highlight_csv(&output, args.raw);
    write_output(&highlighted)?;

    Ok(())
}

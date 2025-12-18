//! YAML subcommand implementation

use anyhow::Result;

use crate::cli::args::YamlArgs;
use crate::cli::output::write_output;
use crate::formats::yaml as yaml_format;
use crate::utils::highlight;

/// Execute the yaml subcommand
pub fn execute(args: YamlArgs) -> Result<()> {
    let content = yaml_format::read_input(args.input.as_deref())?;
    let value = yaml_format::parse(&content)?;
    let output = yaml_format::to_pretty(&value)?;

    let highlighted = highlight::highlight_yaml(&output);
    write_output(&highlighted)?;

    Ok(())
}

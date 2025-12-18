//! dtx - Data Transformation CLI entry point

use anyhow::Result;
use clap::Parser;

use dtx::cli::args::{Cli, Commands};
use dtx::cli::commands::{
    auto, batch, completions, convert, csv, diff, json, merge, patch, query, schema, template,
    toml, validate, xml, yaml,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle global --no-color flag
    if cli.no_color {
        colored::control::set_override(false);
    }

    match cli.command {
        Commands::Json(args) => json::execute(args)?,
        Commands::Yaml(args) => yaml::execute(args)?,
        Commands::Toml(args) => toml::execute(args)?,
        Commands::Csv(args) => csv::execute(args)?,
        Commands::Xml(args) => xml::execute(args)?,
        Commands::Auto(args) => auto::execute(args)?,
        Commands::Convert(args) => convert::execute(args)?,
        Commands::Query(args) => query::execute(args)?,
        Commands::Validate(args) => validate::execute(args)?,
        Commands::Diff(args) => diff::execute(args)?,
        Commands::Schema(args) => schema::execute(args)?,
        Commands::Merge(args) => merge::execute(args)?,
        Commands::Patch(args) => patch::execute(args)?,
        Commands::Template(args) => template::execute(args)?,
        Commands::Batch(args) => batch::execute(args)?,
        Commands::Completions(args) => completions::execute(args)?,
    }

    Ok(())
}

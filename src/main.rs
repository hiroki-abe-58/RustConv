//! dtx - Data Transformation CLI entry point

use anyhow::Result;
use clap::Parser;

use dtx::cli::args::{Cli, Commands};
use dtx::cli::commands::{auto, csv, json, toml, xml, yaml};

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
    }

    Ok(())
}

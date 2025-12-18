//! CLI argument definitions using clap

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// dtx - Data Transformation Swiss Army Knife
#[derive(Parser, Debug)]
#[command(name = "dtx")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Read and format JSON data
    Json(JsonArgs),

    /// Read and format YAML data
    Yaml(YamlArgs),
}

/// Arguments for the json subcommand
#[derive(Parser, Debug)]
pub struct JsonArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// Output compact JSON (no pretty printing)
    #[arg(short, long)]
    pub compact: bool,
}

/// Arguments for the yaml subcommand
#[derive(Parser, Debug)]
pub struct YamlArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,
}

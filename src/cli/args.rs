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

    /// Read and format TOML data
    Toml(TomlArgs),

    /// Read and format CSV data
    Csv(CsvArgs),

    /// Read and format XML data
    Xml(XmlArgs),

    /// Auto-detect format and display
    Auto(AutoArgs),

    /// Convert between formats
    Convert(ConvertArgs),
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

/// Arguments for the toml subcommand
#[derive(Parser, Debug)]
pub struct TomlArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// Output compact TOML (no pretty printing)
    #[arg(short, long)]
    pub compact: bool,
}

/// Arguments for the csv subcommand
#[derive(Parser, Debug)]
pub struct CsvArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// Treat first row as data (no headers)
    #[arg(long)]
    pub no_headers: bool,

    /// Output raw CSV instead of table format
    #[arg(short, long)]
    pub raw: bool,
}

/// Arguments for the xml subcommand
#[derive(Parser, Debug)]
pub struct XmlArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// Output compact XML (no pretty printing)
    #[arg(short, long)]
    pub compact: bool,
}

/// Arguments for the auto subcommand
#[derive(Parser, Debug)]
pub struct AutoArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// Suppress format detection message
    #[arg(short, long)]
    pub quiet: bool,
}

/// Arguments for the convert subcommand
#[derive(Parser, Debug)]
pub struct ConvertArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// Target format(s), comma-separated (e.g., yaml,toml,csv)
    #[arg(short, long, required = true)]
    pub to: String,

    /// Source format (auto-detected if not specified)
    #[arg(short, long)]
    pub from: Option<String>,

    /// Output file (outputs to stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Suppress conversion messages
    #[arg(long)]
    pub quiet: bool,
}

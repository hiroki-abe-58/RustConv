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

    /// Query and transform data using JSONPath and filters
    Query(QueryArgs),
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

/// Arguments for the query subcommand
#[derive(Parser, Debug)]
pub struct QueryArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// JSONPath query (e.g., '$.users[*].name')
    #[arg(short, long)]
    pub query: Option<String>,

    /// Extract all keys from objects
    #[arg(long)]
    pub keys: bool,

    /// Extract all values from objects
    #[arg(long)]
    pub values: bool,

    /// Flatten nested structure
    #[arg(long)]
    pub flatten: bool,

    /// Separator for flattened keys (default: ".")
    #[arg(long)]
    pub separator: Option<String>,

    /// Sort object keys alphabetically
    #[arg(long)]
    pub sort_keys: bool,

    /// Filter array elements (e.g., 'age > 20')
    #[arg(long)]
    pub filter: Option<String>,

    /// Select specific fields (comma-separated)
    #[arg(long)]
    pub select: Option<String>,

    /// Get unique values from array
    #[arg(long)]
    pub unique: bool,

    /// Count elements
    #[arg(long)]
    pub count: bool,

    /// Reverse array elements
    #[arg(long)]
    pub reverse: bool,

    /// Get first N elements
    #[arg(long)]
    pub first: Option<usize>,

    /// Get last N elements
    #[arg(long)]
    pub last: Option<usize>,

    /// Apply operations recursively
    #[arg(short, long)]
    pub recursive: bool,

    /// Output compact JSON
    #[arg(short, long)]
    pub compact: bool,

    /// Output without syntax highlighting
    #[arg(long)]
    pub raw: bool,
}

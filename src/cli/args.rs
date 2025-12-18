//! CLI argument definitions using clap

use clap::{Parser, Subcommand};
use clap_complete::Shell;
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

    /// Validate data against schema or lint for issues
    Validate(ValidateArgs),

    /// Compare two files and show differences
    Diff(DiffArgs),

    /// Generate JSON Schema from data
    Schema(SchemaArgs),

    /// Merge multiple files into one
    Merge(MergeArgs),

    /// Apply JSON Patch (RFC 6902) to a document
    Patch(PatchArgs),

    /// Render template with variable substitution
    Template(TemplateArgs),

    /// Execute batch jobs from config file
    Batch(BatchArgs),

    /// Generate shell completion scripts
    Completions(CompletionsArgs),
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

/// Arguments for the validate subcommand
#[derive(Parser, Debug)]
pub struct ValidateArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// JSON Schema file to validate against
    #[arg(short, long)]
    pub schema: Option<PathBuf>,

    /// Specify input format (auto-detected if not specified)
    #[arg(short, long)]
    pub format: Option<String>,

    /// Treat first row as data (for CSV)
    #[arg(long)]
    pub no_headers: bool,
}

/// Arguments for the diff subcommand
#[derive(Parser, Debug)]
pub struct DiffArgs {
    /// First file to compare
    pub file1: PathBuf,

    /// Second file to compare
    pub file2: PathBuf,

    /// Output JSON Patch format (RFC 6902)
    #[arg(long)]
    pub patch: bool,

    /// Side-by-side comparison
    #[arg(short, long)]
    pub side_by_side: bool,

    /// Show only summary of changes
    #[arg(long)]
    pub summary: bool,
}

/// Arguments for the schema subcommand
#[derive(Parser, Debug)]
pub struct SchemaArgs {
    /// Input file (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// Output file (outputs to stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Generate TypeScript interface instead of JSON Schema
    #[arg(long)]
    pub typescript: bool,

    /// Name for generated type/interface
    #[arg(long)]
    pub name: Option<String>,

    /// Output without syntax highlighting
    #[arg(long)]
    pub raw: bool,
}

/// Arguments for the merge subcommand
#[derive(Parser, Debug)]
pub struct MergeArgs {
    /// Files to merge (at least 2 required)
    #[arg(required = true, num_args = 2..)]
    pub files: Vec<PathBuf>,

    /// Output file (outputs to stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Merge strategy: deep, shallow, concat, union
    #[arg(short, long)]
    pub strategy: Option<String>,

    /// Output format (json, yaml, toml)
    #[arg(short, long)]
    pub format: Option<String>,

    /// Suppress output messages
    #[arg(short, long)]
    pub quiet: bool,
}

/// Arguments for the patch subcommand
#[derive(Parser, Debug)]
pub struct PatchArgs {
    /// Input document (reads from stdin if not provided)
    pub input: Option<PathBuf>,

    /// JSON Patch file to apply
    #[arg(short, long, required = true)]
    pub patch: PathBuf,

    /// Output file (outputs to stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Suppress output messages
    #[arg(short, long)]
    pub quiet: bool,

    /// Output without syntax highlighting
    #[arg(long)]
    pub raw: bool,
}

/// Arguments for the template subcommand
#[derive(Parser, Debug)]
pub struct TemplateArgs {
    /// Template file (reads from stdin if not provided)
    pub template: Option<PathBuf>,

    /// Variables file (JSON or YAML)
    #[arg(short, long)]
    pub vars: Option<PathBuf>,

    /// Set individual variables (key=value)
    #[arg(long, action = clap::ArgAction::Append)]
    pub set: Vec<String>,

    /// Include environment variables
    #[arg(short, long)]
    pub env: bool,

    /// Output file (outputs to stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format (json, yaml)
    #[arg(short, long)]
    pub format: Option<String>,

    /// Fail on missing variables
    #[arg(long)]
    pub strict: bool,

    /// Validate template without rendering
    #[arg(long)]
    pub validate: bool,

    /// Suppress output messages
    #[arg(short, long)]
    pub quiet: bool,

    /// Output without syntax highlighting
    #[arg(long)]
    pub raw: bool,
}

/// Arguments for the batch subcommand
#[derive(Parser, Debug)]
pub struct BatchArgs {
    /// Batch config file (YAML, JSON, or TOML)
    pub config: PathBuf,

    /// Set variables for batch jobs (key=value)
    #[arg(long, action = clap::ArgAction::Append)]
    pub set: Vec<String>,

    /// Continue on error
    #[arg(long)]
    pub continue_on_error: bool,

    /// Suppress output messages
    #[arg(short, long)]
    pub quiet: bool,
}

/// Arguments for the completions subcommand
#[derive(Parser, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

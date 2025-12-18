# dtx - Data Transformation CLI

A Swiss Army knife CLI tool for data transformation. Convert, query, and validate data between JSON, YAML, TOML, CSV, and XML formats.

## Installation

```bash
cargo install dtx
```

Or build from source:

```bash
git clone https://github.com/hiroki-abe-58/dtx.git
cd dtx
cargo build --release
```

## Usage

### Format-specific Commands

```bash
# JSON
dtx json input.json
dtx json input.json --compact

# YAML
dtx yaml input.yaml

# TOML
dtx toml input.toml
dtx toml input.toml --compact

# CSV (displays as formatted table)
dtx csv input.csv
dtx csv input.csv --raw          # Output raw CSV
dtx csv input.csv --no-headers   # First row is data

# XML
dtx xml input.xml
dtx xml input.xml --compact

# Auto-detect format
dtx auto input.json
dtx auto input.yaml --quiet
```

### Format Conversion

```bash
# Basic conversion
dtx convert input.json --to yaml
dtx convert input.yaml --to json
dtx convert input.json --to toml
dtx convert data.csv --to json

# Multiple target formats (comma-separated)
dtx convert input.json --to yaml,toml,csv

# Specify source format explicitly
dtx convert input.txt --from json --to yaml

# Output to file
dtx convert input.json --to yaml --output output.yaml

# Pipe from stdin
cat data.json | dtx convert --from json --to yaml
```

### Supported Conversions

| From | To JSON | To YAML | To TOML | To CSV | To XML |
|------|---------|---------|---------|--------|--------|
| JSON | -       | Yes     | Yes     | Yes*   | Yes    |
| YAML | Yes     | -       | Yes     | Yes*   | Yes    |
| TOML | Yes     | Yes     | -       | Yes*   | Yes    |
| CSV  | Yes     | Yes     | -       | -      | -      |
| XML  | Yes     | Yes     | -       | -      | -      |

*CSV conversion requires array of objects structure

## Features

### Phase 1 (v0.1.0) - Foundation
- JSON/YAML reading and pretty printing
- Standard input / file input support
- Syntax highlighting with color output

### Phase 2 (v0.2.0) - Format Support
- TOML/CSV/XML reading and pretty printing
- Auto format detection (from extension and content)
- CSV table formatting

### Phase 3 (v0.3.0) - Conversion Engine
- Full cross-format conversion support
- Multiple target formats in single command
- File output support
- Intermediate JSON representation for lossless conversion

## Roadmap

- **Phase 4**: JSONPath query and jq-compatible filters
- **Phase 5**: Schema validation and diff
- **Phase 6**: Merge, patch, template, and batch processing
- **Phase 7**: i18n and shell completions
- **Phase 8**: AI-powered natural language queries

## Examples

### Convert JSON config to YAML

```bash
dtx convert config.json --to yaml
```

### Convert CSV data to JSON array

```bash
dtx convert users.csv --to json
```

Output:
```json
[
  {"name": "Alice", "age": 30},
  {"name": "Bob", "age": 25}
]
```

### Convert XML to JSON

```bash
dtx convert data.xml --to json
```

### Batch convert to multiple formats

```bash
dtx convert config.json --to yaml,toml
```

## License

MIT License

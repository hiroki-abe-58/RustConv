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

### JSON

```bash
# From file
dtx json input.json

# From stdin
cat data.json | dtx json

# Compact output
dtx json input.json --compact

# Disable color output
dtx json input.json --no-color
```

### YAML

```bash
dtx yaml input.yaml
cat data.yaml | dtx yaml
```

### TOML

```bash
dtx toml input.toml
dtx toml input.toml --compact
```

### CSV

```bash
# Display as formatted table
dtx csv input.csv

# Output raw CSV
dtx csv input.csv --raw

# Treat first row as data (no headers)
dtx csv input.csv --no-headers
```

### XML

```bash
# Pretty print XML
dtx xml input.xml

# Minify XML
dtx xml input.xml --compact
```

### Auto-detect Format

```bash
# Automatically detect format from file extension or content
dtx auto input.json
dtx auto input.yaml
dtx auto data.csv

# Suppress format detection message
dtx auto input.json --quiet
```

## Features

### Phase 1 (v0.1.0) - Foundation
- JSON reading and pretty printing
- YAML reading and pretty printing
- Standard input / file input support
- Syntax highlighting with color output

### Phase 2 (v0.2.0) - Format Support
- TOML reading and pretty printing
- CSV reading with table formatting
- XML reading and pretty printing
- Auto format detection (from extension and content)
- Syntax highlighting for all formats

## Roadmap

- **Phase 3**: Cross-format conversion engine (JSON <-> YAML <-> TOML <-> CSV <-> XML)
- **Phase 4**: JSONPath query and jq-compatible filters
- **Phase 5**: Schema validation and diff
- **Phase 6**: Merge, patch, template, and batch processing
- **Phase 7**: i18n and shell completions
- **Phase 8**: AI-powered natural language queries

## Supported Formats

| Format | Read | Pretty Print | Compact | Auto-detect |
|--------|------|--------------|---------|-------------|
| JSON   | Yes  | Yes          | Yes     | Yes         |
| YAML   | Yes  | Yes          | -       | Yes         |
| TOML   | Yes  | Yes          | Yes     | Yes         |
| CSV    | Yes  | Table/Raw    | -       | Yes         |
| XML    | Yes  | Yes          | Yes     | Yes         |

## License

MIT License

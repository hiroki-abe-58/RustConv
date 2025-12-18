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

### JSON Pretty Print

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

### YAML Pretty Print

```bash
# From file
dtx yaml input.yaml

# From stdin
cat data.yaml | dtx yaml

# Disable color output
dtx yaml input.yaml --no-color
```

## Features (Phase 1)

- JSON reading and pretty printing
- YAML reading and pretty printing
- Standard input / file input support
- Syntax highlighting with color output

## Roadmap

- **Phase 2**: TOML, CSV, XML support + auto format detection
- **Phase 3**: Cross-format conversion engine
- **Phase 4**: JSONPath query and jq-compatible filters
- **Phase 5**: Schema validation and diff
- **Phase 6**: Merge, patch, template, and batch processing
- **Phase 7**: i18n and shell completions
- **Phase 8**: AI-powered natural language queries

## License

MIT License


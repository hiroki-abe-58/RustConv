# dtx - Data Transformation CLI

A Swiss Army knife CLI tool for data transformation. Convert, query, validate, merge, and batch process data between JSON, YAML, TOML, CSV, and XML formats.

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
# JSON, YAML, TOML, CSV, XML
dtx json input.json
dtx yaml input.yaml
dtx toml input.toml
dtx csv input.csv
dtx xml input.xml

# Auto-detect format
dtx auto input.json
```

### Format Conversion

```bash
# Basic conversion
dtx convert input.json --to yaml
dtx convert input.yaml --to json
dtx convert data.csv --to json

# Multiple target formats
dtx convert input.json --to yaml,toml,csv

# Output to file
dtx convert input.json --to yaml --output output.yaml
```

### Query and Transform

```bash
# JSONPath query
dtx query data.json -q '$.users[*].name'
dtx query data.json -q '$.store.book[?(@.price < 10)]'

# Extract keys/values
dtx query data.json --keys
dtx query data.json --values
dtx query data.json --keys --recursive

# Flatten nested structure
dtx query data.json --flatten
dtx query data.json --flatten --separator "_"

# Sort keys
dtx query data.json --sort-keys

# Filter array elements
dtx query data.json -q '$.users' --filter 'age > 25'
dtx query data.json -q '$.users' --filter 'name == "Alice"'
dtx query data.json -q '$.items' --filter 'status contains active'

# Select specific fields
dtx query data.json -q '$.users' --select 'name,email'

# Array operations
dtx query data.json -q '$.items' --first 5
dtx query data.json -q '$.items' --last 3
dtx query data.json -q '$.items' --reverse
dtx query data.json -q '$.items' --unique
dtx query data.json -q '$.items' --count
```

### Validation

```bash
# Lint JSON for issues
dtx validate data.json

# Lint YAML
dtx validate config.yaml

# Lint TOML
dtx validate config.toml

# Validate CSV structure
dtx validate data.csv

# Validate against JSON Schema
dtx validate data.json --schema schema.json

# Specify format explicitly
dtx validate data.json --format json
```

### Diff (Compare Files)

```bash
# Compare two JSON files (unified diff)
dtx diff file1.json file2.json

# Compare different formats (auto-converts for comparison)
dtx diff data.json data.yaml

# Side-by-side comparison
dtx diff file1.json file2.json --side-by-side

# JSON Patch format (RFC 6902)
dtx diff file1.json file2.json --patch

# Summary only
dtx diff file1.json file2.json --summary
```

### Schema Generation

```bash
# Generate JSON Schema from data
dtx schema data.json

# Generate TypeScript interface
dtx schema data.json --typescript

# Specify interface name
dtx schema data.json --typescript --name UserData

# Output to file
dtx schema data.json --output schema.json

# Raw output (no syntax highlighting)
dtx schema data.json --raw
```

### Merge Files

```bash
# Deep merge (default) - recursively merge, later values win
dtx merge config1.json config2.json

# Shallow merge - only top-level keys
dtx merge config1.json config2.json --strategy shallow

# Concat arrays instead of replacing
dtx merge config1.json config2.json --strategy concat

# Union arrays (unique values only)
dtx merge config1.json config2.json --strategy union

# Output to file
dtx merge base.yaml override.yaml --output merged.yaml

# Specify output format
dtx merge a.json b.yaml --format yaml
```

### Apply JSON Patch

```bash
# Apply patch to document
dtx patch input.json --patch changes.json

# Output to file
dtx patch input.json --patch changes.json --output patched.json

# Patch from stdin
cat input.json | dtx patch --patch changes.json
```

### Template Rendering

```bash
# Render template with variables file
dtx template template.json --vars variables.yaml

# Set individual variables
dtx template template.json --set name=Alice --set age=30

# Include environment variables
dtx template template.json --vars config.yaml --env

# Strict mode - fail on missing variables
dtx template template.json --vars partial.yaml --strict

# Validate template without rendering
dtx template template.json --vars config.yaml --validate
```

### Batch Processing

```bash
# Execute batch jobs from config
dtx batch jobs.yaml

# With variables
dtx batch jobs.yaml --set env=production

# Continue on error
dtx batch jobs.yaml --continue-on-error
```

## Features

### Phase 1 (v0.1.0) - Foundation
- JSON/YAML reading and pretty printing
- Syntax highlighting with color output

### Phase 2 (v0.2.0) - Format Support
- TOML/CSV/XML reading and pretty printing
- Auto format detection

### Phase 3 (v0.3.0) - Conversion Engine
- Full cross-format conversion support
- Multiple target formats in single command

### Phase 4 (v0.4.0) - Query & Transform
- JSONPath query support
- Key/value extraction
- Flatten nested structures
- Sort keys
- Filter expressions (==, !=, >, <, >=, <=, contains, startswith, endswith)
- Field selection
- Array operations (first, last, reverse, unique, count)

### Phase 5 (v0.5.0) - Validation & Schema
- JSON Schema validation
- JSON/YAML/TOML/CSV linting
- Diff comparison (unified, side-by-side, JSON Patch)
- JSON Schema generation from data
- TypeScript interface generation

### Phase 6 (v0.6.0) - Merge & Utilities
- Merge files with multiple strategies (deep, shallow, concat, union)
- JSON Patch (RFC 6902) support
- Template rendering with variable substitution
- Batch processing for automation

## Merge Strategies

| Strategy | Description |
|----------|-------------|
| `deep` | Recursively merge nested objects, later values win |
| `shallow` | Only merge top-level keys |
| `concat` | Concatenate arrays instead of replacing |
| `union` | Merge arrays with unique values only |

## Template Syntax

Templates use `{{ variable }}` syntax for variable substitution:

```json
{
  "greeting": "Hello, {{ user.name }}!",
  "config": {
    "env": "{{ environment }}",
    "items": ["{{ items[0] }}", "{{ items[1] }}"]
  }
}
```

Variables support:
- Dot notation: `{{ user.address.city }}`
- Array indexing: `{{ items[0] }}`
- Nested paths: `{{ config.server.host }}`

## Batch Configuration

Batch configs support YAML, JSON, or TOML format:

```yaml
continue_on_error: true
variables:
  env: production
jobs:
  - name: "Validate config"
    action: validate
    input: "config.json"
    schema: "schema.json"

  - name: "Convert to YAML"
    action: convert
    input: "config.json"
    output: "output/config.yaml"
    to: "yaml"

  - name: "Merge configs"
    action: merge
    inputs:
      - "base.json"
      - "override.json"
    output: "merged.json"
    strategy: "deep"

  - name: "Transform data"
    action: transform
    input: "data.json"
    output: "users.json"
    query: "$.users[*]"
```

### Supported Batch Actions

| Action | Description |
|--------|-------------|
| `convert` | Convert file between formats |
| `merge` | Merge multiple files |
| `validate` | Validate file (with optional schema) |
| `copy` | Copy file |
| `transform` | Apply JSONPath query |

## Roadmap

- **Phase 7**: i18n and shell completions
- **Phase 8**: AI-powered natural language queries

## License

MIT License

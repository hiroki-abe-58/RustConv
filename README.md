# dtx - Data Transformation CLI

A Swiss Army knife CLI tool for data transformation. Convert, query, validate, and compare data between JSON, YAML, TOML, CSV, and XML formats.

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

## Query Examples

### JSONPath Syntax

```bash
# Get all user names
dtx query users.json -q '$.users[*].name'

# Get first user
dtx query users.json -q '$.users[0]'

# Get users with specific property
dtx query users.json -q '$.users[?(@.active == true)]'

# Nested path
dtx query data.json -q '$.store.book[*].author'
```

### Filter Expressions

```bash
# Numeric comparison
dtx query data.json -q '$.items' --filter 'price > 100'
dtx query data.json -q '$.users' --filter 'age >= 18'

# String comparison
dtx query data.json -q '$.users' --filter 'name == "Alice"'
dtx query data.json -q '$.items' --filter 'category != "electronics"'

# String operations
dtx query data.json -q '$.products' --filter 'name contains phone'
dtx query data.json -q '$.files' --filter 'path startswith /home'
dtx query data.json -q '$.urls' --filter 'url endswith .html'
```

### Combining Operations

```bash
# Filter then select fields
dtx query users.json -q '$.users' --filter 'age > 25' --select 'name,email'

# Query, filter, and get first 5
dtx query data.json -q '$.products' --filter 'price < 50' --first 5

# Flatten and sort
dtx query config.json --flatten --sort-keys
```

## Validation Examples

### JSON Schema Validation

```bash
# Define a schema (schema.json)
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "age": {"type": "integer", "minimum": 0}
  },
  "required": ["name", "age"]
}

# Validate data against schema
dtx validate user.json --schema schema.json
```

### Linting Checks

The linter checks for:
- **JSON**: Syntax errors, empty objects/arrays, similar keys (case differences), mixed array types
- **YAML**: Syntax errors, tab characters, trailing whitespace, inconsistent indentation
- **TOML**: Syntax errors, trailing whitespace, long lines
- **CSV**: Column count consistency, empty cells, duplicate headers

## Diff Examples

```bash
# See what changed between versions
dtx diff config-old.json config-new.json

# Generate JSON Patch for API updates
dtx diff old-api.json new-api.json --patch

# Quick overview of changes
dtx diff v1.yaml v2.yaml --summary
```

## Roadmap

- **Phase 6**: Merge, patch, template, and batch processing
- **Phase 7**: i18n and shell completions
- **Phase 8**: AI-powered natural language queries

## License

MIT License

//! Validation engine for various data formats

use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value as JsonValue;

use crate::formats::csv as csv_format;

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub path: String,
    pub message: String,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, path: &str, message: &str) {
        self.valid = false;
        self.errors.push(ValidationError {
            path: path.to_string(),
            message: message.to_string(),
        });
    }

    pub fn add_warning(&mut self, path: &str, message: &str) {
        self.warnings.push(ValidationWarning {
            path: path.to_string(),
            message: message.to_string(),
        });
    }

    pub fn format_output(&self) -> String {
        let mut output = String::new();

        if self.valid {
            output.push_str(&format!("{}\n", "Validation passed".green().bold()));
        } else {
            output.push_str(&format!("{}\n", "Validation failed".red().bold()));
        }

        if !self.errors.is_empty() {
            output.push_str(&format!("\n{} ({}):\n", "Errors".red(), self.errors.len()));
            for error in &self.errors {
                output.push_str(&format!(
                    "  {} {}: {}\n",
                    "x".red(),
                    error.path.cyan(),
                    error.message
                ));
            }
        }

        if !self.warnings.is_empty() {
            output.push_str(&format!(
                "\n{} ({}):\n",
                "Warnings".yellow(),
                self.warnings.len()
            ));
            for warning in &self.warnings {
                output.push_str(&format!(
                    "  {} {}: {}\n",
                    "!".yellow(),
                    warning.path.cyan(),
                    warning.message
                ));
            }
        }

        output
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate JSON against a JSON Schema
pub fn validate_json_schema(data: &JsonValue, schema: &JsonValue) -> Result<ValidationResult> {
    let validator = jsonschema::validator_for(schema)
        .map_err(|e| anyhow::anyhow!("Invalid JSON Schema: {}", e))?;

    let mut result = ValidationResult::new();

    for error in validator.iter_errors(data) {
        let path = error.instance_path.to_string();
        let path = if path.is_empty() {
            "$".to_string()
        } else {
            path
        };
        result.add_error(&path, &error.to_string());
    }

    Ok(result)
}

/// Lint JSON for common issues
pub fn lint_json(content: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Try to parse
    let value: JsonValue = serde_json::from_str(content).context("Invalid JSON syntax")?;

    // Check for common issues
    lint_json_value(&value, "$", &mut result);

    Ok(result)
}

fn lint_json_value(value: &JsonValue, path: &str, result: &mut ValidationResult) {
    match value {
        JsonValue::Object(obj) => {
            // Check for empty objects
            if obj.is_empty() {
                result.add_warning(path, "Empty object");
            }

            // Check for duplicate-like keys (case sensitivity)
            let keys: Vec<&String> = obj.keys().collect();
            for (i, key1) in keys.iter().enumerate() {
                for key2 in keys.iter().skip(i + 1) {
                    if key1.to_lowercase() == key2.to_lowercase() && key1 != key2 {
                        result.add_warning(
                            path,
                            &format!(
                                "Similar keys with different case: '{}' and '{}'",
                                key1, key2
                            ),
                        );
                    }
                }
            }

            // Recurse into children
            for (key, val) in obj {
                let child_path = format!("{}.{}", path, key);
                lint_json_value(val, &child_path, result);
            }
        }
        JsonValue::Array(arr) => {
            // Check for empty arrays
            if arr.is_empty() {
                result.add_warning(path, "Empty array");
            }

            // Check for mixed types in array
            if arr.len() > 1 {
                let first_type = get_json_type(&arr[0]);
                for (i, item) in arr.iter().enumerate().skip(1) {
                    let item_type = get_json_type(item);
                    if item_type != first_type && first_type != "null" && item_type != "null" {
                        result.add_warning(
                            path,
                            &format!(
                                "Mixed types in array: {} at index 0, {} at index {}",
                                first_type, item_type, i
                            ),
                        );
                        break;
                    }
                }
            }

            // Recurse into children
            for (i, val) in arr.iter().enumerate() {
                let child_path = format!("{}[{}]", path, i);
                lint_json_value(val, &child_path, result);
            }
        }
        JsonValue::String(s) => {
            // Check for potential issues in strings
            if s.trim().is_empty() && !s.is_empty() {
                result.add_warning(path, "String contains only whitespace");
            }
        }
        _ => {}
    }
}

fn get_json_type(value: &JsonValue) -> &'static str {
    match value {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "boolean",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}

/// Lint YAML for common issues
pub fn lint_yaml(content: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Try to parse
    let _value: serde_yaml::Value =
        serde_yaml::from_str(content).context("Invalid YAML syntax")?;

    // Check for tabs (YAML should use spaces)
    for (i, line) in content.lines().enumerate() {
        if line.contains('\t') {
            result.add_warning(
                &format!("line {}", i + 1),
                "Tab character found (YAML should use spaces for indentation)",
            );
        }
    }

    // Check for trailing whitespace
    for (i, line) in content.lines().enumerate() {
        if line != line.trim_end() {
            result.add_warning(&format!("line {}", i + 1), "Trailing whitespace");
        }
    }

    // Check for inconsistent indentation
    let mut indent_size: Option<usize> = None;
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let leading_spaces = line.len() - trimmed.len();
            if leading_spaces > 0 {
                if let Some(expected) = indent_size {
                    if leading_spaces % expected != 0 {
                        result.add_warning(
                            &format!("line {}", i + 1),
                            &format!(
                                "Inconsistent indentation: {} spaces (expected multiple of {})",
                                leading_spaces, expected
                            ),
                        );
                    }
                } else if leading_spaces >= 2 {
                    indent_size = Some(leading_spaces);
                }
            }
        }
    }

    Ok(result)
}

/// Lint TOML for common issues
pub fn lint_toml(content: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Try to parse
    let _value: toml::Value = content.parse().context("Invalid TOML syntax")?;

    // Check for trailing whitespace
    for (i, line) in content.lines().enumerate() {
        if line != line.trim_end() {
            result.add_warning(&format!("line {}", i + 1), "Trailing whitespace");
        }
    }

    // Check for very long lines
    for (i, line) in content.lines().enumerate() {
        if line.len() > 120 {
            result.add_warning(
                &format!("line {}", i + 1),
                &format!("Line too long: {} characters (recommended max: 120)", line.len()),
            );
        }
    }

    Ok(result)
}

/// Validate CSV structure
pub fn validate_csv(content: &str, has_headers: bool) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    let data = csv_format::parse(content, has_headers)?;

    // Check column count consistency
    let expected_cols = data
        .headers
        .as_ref()
        .map(|h| h.len())
        .unwrap_or_else(|| data.rows.first().map(|r| r.len()).unwrap_or(0));

    for (i, row) in data.rows.iter().enumerate() {
        if row.len() != expected_cols {
            result.add_error(
                &format!("row {}", i + 1 + if has_headers { 1 } else { 0 }),
                &format!(
                    "Column count mismatch: expected {}, found {}",
                    expected_cols,
                    row.len()
                ),
            );
        }
    }

    // Check for empty cells
    for (i, row) in data.rows.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            if cell.trim().is_empty() {
                let col_name = data
                    .headers
                    .as_ref()
                    .and_then(|h| h.get(j))
                    .map(|s| s.as_str())
                    .unwrap_or("column");
                let row_num = i + 1 + if has_headers { 1 } else { 0 };
                result.add_warning(
                    &format!("row {}, {}", row_num, col_name),
                    "Empty cell",
                );
            }
        }
    }

    // Check for duplicate headers
    if let Some(headers) = &data.headers {
        let mut seen = std::collections::HashSet::new();
        for header in headers {
            if !seen.insert(header.to_lowercase()) {
                result.add_error("headers", &format!("Duplicate header: '{}'", header));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_schema_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name"]
        });

        let valid_data = json!({"name": "Alice", "age": 30});
        let result = validate_json_schema(&valid_data, &schema).unwrap();
        assert!(result.valid);

        let invalid_data = json!({"age": 30});
        let result = validate_json_schema(&invalid_data, &schema).unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn test_lint_json() {
        let json = r#"{"name": "test", "items": []}"#;
        let result = lint_json(json).unwrap();
        assert!(result.warnings.iter().any(|w| w.message.contains("Empty array")));
    }

    #[test]
    fn test_validate_csv() {
        // Test for duplicate headers
        let csv = "name,name\nAlice,30\nBob,25";
        let result = validate_csv(csv, true).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.message.contains("Duplicate")));
    }
}


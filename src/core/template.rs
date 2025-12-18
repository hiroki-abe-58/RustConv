//! Template engine for variable substitution

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value as JsonValue;

/// Template options
#[derive(Debug, Clone)]
pub struct TemplateOptions {
    /// Variable delimiter start (default: "{{")
    pub delimiter_start: String,
    /// Variable delimiter end (default: "}}")
    pub delimiter_end: String,
    /// Fail on missing variables (default: false)
    pub strict: bool,
    /// Default value for missing variables
    pub default_value: Option<String>,
}

impl Default for TemplateOptions {
    fn default() -> Self {
        Self {
            delimiter_start: "{{".to_string(),
            delimiter_end: "}}".to_string(),
            strict: false,
            default_value: None,
        }
    }
}

/// Render a template string with variables
pub fn render_string(template: &str, vars: &JsonValue, options: &TemplateOptions) -> Result<String> {
    let pattern = format!(
        "{}\\s*([\\w.\\[\\]]+)\\s*{}",
        regex::escape(&options.delimiter_start),
        regex::escape(&options.delimiter_end)
    );
    let re = Regex::new(&pattern).context("Failed to compile template regex")?;

    let mut result = template.to_string();
    let mut replacements: Vec<(String, String)> = Vec::new();

    for cap in re.captures_iter(template) {
        let full_match = cap.get(0).unwrap().as_str();
        let var_path = cap.get(1).unwrap().as_str();

        let value = get_var_value(vars, var_path);

        let replacement = match value {
            Some(v) => json_value_to_string(v),
            None => {
                if options.strict {
                    anyhow::bail!("Variable '{}' not found", var_path);
                }
                options
                    .default_value
                    .clone()
                    .unwrap_or_else(|| full_match.to_string())
            }
        };

        replacements.push((full_match.to_string(), replacement));
    }

    for (pattern, replacement) in replacements {
        result = result.replace(&pattern, &replacement);
    }

    Ok(result)
}

/// Render a template JSON value with variables
pub fn render_value(template: &JsonValue, vars: &JsonValue, options: &TemplateOptions) -> Result<JsonValue> {
    match template {
        JsonValue::String(s) => {
            let rendered = render_string(s, vars, options)?;
            // Try to parse as JSON if it looks like a JSON value
            if let Ok(parsed) = serde_json::from_str(&rendered) {
                Ok(parsed)
            } else {
                Ok(JsonValue::String(rendered))
            }
        }
        JsonValue::Array(arr) => {
            let rendered: Result<Vec<JsonValue>> = arr
                .iter()
                .map(|v| render_value(v, vars, options))
                .collect();
            Ok(JsonValue::Array(rendered?))
        }
        JsonValue::Object(obj) => {
            let mut result = serde_json::Map::new();
            for (key, value) in obj {
                let rendered_key = render_string(key, vars, options)?;
                let rendered_value = render_value(value, vars, options)?;
                result.insert(rendered_key, rendered_value);
            }
            Ok(JsonValue::Object(result))
        }
        _ => Ok(template.clone()),
    }
}

/// Get variable value from JSON using dot notation
fn get_var_value<'a>(vars: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let mut current = vars;

    for part in path.split('.') {
        // Check for array index notation: items[0]
        if let Some(bracket_pos) = part.find('[') {
            let key = &part[..bracket_pos];
            let index_str = part[bracket_pos + 1..part.len() - 1].to_string();

            if !key.is_empty() {
                current = current.get(key)?;
            }

            let index: usize = index_str.parse().ok()?;
            current = current.get(index)?;
        } else {
            current = current.get(part)?;
        }
    }

    Some(current)
}

/// Convert JSON value to string for template substitution
fn json_value_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::String(s) => s.clone(),
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        // For complex types, serialize as JSON
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Parse environment variables into a JSON object
pub fn env_to_json() -> JsonValue {
    let mut map = serde_json::Map::new();
    for (key, value) in std::env::vars() {
        map.insert(key, JsonValue::String(value));
    }
    JsonValue::Object(map)
}

/// Merge multiple variable sources (later sources override earlier)
pub fn merge_vars(sources: &[&JsonValue]) -> JsonValue {
    let mut result = serde_json::Map::new();

    for source in sources {
        if let JsonValue::Object(obj) = source {
            for (key, value) in obj {
                result.insert(key.clone(), value.clone());
            }
        }
    }

    JsonValue::Object(result)
}

/// Extract variables from template string
pub fn extract_variables(template: &str, options: &TemplateOptions) -> Vec<String> {
    let pattern = format!(
        "{}\\s*([\\w.\\[\\]]+)\\s*{}",
        regex::escape(&options.delimiter_start),
        regex::escape(&options.delimiter_end)
    );
    let re = Regex::new(&pattern).unwrap();

    re.captures_iter(template)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Validate that all template variables have corresponding values
pub fn validate_template(template: &JsonValue, vars: &JsonValue, options: &TemplateOptions) -> Result<Vec<String>> {
    let mut missing = Vec::new();
    validate_template_recursive(template, vars, options, &mut missing);
    Ok(missing)
}

fn validate_template_recursive(
    template: &JsonValue,
    vars: &JsonValue,
    options: &TemplateOptions,
    missing: &mut Vec<String>,
) {
    match template {
        JsonValue::String(s) => {
            for var in extract_variables(s, options) {
                if get_var_value(vars, &var).is_none() && !missing.contains(&var) {
                    missing.push(var);
                }
            }
        }
        JsonValue::Array(arr) => {
            for item in arr {
                validate_template_recursive(item, vars, options, missing);
            }
        }
        JsonValue::Object(obj) => {
            for (key, value) in obj {
                for var in extract_variables(key, options) {
                    if get_var_value(vars, &var).is_none() && !missing.contains(&var) {
                        missing.push(var);
                    }
                }
                validate_template_recursive(value, vars, options, missing);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_string() {
        let vars = json!({
            "name": "Alice",
            "age": 30
        });
        let options = TemplateOptions::default();

        let result = render_string("Hello, {{ name }}! You are {{ age }} years old.", &vars, &options).unwrap();
        assert_eq!(result, "Hello, Alice! You are 30 years old.");
    }

    #[test]
    fn test_render_nested_vars() {
        let vars = json!({
            "user": {
                "name": "Bob",
                "address": {
                    "city": "Tokyo"
                }
            }
        });
        let options = TemplateOptions::default();

        let result = render_string("{{ user.name }} lives in {{ user.address.city }}", &vars, &options).unwrap();
        assert_eq!(result, "Bob lives in Tokyo");
    }

    #[test]
    fn test_render_array_index() {
        let vars = json!({
            "items": ["first", "second", "third"]
        });
        let options = TemplateOptions::default();

        let result = render_string("{{ items[0] }} and {{ items[2] }}", &vars, &options).unwrap();
        assert_eq!(result, "first and third");
    }

    #[test]
    fn test_render_value() {
        let template = json!({
            "greeting": "Hello, {{ name }}!",
            "data": {
                "age": "{{ age }}"
            }
        });
        let vars = json!({
            "name": "Charlie",
            "age": 25
        });
        let options = TemplateOptions::default();

        let result = render_value(&template, &vars, &options).unwrap();
        assert_eq!(result["greeting"], "Hello, Charlie!");
        // "25" is parsed as JSON number 25
        assert_eq!(result["data"]["age"], 25);
    }

    #[test]
    fn test_extract_variables() {
        let template = "Hello {{ name }}, your balance is {{ account.balance }}";
        let options = TemplateOptions::default();

        let vars = extract_variables(template, &options);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"account.balance".to_string()));
    }
}


//! Query engine for JSONPath and data transformation operations

use anyhow::{bail, Context, Result};
use jsonpath_rust::JsonPath;
use serde_json::{Map, Value as JsonValue};
use std::str::FromStr;

/// Execute a JSONPath query on JSON data
pub fn jsonpath_query(value: &JsonValue, path: &str) -> Result<JsonValue> {
    let json_path =
        JsonPath::from_str(path).with_context(|| format!("Invalid JSONPath: {}", path))?;

    let results = json_path.find(value);

    // Results is a JsonValue (usually an array)
    match results {
        JsonValue::Array(arr) if arr.len() == 1 => Ok(arr.into_iter().next().unwrap()),
        other => Ok(other),
    }
}

/// Extract all keys from a JSON object (recursive)
pub fn extract_keys(value: &JsonValue, recursive: bool) -> JsonValue {
    let mut keys = Vec::new();
    collect_keys(value, recursive, &mut keys, String::new());
    JsonValue::Array(keys.into_iter().map(JsonValue::String).collect())
}

fn collect_keys(value: &JsonValue, recursive: bool, keys: &mut Vec<String>, prefix: String) {
    if let JsonValue::Object(obj) = value {
        for (key, val) in obj {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            if recursive {
                keys.push(full_key.clone());
                collect_keys(val, recursive, keys, full_key);
            } else {
                keys.push(key.clone());
            }
        }
    } else if let JsonValue::Array(arr) = value {
        if recursive {
            for (i, item) in arr.iter().enumerate() {
                let item_prefix = if prefix.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", prefix, i)
                };
                collect_keys(item, recursive, keys, item_prefix);
            }
        }
    }
}

/// Extract all values from a JSON object (recursive)
pub fn extract_values(value: &JsonValue, recursive: bool) -> JsonValue {
    let mut values = Vec::new();
    collect_values(value, recursive, &mut values);
    JsonValue::Array(values)
}

fn collect_values(value: &JsonValue, recursive: bool, values: &mut Vec<JsonValue>) {
    match value {
        JsonValue::Object(obj) => {
            for val in obj.values() {
                if recursive {
                    if val.is_object() || val.is_array() {
                        collect_values(val, recursive, values);
                    } else {
                        values.push(val.clone());
                    }
                } else {
                    values.push(val.clone());
                }
            }
        }
        JsonValue::Array(arr) => {
            if recursive {
                for item in arr {
                    if item.is_object() || item.is_array() {
                        collect_values(item, recursive, values);
                    } else {
                        values.push(item.clone());
                    }
                }
            } else {
                values.extend(arr.clone());
            }
        }
        _ => {
            values.push(value.clone());
        }
    }
}

/// Flatten a nested JSON structure
pub fn flatten(value: &JsonValue, separator: &str) -> JsonValue {
    let mut result = Map::new();
    flatten_recursive(value, String::new(), separator, &mut result);
    JsonValue::Object(result)
}

fn flatten_recursive(
    value: &JsonValue,
    prefix: String,
    separator: &str,
    result: &mut Map<String, JsonValue>,
) {
    match value {
        JsonValue::Object(obj) => {
            for (key, val) in obj {
                let new_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}{}{}", prefix, separator, key)
                };
                flatten_recursive(val, new_key, separator, result);
            }
        }
        JsonValue::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                let new_key = if prefix.is_empty() {
                    format!("{}", i)
                } else {
                    format!("{}{}[{}]", prefix, separator, i)
                };
                flatten_recursive(item, new_key, separator, result);
            }
        }
        _ => {
            result.insert(prefix, value.clone());
        }
    }
}

/// Sort object keys alphabetically (recursive)
pub fn sort_keys(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(obj) => {
            let mut sorted: Vec<(String, JsonValue)> =
                obj.iter().map(|(k, v)| (k.clone(), sort_keys(v))).collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));

            let mut new_obj = Map::new();
            for (k, v) in sorted {
                new_obj.insert(k, v);
            }
            JsonValue::Object(new_obj)
        }
        JsonValue::Array(arr) => JsonValue::Array(arr.iter().map(sort_keys).collect()),
        _ => value.clone(),
    }
}

/// Filter array elements based on a simple expression
/// Supports: field == value, field != value, field > value, field < value, field >= value, field <= value
pub fn filter_array(value: &JsonValue, expression: &str) -> Result<JsonValue> {
    let arr = value
        .as_array()
        .context("Filter can only be applied to arrays")?;

    let filter = parse_filter_expression(expression)?;
    let filtered: Vec<JsonValue> = arr
        .iter()
        .filter(|item| evaluate_filter(item, &filter))
        .cloned()
        .collect();

    Ok(JsonValue::Array(filtered))
}

#[derive(Debug)]
enum FilterOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug)]
struct FilterExpression {
    field: String,
    op: FilterOp,
    value: String,
}

fn parse_filter_expression(expr: &str) -> Result<FilterExpression> {
    let expr = expr.trim();

    // Try to match operators (order matters - longer operators first)
    let operators = [
        (">=", FilterOp::Ge),
        ("<=", FilterOp::Le),
        ("!=", FilterOp::Ne),
        ("==", FilterOp::Eq),
        (">", FilterOp::Gt),
        ("<", FilterOp::Lt),
        (" contains ", FilterOp::Contains),
        (" startswith ", FilterOp::StartsWith),
        (" endswith ", FilterOp::EndsWith),
    ];

    for (op_str, op) in operators {
        if let Some(pos) = expr.to_lowercase().find(op_str) {
            let field = expr[..pos].trim().to_string();
            let value = expr[pos + op_str.len()..].trim().to_string();

            // Remove quotes from value if present
            let value = value.trim_matches('"').trim_matches('\'').to_string();

            return Ok(FilterExpression { field, op, value });
        }
    }

    bail!(
        "Invalid filter expression: {}. Use format: field op value (e.g., age > 20, name == \"test\")",
        expr
    )
}

fn evaluate_filter(item: &JsonValue, filter: &FilterExpression) -> bool {
    // Handle nested field paths (e.g., "user.name")
    let field_value = get_nested_value(item, &filter.field);

    match field_value {
        Some(val) => match &filter.op {
            FilterOp::Eq => compare_values(val, &filter.value) == Some(std::cmp::Ordering::Equal),
            FilterOp::Ne => compare_values(val, &filter.value) != Some(std::cmp::Ordering::Equal),
            FilterOp::Gt => compare_values(val, &filter.value) == Some(std::cmp::Ordering::Greater),
            FilterOp::Lt => compare_values(val, &filter.value) == Some(std::cmp::Ordering::Less),
            FilterOp::Ge => {
                matches!(
                    compare_values(val, &filter.value),
                    Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal)
                )
            }
            FilterOp::Le => {
                matches!(
                    compare_values(val, &filter.value),
                    Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal)
                )
            }
            FilterOp::Contains => val
                .as_str()
                .map(|s| s.to_lowercase().contains(&filter.value.to_lowercase()))
                .unwrap_or(false),
            FilterOp::StartsWith => val
                .as_str()
                .map(|s| s.to_lowercase().starts_with(&filter.value.to_lowercase()))
                .unwrap_or(false),
            FilterOp::EndsWith => val
                .as_str()
                .map(|s| s.to_lowercase().ends_with(&filter.value.to_lowercase()))
                .unwrap_or(false),
        },
        None => false,
    }
}

fn get_nested_value<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            JsonValue::Object(obj) => {
                current = obj.get(part)?;
            }
            JsonValue::Array(arr) => {
                let index: usize = part.parse().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

fn compare_values(json_val: &JsonValue, filter_val: &str) -> Option<std::cmp::Ordering> {
    match json_val {
        JsonValue::Number(n) => {
            if let Ok(filter_num) = filter_val.parse::<f64>() {
                n.as_f64().map(|jn| {
                    jn.partial_cmp(&filter_num)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            } else {
                None
            }
        }
        JsonValue::String(s) => Some(s.cmp(&filter_val.to_string())),
        JsonValue::Bool(b) => {
            let filter_bool = filter_val.to_lowercase() == "true";
            Some(b.cmp(&filter_bool))
        }
        JsonValue::Null => {
            if filter_val.to_lowercase() == "null" {
                Some(std::cmp::Ordering::Equal)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Select specific fields from objects in an array
pub fn select_fields(value: &JsonValue, fields: &[String]) -> Result<JsonValue> {
    match value {
        JsonValue::Array(arr) => {
            let selected: Vec<JsonValue> = arr
                .iter()
                .map(|item| select_from_object(item, fields))
                .collect();
            Ok(JsonValue::Array(selected))
        }
        JsonValue::Object(_) => Ok(select_from_object(value, fields)),
        _ => bail!("Select can only be applied to objects or arrays of objects"),
    }
}

fn select_from_object(value: &JsonValue, fields: &[String]) -> JsonValue {
    if let JsonValue::Object(obj) = value {
        let mut new_obj = Map::new();
        for field in fields {
            if let Some(val) = obj.get(field) {
                new_obj.insert(field.clone(), val.clone());
            }
        }
        JsonValue::Object(new_obj)
    } else {
        value.clone()
    }
}

/// Get unique values from an array
pub fn unique(value: &JsonValue) -> Result<JsonValue> {
    let arr = value
        .as_array()
        .context("Unique can only be applied to arrays")?;

    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for item in arr {
        let key = serde_json::to_string(item).unwrap_or_default();
        if seen.insert(key) {
            result.push(item.clone());
        }
    }

    Ok(JsonValue::Array(result))
}

/// Count elements or occurrences
pub fn count(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Array(arr) => JsonValue::Number(arr.len().into()),
        JsonValue::Object(obj) => JsonValue::Number(obj.len().into()),
        JsonValue::String(s) => JsonValue::Number(s.len().into()),
        _ => JsonValue::Number(1.into()),
    }
}

/// Reverse array elements
pub fn reverse(value: &JsonValue) -> Result<JsonValue> {
    let arr = value
        .as_array()
        .context("Reverse can only be applied to arrays")?;
    let reversed: Vec<JsonValue> = arr.iter().rev().cloned().collect();
    Ok(JsonValue::Array(reversed))
}

/// Get first N elements
pub fn first(value: &JsonValue, n: usize) -> Result<JsonValue> {
    let arr = value
        .as_array()
        .context("First can only be applied to arrays")?;
    let taken: Vec<JsonValue> = arr.iter().take(n).cloned().collect();
    Ok(JsonValue::Array(taken))
}

/// Get last N elements
pub fn last(value: &JsonValue, n: usize) -> Result<JsonValue> {
    let arr = value
        .as_array()
        .context("Last can only be applied to arrays")?;
    let len = arr.len();
    let skip = len.saturating_sub(n);
    let taken: Vec<JsonValue> = arr.iter().skip(skip).cloned().collect();
    Ok(JsonValue::Array(taken))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonpath_query() {
        let data = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        let result = jsonpath_query(&data, "$.users[*].name").unwrap();
        assert_eq!(result, json!(["Alice", "Bob"]));
    }

    #[test]
    fn test_extract_keys() {
        let data = json!({"a": 1, "b": {"c": 2}});
        let keys = extract_keys(&data, false);
        assert!(keys.as_array().unwrap().contains(&json!("a")));
        assert!(keys.as_array().unwrap().contains(&json!("b")));
    }

    #[test]
    fn test_flatten() {
        let data = json!({"a": {"b": 1}});
        let flat = flatten(&data, ".");
        assert_eq!(flat, json!({"a.b": 1}));
    }

    #[test]
    fn test_sort_keys() {
        let data = json!({"c": 3, "a": 1, "b": 2});
        let sorted = sort_keys(&data);
        let keys: Vec<&String> = sorted.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_filter_array() {
        let data = json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]);

        let filtered = filter_array(&data, "age > 26").unwrap();
        assert_eq!(filtered.as_array().unwrap().len(), 1);
        assert_eq!(filtered[0]["name"], "Alice");
    }

    #[test]
    fn test_count() {
        let data = json!([1, 2, 3, 4, 5]);
        assert_eq!(count(&data), json!(5));
    }
}

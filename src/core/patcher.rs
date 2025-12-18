//! JSON Patch (RFC 6902) implementation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as JsonValue};

/// JSON Patch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum PatchOperation {
    Add { path: String, value: JsonValue },
    Remove { path: String },
    Replace { path: String, value: JsonValue },
    Move { from: String, path: String },
    Copy { from: String, path: String },
    Test { path: String, value: JsonValue },
}

/// Apply a JSON Patch to a document
pub fn apply_patch(doc: &JsonValue, patch: &[PatchOperation]) -> Result<JsonValue> {
    let mut result = doc.clone();

    for (i, op) in patch.iter().enumerate() {
        result = apply_operation(&result, op)
            .with_context(|| format!("Failed to apply patch operation {} ({:?})", i, op))?;
    }

    Ok(result)
}

fn apply_operation(doc: &JsonValue, op: &PatchOperation) -> Result<JsonValue> {
    match op {
        PatchOperation::Add { path, value } => add_value(doc, path, value),
        PatchOperation::Remove { path } => remove_value(doc, path),
        PatchOperation::Replace { path, value } => replace_value(doc, path, value),
        PatchOperation::Move { from, path } => move_value(doc, from, path),
        PatchOperation::Copy { from, path } => copy_value(doc, from, path),
        PatchOperation::Test { path, value } => test_value(doc, path, value),
    }
}

/// Parse JSON Pointer path into parts
fn parse_path(path: &str) -> Vec<String> {
    if path.is_empty() || path == "/" {
        return vec![];
    }

    path.trim_start_matches('/')
        .split('/')
        .map(|s| {
            // Unescape JSON Pointer encoding
            s.replace("~1", "/").replace("~0", "~")
        })
        .collect()
}

/// Get value at path
fn get_value<'a>(doc: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let parts = parse_path(path);
    let mut current = doc;

    for part in parts {
        match current {
            JsonValue::Object(obj) => {
                current = obj.get(&part)?;
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

/// Add value at path
fn add_value(doc: &JsonValue, path: &str, value: &JsonValue) -> Result<JsonValue> {
    if path.is_empty() {
        return Ok(value.clone());
    }

    let parts = parse_path(path);
    add_value_recursive(doc, &parts, value)
}

fn add_value_recursive(doc: &JsonValue, path: &[String], value: &JsonValue) -> Result<JsonValue> {
    if path.is_empty() {
        return Ok(value.clone());
    }

    let key = &path[0];

    match doc {
        JsonValue::Object(obj) => {
            let mut result = obj.clone();
            if path.len() == 1 {
                result.insert(key.clone(), value.clone());
            } else if let Some(existing) = obj.get(key) {
                result.insert(key.clone(), add_value_recursive(existing, &path[1..], value)?);
            } else {
                // Create path
                let new_obj = add_value_recursive(&JsonValue::Object(Map::new()), &path[1..], value)?;
                result.insert(key.clone(), new_obj);
            }
            Ok(JsonValue::Object(result))
        }
        JsonValue::Array(arr) => {
            let mut result = arr.clone();
            if key == "-" {
                // Append to array
                if path.len() == 1 {
                    result.push(value.clone());
                } else {
                    anyhow::bail!("Cannot navigate into '-' (append position)");
                }
            } else {
                let index: usize = key.parse().context("Invalid array index")?;
                if path.len() == 1 {
                    if index > arr.len() {
                        anyhow::bail!("Array index {} out of bounds", index);
                    }
                    result.insert(index, value.clone());
                } else if index < arr.len() {
                    result[index] = add_value_recursive(&arr[index], &path[1..], value)?;
                } else {
                    anyhow::bail!("Array index {} out of bounds", index);
                }
            }
            Ok(JsonValue::Array(result))
        }
        _ => {
            if path.len() == 1 {
                // Create object with key
                let mut obj = Map::new();
                obj.insert(key.clone(), value.clone());
                Ok(JsonValue::Object(obj))
            } else {
                anyhow::bail!("Cannot add to non-container at path");
            }
        }
    }
}

/// Remove value at path
fn remove_value(doc: &JsonValue, path: &str) -> Result<JsonValue> {
    if path.is_empty() {
        anyhow::bail!("Cannot remove root");
    }

    let parts = parse_path(path);
    remove_value_recursive(doc, &parts)
}

fn remove_value_recursive(doc: &JsonValue, path: &[String]) -> Result<JsonValue> {
    if path.is_empty() {
        anyhow::bail!("Cannot remove root");
    }

    let key = &path[0];

    match doc {
        JsonValue::Object(obj) => {
            let mut result = obj.clone();
            if path.len() == 1 {
                if result.remove(key).is_none() {
                    anyhow::bail!("Key '{}' not found", key);
                }
            } else if let Some(existing) = obj.get(key) {
                result.insert(key.clone(), remove_value_recursive(existing, &path[1..])?);
            } else {
                anyhow::bail!("Key '{}' not found", key);
            }
            Ok(JsonValue::Object(result))
        }
        JsonValue::Array(arr) => {
            let index: usize = key.parse().context("Invalid array index")?;
            if index >= arr.len() {
                anyhow::bail!("Array index {} out of bounds", index);
            }

            let mut result = arr.clone();
            if path.len() == 1 {
                result.remove(index);
            } else {
                result[index] = remove_value_recursive(&arr[index], &path[1..])?;
            }
            Ok(JsonValue::Array(result))
        }
        _ => anyhow::bail!("Cannot remove from non-container"),
    }
}

/// Replace value at path
fn replace_value(doc: &JsonValue, path: &str, value: &JsonValue) -> Result<JsonValue> {
    if path.is_empty() {
        return Ok(value.clone());
    }

    let parts = parse_path(path);
    replace_value_recursive(doc, &parts, value)
}

fn replace_value_recursive(
    doc: &JsonValue,
    path: &[String],
    value: &JsonValue,
) -> Result<JsonValue> {
    if path.is_empty() {
        return Ok(value.clone());
    }

    let key = &path[0];

    match doc {
        JsonValue::Object(obj) => {
            let mut result = obj.clone();
            if path.len() == 1 {
                if !obj.contains_key(key) {
                    anyhow::bail!("Key '{}' not found for replace", key);
                }
                result.insert(key.clone(), value.clone());
            } else if let Some(existing) = obj.get(key) {
                result.insert(
                    key.clone(),
                    replace_value_recursive(existing, &path[1..], value)?,
                );
            } else {
                anyhow::bail!("Key '{}' not found", key);
            }
            Ok(JsonValue::Object(result))
        }
        JsonValue::Array(arr) => {
            let index: usize = key.parse().context("Invalid array index")?;
            if index >= arr.len() {
                anyhow::bail!("Array index {} out of bounds", index);
            }

            let mut result = arr.clone();
            if path.len() == 1 {
                result[index] = value.clone();
            } else {
                result[index] = replace_value_recursive(&arr[index], &path[1..], value)?;
            }
            Ok(JsonValue::Array(result))
        }
        _ => anyhow::bail!("Cannot replace in non-container"),
    }
}

/// Move value from one path to another
fn move_value(doc: &JsonValue, from: &str, to: &str) -> Result<JsonValue> {
    let value = get_value(doc, from)
        .context(format!("Source path '{}' not found", from))?
        .clone();
    let without_source = remove_value(doc, from)?;
    add_value(&without_source, to, &value)
}

/// Copy value from one path to another
fn copy_value(doc: &JsonValue, from: &str, to: &str) -> Result<JsonValue> {
    let value = get_value(doc, from)
        .context(format!("Source path '{}' not found", from))?
        .clone();
    add_value(doc, to, &value)
}

/// Test that value at path equals expected value
fn test_value(doc: &JsonValue, path: &str, expected: &JsonValue) -> Result<JsonValue> {
    let actual = get_value(doc, path).context(format!("Path '{}' not found", path))?;

    if actual == expected {
        Ok(doc.clone())
    } else {
        anyhow::bail!(
            "Test failed at '{}': expected {}, got {}",
            path,
            expected,
            actual
        )
    }
}

/// Parse patch from JSON value
pub fn parse_patch(value: &JsonValue) -> Result<Vec<PatchOperation>> {
    let arr = value
        .as_array()
        .context("Patch must be an array of operations")?;

    arr.iter()
        .enumerate()
        .map(|(i, op)| {
            serde_json::from_value(op.clone())
                .with_context(|| format!("Invalid patch operation at index {}", i))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_add_operation() {
        let doc = json!({"foo": "bar"});
        let patch = vec![PatchOperation::Add {
            path: "/baz".to_string(),
            value: json!("qux"),
        }];

        let result = apply_patch(&doc, &patch).unwrap();
        assert_eq!(result["foo"], "bar");
        assert_eq!(result["baz"], "qux");
    }

    #[test]
    fn test_remove_operation() {
        let doc = json!({"foo": "bar", "baz": "qux"});
        let patch = vec![PatchOperation::Remove {
            path: "/baz".to_string(),
        }];

        let result = apply_patch(&doc, &patch).unwrap();
        assert_eq!(result["foo"], "bar");
        assert!(result.get("baz").is_none());
    }

    #[test]
    fn test_replace_operation() {
        let doc = json!({"foo": "bar"});
        let patch = vec![PatchOperation::Replace {
            path: "/foo".to_string(),
            value: json!("baz"),
        }];

        let result = apply_patch(&doc, &patch).unwrap();
        assert_eq!(result["foo"], "baz");
    }

    #[test]
    fn test_move_operation() {
        let doc = json!({"foo": {"bar": "baz"}});
        let patch = vec![PatchOperation::Move {
            from: "/foo/bar".to_string(),
            path: "/qux".to_string(),
        }];

        let result = apply_patch(&doc, &patch).unwrap();
        assert_eq!(result["qux"], "baz");
        assert!(result["foo"].get("bar").is_none());
    }

    #[test]
    fn test_test_operation() {
        let doc = json!({"foo": "bar"});
        let patch = vec![PatchOperation::Test {
            path: "/foo".to_string(),
            value: json!("bar"),
        }];

        let result = apply_patch(&doc, &patch);
        assert!(result.is_ok());

        let patch_fail = vec![PatchOperation::Test {
            path: "/foo".to_string(),
            value: json!("baz"),
        }];
        let result_fail = apply_patch(&doc, &patch_fail);
        assert!(result_fail.is_err());
    }
}


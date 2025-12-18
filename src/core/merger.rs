//! Merge engine for combining data structures

use anyhow::Result;
use serde_json::{Map, Value as JsonValue};

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeStrategy {
    /// Deep merge - recursively merge objects, later values win
    Deep,
    /// Shallow merge - only merge top level, later values win
    Shallow,
    /// Concat arrays instead of replacing
    ConcatArrays,
    /// Union arrays (unique values only)
    UnionArrays,
}

/// Merge two JSON values with the specified strategy
pub fn merge(base: &JsonValue, overlay: &JsonValue, strategy: MergeStrategy) -> Result<JsonValue> {
    match strategy {
        MergeStrategy::Deep => deep_merge(base, overlay),
        MergeStrategy::Shallow => shallow_merge(base, overlay),
        MergeStrategy::ConcatArrays => deep_merge_with_array_concat(base, overlay),
        MergeStrategy::UnionArrays => deep_merge_with_array_union(base, overlay),
    }
}

/// Deep merge two values
fn deep_merge(base: &JsonValue, overlay: &JsonValue) -> Result<JsonValue> {
    match (base, overlay) {
        (JsonValue::Object(base_obj), JsonValue::Object(overlay_obj)) => {
            let mut result = base_obj.clone();
            for (key, overlay_value) in overlay_obj {
                if let Some(base_value) = base_obj.get(key) {
                    result.insert(key.clone(), deep_merge(base_value, overlay_value)?);
                } else {
                    result.insert(key.clone(), overlay_value.clone());
                }
            }
            Ok(JsonValue::Object(result))
        }
        // For non-objects, overlay wins
        (_, overlay) => Ok(overlay.clone()),
    }
}

/// Shallow merge - only top level
fn shallow_merge(base: &JsonValue, overlay: &JsonValue) -> Result<JsonValue> {
    match (base, overlay) {
        (JsonValue::Object(base_obj), JsonValue::Object(overlay_obj)) => {
            let mut result = base_obj.clone();
            for (key, value) in overlay_obj {
                result.insert(key.clone(), value.clone());
            }
            Ok(JsonValue::Object(result))
        }
        (_, overlay) => Ok(overlay.clone()),
    }
}

/// Deep merge with array concatenation
fn deep_merge_with_array_concat(base: &JsonValue, overlay: &JsonValue) -> Result<JsonValue> {
    match (base, overlay) {
        (JsonValue::Object(base_obj), JsonValue::Object(overlay_obj)) => {
            let mut result = base_obj.clone();
            for (key, overlay_value) in overlay_obj {
                if let Some(base_value) = base_obj.get(key) {
                    result.insert(
                        key.clone(),
                        deep_merge_with_array_concat(base_value, overlay_value)?,
                    );
                } else {
                    result.insert(key.clone(), overlay_value.clone());
                }
            }
            Ok(JsonValue::Object(result))
        }
        (JsonValue::Array(base_arr), JsonValue::Array(overlay_arr)) => {
            let mut result = base_arr.clone();
            result.extend(overlay_arr.iter().cloned());
            Ok(JsonValue::Array(result))
        }
        (_, overlay) => Ok(overlay.clone()),
    }
}

/// Deep merge with array union (unique values)
fn deep_merge_with_array_union(base: &JsonValue, overlay: &JsonValue) -> Result<JsonValue> {
    match (base, overlay) {
        (JsonValue::Object(base_obj), JsonValue::Object(overlay_obj)) => {
            let mut result = base_obj.clone();
            for (key, overlay_value) in overlay_obj {
                if let Some(base_value) = base_obj.get(key) {
                    result.insert(
                        key.clone(),
                        deep_merge_with_array_union(base_value, overlay_value)?,
                    );
                } else {
                    result.insert(key.clone(), overlay_value.clone());
                }
            }
            Ok(JsonValue::Object(result))
        }
        (JsonValue::Array(base_arr), JsonValue::Array(overlay_arr)) => {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();

            for item in base_arr.iter().chain(overlay_arr.iter()) {
                let key = serde_json::to_string(item).unwrap_or_default();
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }
            Ok(JsonValue::Array(result))
        }
        (_, overlay) => Ok(overlay.clone()),
    }
}

/// Merge multiple values sequentially
pub fn merge_all(values: &[JsonValue], strategy: MergeStrategy) -> Result<JsonValue> {
    if values.is_empty() {
        return Ok(JsonValue::Null);
    }

    let mut result = values[0].clone();
    for value in values.iter().skip(1) {
        result = merge(&result, value, strategy)?;
    }
    Ok(result)
}

/// Merge with path - merge overlay at a specific path in base
pub fn merge_at_path(
    base: &JsonValue,
    overlay: &JsonValue,
    path: &str,
    strategy: MergeStrategy,
) -> Result<JsonValue> {
    if path.is_empty() || path == "$" {
        return merge(base, overlay, strategy);
    }

    let parts: Vec<&str> = path
        .trim_start_matches('$')
        .trim_start_matches('.')
        .split('.')
        .filter(|s| !s.is_empty())
        .collect();

    merge_at_path_recursive(base, overlay, &parts, strategy)
}

fn merge_at_path_recursive(
    base: &JsonValue,
    overlay: &JsonValue,
    path: &[&str],
    strategy: MergeStrategy,
) -> Result<JsonValue> {
    if path.is_empty() {
        return merge(base, overlay, strategy);
    }

    match base {
        JsonValue::Object(obj) => {
            let mut result = obj.clone();
            let key = path[0];

            if let Some(existing) = obj.get(key) {
                let merged = merge_at_path_recursive(existing, overlay, &path[1..], strategy)?;
                result.insert(key.to_string(), merged);
            } else if path.len() == 1 {
                result.insert(key.to_string(), overlay.clone());
            } else {
                // Create nested structure
                let nested = merge_at_path_recursive(&JsonValue::Object(Map::new()), overlay, &path[1..], strategy)?;
                result.insert(key.to_string(), nested);
            }

            Ok(JsonValue::Object(result))
        }
        JsonValue::Array(arr) => {
            // Try to parse as index
            if let Ok(index) = path[0].parse::<usize>() {
                let mut result = arr.clone();
                if index < arr.len() {
                    let merged =
                        merge_at_path_recursive(&arr[index], overlay, &path[1..], strategy)?;
                    result[index] = merged;
                }
                Ok(JsonValue::Array(result))
            } else {
                Ok(base.clone())
            }
        }
        _ => Ok(base.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge() {
        let base = json!({
            "a": 1,
            "b": {"c": 2, "d": 3}
        });
        let overlay = json!({
            "b": {"c": 4, "e": 5},
            "f": 6
        });

        let result = merge(&base, &overlay, MergeStrategy::Deep).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"]["c"], 4);
        assert_eq!(result["b"]["d"], 3);
        assert_eq!(result["b"]["e"], 5);
        assert_eq!(result["f"], 6);
    }

    #[test]
    fn test_shallow_merge() {
        let base = json!({"a": {"b": 1}});
        let overlay = json!({"a": {"c": 2}});

        let result = merge(&base, &overlay, MergeStrategy::Shallow).unwrap();
        // Shallow merge replaces the whole object
        assert_eq!(result["a"]["c"], 2);
        assert!(result["a"].get("b").is_none());
    }

    #[test]
    fn test_array_concat() {
        let base = json!({"items": [1, 2]});
        let overlay = json!({"items": [3, 4]});

        let result = merge(&base, &overlay, MergeStrategy::ConcatArrays).unwrap();
        assert_eq!(result["items"], json!([1, 2, 3, 4]));
    }

    #[test]
    fn test_array_union() {
        let base = json!({"items": [1, 2, 3]});
        let overlay = json!({"items": [2, 3, 4]});

        let result = merge(&base, &overlay, MergeStrategy::UnionArrays).unwrap();
        assert_eq!(result["items"], json!([1, 2, 3, 4]));
    }
}


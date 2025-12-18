//! Diff calculation engine for comparing data structures

use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value as JsonValue;
use similar::{ChangeTag, TextDiff};

use crate::core::converter;
use crate::formats::detect::Format;

/// Diff output format
#[derive(Debug, Clone, Copy)]
pub enum DiffFormat {
    /// Unified diff format
    Unified,
    /// Side-by-side comparison
    SideBySide,
    /// JSON patch format (RFC 6902)
    JsonPatch,
}

/// Calculate diff between two files/contents
pub fn diff(
    content1: &str,
    content2: &str,
    format1: Format,
    format2: Format,
    output_format: DiffFormat,
) -> Result<String> {
    // Normalize both to JSON for comparison
    let json1 = normalize_to_json(content1, format1)?;
    let json2 = normalize_to_json(content2, format2)?;

    match output_format {
        DiffFormat::Unified => unified_diff(&json1, &json2),
        DiffFormat::SideBySide => side_by_side_diff(&json1, &json2),
        DiffFormat::JsonPatch => json_patch_diff(&json1, &json2),
    }
}

fn normalize_to_json(content: &str, format: Format) -> Result<String> {
    if format == Format::Json {
        // Parse and re-serialize for consistent formatting
        let value: JsonValue = serde_json::from_str(content).context("Failed to parse JSON")?;
        serde_json::to_string_pretty(&value).context("Failed to serialize JSON")
    } else {
        // Convert to JSON
        converter::convert(content, format, Format::Json)
    }
}

fn unified_diff(text1: &str, text2: &str) -> Result<String> {
    let diff = TextDiff::from_lines(text1, text2);
    let mut output = String::new();

    // Header
    output.push_str(&format!("{}\n", "--- a".red()));
    output.push_str(&format!("{}\n", "+++ b".green()));

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            output.push_str(&format!("{}\n", "...".dimmed()));
        }

        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", "red"),
                    ChangeTag::Insert => ("+", "green"),
                    ChangeTag::Equal => (" ", "white"),
                };

                let line_content = change.value();

                let formatted = match style {
                    "red" => format!("{}{}", sign.red(), line_content.red()),
                    "green" => format!("{}{}", sign.green(), line_content.green()),
                    _ => format!("{}{}", sign.dimmed(), line_content),
                };

                output.push_str(&formatted);
                if !change.missing_newline() {
                    output.push('\n');
                }
            }
        }
    }

    Ok(output)
}

fn side_by_side_diff(text1: &str, text2: &str) -> Result<String> {
    let diff = TextDiff::from_lines(text1, text2);
    let mut output = String::new();

    let width = 40;
    let separator = " | ";

    // Header
    output.push_str(&format!(
        "{:^width$}{}{:^width$}\n",
        "Left".bold(),
        separator,
        "Right".bold(),
        width = width
    ));
    output.push_str(&format!("{}\n", "-".repeat(width * 2 + separator.len())));

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let line = change.value().trim_end();

            match change.tag() {
                ChangeTag::Delete => {
                    let left = truncate_or_pad(line, width);
                    output.push_str(&format!(
                        "{}{}{}\n",
                        left.red(),
                        separator.dimmed(),
                        " ".repeat(width)
                    ));
                }
                ChangeTag::Insert => {
                    let right = truncate_or_pad(line, width);
                    output.push_str(&format!(
                        "{}{}{}\n",
                        " ".repeat(width),
                        separator.dimmed(),
                        right.green()
                    ));
                }
                ChangeTag::Equal => {
                    let content = truncate_or_pad(line, width);
                    output.push_str(&format!(
                        "{}{}{}\n",
                        content.dimmed(),
                        separator.dimmed(),
                        content.dimmed()
                    ));
                }
            }
        }
    }

    Ok(output)
}

fn truncate_or_pad(s: &str, width: usize) -> String {
    if s.len() > width {
        format!("{}...", &s[..width - 3])
    } else {
        format!("{:width$}", s, width = width)
    }
}

fn json_patch_diff(text1: &str, text2: &str) -> Result<String> {
    let value1: JsonValue = serde_json::from_str(text1).context("Failed to parse JSON 1")?;
    let value2: JsonValue = serde_json::from_str(text2).context("Failed to parse JSON 2")?;

    let mut patches = Vec::new();
    generate_json_patches(&value1, &value2, "", &mut patches);

    let result = JsonValue::Array(patches);
    serde_json::to_string_pretty(&result).context("Failed to serialize patches")
}

fn generate_json_patches(old: &JsonValue, new: &JsonValue, path: &str, patches: &mut Vec<JsonValue>) {
    if old == new {
        return;
    }

    match (old, new) {
        (JsonValue::Object(old_obj), JsonValue::Object(new_obj)) => {
            // Check for removed keys
            for key in old_obj.keys() {
                if !new_obj.contains_key(key) {
                    patches.push(serde_json::json!({
                        "op": "remove",
                        "path": format!("{}/{}", path, escape_json_pointer(key))
                    }));
                }
            }

            // Check for added or modified keys
            for (key, new_val) in new_obj {
                let new_path = format!("{}/{}", path, escape_json_pointer(key));
                if let Some(old_val) = old_obj.get(key) {
                    generate_json_patches(old_val, new_val, &new_path, patches);
                } else {
                    patches.push(serde_json::json!({
                        "op": "add",
                        "path": new_path,
                        "value": new_val
                    }));
                }
            }
        }
        (JsonValue::Array(old_arr), JsonValue::Array(new_arr)) => {
            // Simple array diff - could be optimized with LCS
            let max_len = old_arr.len().max(new_arr.len());
            for i in 0..max_len {
                let item_path = format!("{}/{}", path, i);
                match (old_arr.get(i), new_arr.get(i)) {
                    (Some(old_val), Some(new_val)) => {
                        generate_json_patches(old_val, new_val, &item_path, patches);
                    }
                    (Some(_), None) => {
                        patches.push(serde_json::json!({
                            "op": "remove",
                            "path": item_path
                        }));
                    }
                    (None, Some(new_val)) => {
                        patches.push(serde_json::json!({
                            "op": "add",
                            "path": item_path,
                            "value": new_val
                        }));
                    }
                    (None, None) => {}
                }
            }
        }
        _ => {
            // Different types or different primitive values
            patches.push(serde_json::json!({
                "op": "replace",
                "path": if path.is_empty() { "/" } else { path },
                "value": new
            }));
        }
    }
}

fn escape_json_pointer(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

/// Check if two values are structurally equal (ignoring key order)
pub fn structural_equal(value1: &JsonValue, value2: &JsonValue) -> bool {
    match (value1, value2) {
        (JsonValue::Object(obj1), JsonValue::Object(obj2)) => {
            if obj1.len() != obj2.len() {
                return false;
            }
            obj1.iter()
                .all(|(k, v)| obj2.get(k).map(|v2| structural_equal(v, v2)).unwrap_or(false))
        }
        (JsonValue::Array(arr1), JsonValue::Array(arr2)) => {
            if arr1.len() != arr2.len() {
                return false;
            }
            arr1.iter()
                .zip(arr2.iter())
                .all(|(v1, v2)| structural_equal(v1, v2))
        }
        _ => value1 == value2,
    }
}

/// Generate a summary of differences
pub fn diff_summary(content1: &str, content2: &str, format1: Format, format2: Format) -> Result<String> {
    let json1 = normalize_to_json(content1, format1)?;
    let json2 = normalize_to_json(content2, format2)?;

    let value1: JsonValue = serde_json::from_str(&json1)?;
    let value2: JsonValue = serde_json::from_str(&json2)?;

    let mut added = 0;
    let mut removed = 0;
    let mut modified = 0;

    count_changes(&value1, &value2, &mut added, &mut removed, &mut modified);

    let mut output = String::new();

    if added == 0 && removed == 0 && modified == 0 {
        output.push_str(&format!("{}\n", "Files are identical".green()));
    } else {
        output.push_str(&format!("{}\n", "Differences found:".yellow()));
        if added > 0 {
            output.push_str(&format!("  {} {}\n", format!("+{}", added).green(), "additions"));
        }
        if removed > 0 {
            output.push_str(&format!("  {} {}\n", format!("-{}", removed).red(), "removals"));
        }
        if modified > 0 {
            output.push_str(&format!("  {} {}\n", format!("~{}", modified).yellow(), "modifications"));
        }
    }

    Ok(output)
}

fn count_changes(old: &JsonValue, new: &JsonValue, added: &mut usize, removed: &mut usize, modified: &mut usize) {
    match (old, new) {
        (JsonValue::Object(old_obj), JsonValue::Object(new_obj)) => {
            for key in old_obj.keys() {
                if !new_obj.contains_key(key) {
                    *removed += 1;
                }
            }
            for (key, new_val) in new_obj {
                if let Some(old_val) = old_obj.get(key) {
                    count_changes(old_val, new_val, added, removed, modified);
                } else {
                    *added += 1;
                }
            }
        }
        (JsonValue::Array(old_arr), JsonValue::Array(new_arr)) => {
            let old_len = old_arr.len();
            let new_len = new_arr.len();

            if new_len > old_len {
                *added += new_len - old_len;
            } else if old_len > new_len {
                *removed += old_len - new_len;
            }

            for (old_val, new_val) in old_arr.iter().zip(new_arr.iter()) {
                count_changes(old_val, new_val, added, removed, modified);
            }
        }
        _ if old != new => {
            *modified += 1;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_structural_equal() {
        let v1 = json!({"a": 1, "b": 2});
        let v2 = json!({"b": 2, "a": 1});
        assert!(structural_equal(&v1, &v2));

        let v3 = json!({"a": 1, "b": 3});
        assert!(!structural_equal(&v1, &v3));
    }

    #[test]
    fn test_json_patch() {
        let old = r#"{"name": "Alice"}"#;
        let new = r#"{"name": "Bob"}"#;
        let patch = json_patch_diff(old, new).unwrap();
        assert!(patch.contains("replace"));
    }
}


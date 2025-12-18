//! JSON Schema generation from data

use serde_json::{json, Map, Value as JsonValue};

/// Generate JSON Schema from a JSON value
pub fn generate_schema(value: &JsonValue) -> JsonValue {
    let mut schema = Map::new();
    schema.insert("$schema".to_string(), json!("https://json-schema.org/draft/2020-12/schema"));

    let type_schema = infer_type(value);
    for (k, v) in type_schema.as_object().unwrap() {
        schema.insert(k.clone(), v.clone());
    }

    JsonValue::Object(schema)
}

fn infer_type(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Null => json!({"type": "null"}),
        JsonValue::Bool(_) => json!({"type": "boolean"}),
        JsonValue::Number(n) => {
            if n.is_i64() {
                json!({"type": "integer"})
            } else {
                json!({"type": "number"})
            }
        }
        JsonValue::String(s) => infer_string_format(s),
        JsonValue::Array(arr) => infer_array_schema(arr),
        JsonValue::Object(obj) => infer_object_schema(obj),
    }
}

fn infer_string_format(s: &str) -> JsonValue {
    // Check for common string formats
    if is_email(s) {
        json!({"type": "string", "format": "email"})
    } else if is_uri(s) {
        json!({"type": "string", "format": "uri"})
    } else if is_date(s) {
        json!({"type": "string", "format": "date"})
    } else if is_datetime(s) {
        json!({"type": "string", "format": "date-time"})
    } else if is_uuid(s) {
        json!({"type": "string", "format": "uuid"})
    } else if is_ipv4(s) {
        json!({"type": "string", "format": "ipv4"})
    } else {
        json!({"type": "string"})
    }
}

fn is_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.')
}

fn is_uri(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
}

fn is_date(s: &str) -> bool {
    // Simple YYYY-MM-DD pattern
    if s.len() != 10 {
        return false;
    }
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

fn is_datetime(s: &str) -> bool {
    // ISO 8601 datetime pattern
    s.contains('T') && (s.ends_with('Z') || s.contains('+') || s.contains('-'))
        && s.len() >= 19
}

fn is_uuid(s: &str) -> bool {
    // UUID pattern: 8-4-4-4-12
    if s.len() != 36 {
        return false;
    }
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn is_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| p.parse::<u8>().is_ok())
}

fn infer_array_schema(arr: &[JsonValue]) -> JsonValue {
    if arr.is_empty() {
        return json!({"type": "array"});
    }

    // Check if all items have the same type
    let item_schemas: Vec<JsonValue> = arr.iter().map(infer_type).collect();

    // Try to merge schemas
    let merged = merge_schemas(&item_schemas);

    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("array"));
    schema.insert("items".to_string(), merged);

    JsonValue::Object(schema)
}

fn infer_object_schema(obj: &Map<String, JsonValue>) -> JsonValue {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));

    let mut properties = Map::new();
    let mut required = Vec::new();

    for (key, value) in obj {
        properties.insert(key.clone(), infer_type(value));

        // Assume all fields are required (from a single sample)
        if !value.is_null() {
            required.push(json!(key));
        }
    }

    if !properties.is_empty() {
        schema.insert("properties".to_string(), JsonValue::Object(properties));
    }

    if !required.is_empty() {
        schema.insert("required".to_string(), JsonValue::Array(required));
    }

    JsonValue::Object(schema)
}

fn merge_schemas(schemas: &[JsonValue]) -> JsonValue {
    if schemas.is_empty() {
        return json!({});
    }

    if schemas.len() == 1 {
        return schemas[0].clone();
    }

    // Check if all schemas are the same type
    let types: Vec<&str> = schemas
        .iter()
        .filter_map(|s| s.get("type").and_then(|t| t.as_str()))
        .collect();

    let all_same_type = types.windows(2).all(|w| w[0] == w[1]);

    if all_same_type && !types.is_empty() {
        let base_type = types[0];

        if base_type == "object" {
            // Merge object schemas
            return merge_object_schemas(schemas);
        } else {
            // For primitives, just return the first schema
            return schemas[0].clone();
        }
    }

    // Different types - use anyOf
    let unique_schemas: Vec<JsonValue> = schemas
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if unique_schemas.len() == 1 {
        return unique_schemas[0].clone();
    }

    json!({"anyOf": unique_schemas})
}

fn merge_object_schemas(schemas: &[JsonValue]) -> JsonValue {
    let mut all_properties: std::collections::HashMap<String, Vec<JsonValue>> =
        std::collections::HashMap::new();
    let mut all_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

    for schema in schemas {
        if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in props {
                all_keys.insert(key.clone());
                all_properties
                    .entry(key.clone())
                    .or_default()
                    .push(value.clone());
            }
        }
    }

    let mut merged_properties = Map::new();
    let mut required = Vec::new();

    for key in &all_keys {
        if let Some(prop_schemas) = all_properties.get(key) {
            let merged = merge_schemas(prop_schemas);
            merged_properties.insert(key.clone(), merged);

            // Only required if present in all schemas
            if prop_schemas.len() == schemas.len() {
                required.push(json!(key));
            }
        }
    }

    let mut result = Map::new();
    result.insert("type".to_string(), json!("object"));

    if !merged_properties.is_empty() {
        result.insert(
            "properties".to_string(),
            JsonValue::Object(merged_properties),
        );
    }

    if !required.is_empty() {
        result.insert("required".to_string(), JsonValue::Array(required));
    }

    JsonValue::Object(result)
}

/// Generate TypeScript interface from JSON Schema
pub fn schema_to_typescript(schema: &JsonValue, name: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!("interface {} {{\n", name));

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        let required: Vec<&str> = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        for (key, prop_schema) in properties {
            let ts_type = json_schema_to_ts_type(prop_schema);
            let optional = if required.contains(&key.as_str()) {
                ""
            } else {
                "?"
            };
            output.push_str(&format!("  {}{}: {};\n", key, optional, ts_type));
        }
    }

    output.push_str("}\n");
    output
}

fn json_schema_to_ts_type(schema: &JsonValue) -> String {
    if let Some(any_of) = schema.get("anyOf").and_then(|a| a.as_array()) {
        let types: Vec<String> = any_of.iter().map(json_schema_to_ts_type).collect();
        return types.join(" | ");
    }

    let type_str = schema.get("type").and_then(|t| t.as_str()).unwrap_or("any");

    match type_str {
        "string" => "string".to_string(),
        "number" | "integer" => "number".to_string(),
        "boolean" => "boolean".to_string(),
        "null" => "null".to_string(),
        "array" => {
            if let Some(items) = schema.get("items") {
                format!("{}[]", json_schema_to_ts_type(items))
            } else {
                "any[]".to_string()
            }
        }
        "object" => {
            if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                let props: Vec<String> = properties
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, json_schema_to_ts_type(v)))
                    .collect();
                format!("{{ {} }}", props.join("; "))
            } else {
                "object".to_string()
            }
        }
        _ => "any".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_schema_primitive() {
        let value = json!(42);
        let schema = generate_schema(&value);
        assert_eq!(schema.get("type").unwrap(), "integer");
    }

    #[test]
    fn test_generate_schema_object() {
        let value = json!({"name": "Alice", "age": 30});
        let schema = generate_schema(&value);
        assert_eq!(schema.get("type").unwrap(), "object");
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_generate_schema_array() {
        let value = json!([1, 2, 3]);
        let schema = generate_schema(&value);
        assert_eq!(schema.get("type").unwrap(), "array");
    }

    #[test]
    fn test_string_format_detection() {
        let email = infer_string_format("test@example.com");
        assert_eq!(email.get("format").unwrap(), "email");

        let uri = infer_string_format("https://example.com");
        assert_eq!(uri.get("format").unwrap(), "uri");

        let date = infer_string_format("2024-01-15");
        assert_eq!(date.get("format").unwrap(), "date");
    }
}


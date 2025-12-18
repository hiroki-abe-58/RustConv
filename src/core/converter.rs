//! Format conversion engine
//!
//! Provides conversion between all supported formats using serde_json::Value as
//! the intermediate representation.

use anyhow::{bail, Context, Result};
use serde_json::Value as JsonValue;

use crate::formats::detect::Format;
use crate::formats::{
    csv as csv_format, json as json_format, toml as toml_format, yaml as yaml_format,
};

/// Convert content from one format to another
pub fn convert(content: &str, from: Format, to: Format) -> Result<String> {
    if from == to {
        // Same format, just return formatted version
        return format_content(content, to);
    }

    // Convert to intermediate JSON Value
    let value = parse_to_json_value(content, from)?;

    // Convert from JSON Value to target format
    json_value_to_format(&value, to)
}

/// Parse content into serde_json::Value (intermediate representation)
fn parse_to_json_value(content: &str, format: Format) -> Result<JsonValue> {
    match format {
        Format::Json => serde_json::from_str(content).context("Failed to parse JSON"),
        Format::Yaml => {
            let yaml_value: serde_yaml::Value =
                serde_yaml::from_str(content).context("Failed to parse YAML")?;
            yaml_to_json_value(yaml_value)
        }
        Format::Toml => {
            let toml_value: toml::Value = content.parse().context("Failed to parse TOML")?;
            toml_to_json_value(toml_value)
        }
        Format::Csv => csv_to_json_value(content),
        Format::Xml => xml_to_json_value(content),
    }
}

/// Convert serde_json::Value to target format string
fn json_value_to_format(value: &JsonValue, format: Format) -> Result<String> {
    match format {
        Format::Json => serde_json::to_string_pretty(value).context("Failed to serialize JSON"),
        Format::Yaml => serde_yaml::to_string(value).context("Failed to serialize YAML"),
        Format::Toml => {
            let toml_value = json_to_toml_value(value)?;
            toml::to_string_pretty(&toml_value).context("Failed to serialize TOML")
        }
        Format::Csv => json_to_csv(value),
        Format::Xml => json_to_xml(value),
    }
}

/// Format content in same format (just pretty print)
fn format_content(content: &str, format: Format) -> Result<String> {
    match format {
        Format::Json => {
            let value = json_format::parse(content)?;
            json_format::to_pretty(&value)
        }
        Format::Yaml => {
            let value = yaml_format::parse(content)?;
            yaml_format::to_pretty(&value)
        }
        Format::Toml => {
            let value = toml_format::parse(content)?;
            toml_format::to_pretty(&value)
        }
        Format::Csv => {
            let data = csv_format::parse(content, true)?;
            csv_format::to_csv(&data)
        }
        Format::Xml => crate::formats::xml::to_pretty(content),
    }
}

// ============================================================================
// YAML <-> JSON conversion
// ============================================================================

fn yaml_to_json_value(yaml: serde_yaml::Value) -> Result<JsonValue> {
    match yaml {
        serde_yaml::Value::Null => Ok(JsonValue::Null),
        serde_yaml::Value::Bool(b) => Ok(JsonValue::Bool(b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(JsonValue::Number(i.into()))
            } else if let Some(f) = n.as_f64() {
                Ok(serde_json::Number::from_f64(f)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null))
            } else {
                Ok(JsonValue::Null)
            }
        }
        serde_yaml::Value::String(s) => Ok(JsonValue::String(s)),
        serde_yaml::Value::Sequence(arr) => {
            let json_arr: Result<Vec<JsonValue>> =
                arr.into_iter().map(yaml_to_json_value).collect();
            Ok(JsonValue::Array(json_arr?))
        }
        serde_yaml::Value::Mapping(map) => {
            let mut json_obj = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s,
                    other => serde_yaml::to_string(&other)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                };
                json_obj.insert(key, yaml_to_json_value(v)?);
            }
            Ok(JsonValue::Object(json_obj))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json_value(tagged.value),
    }
}

// ============================================================================
// TOML <-> JSON conversion
// ============================================================================

fn toml_to_json_value(toml: toml::Value) -> Result<JsonValue> {
    match toml {
        toml::Value::String(s) => Ok(JsonValue::String(s)),
        toml::Value::Integer(i) => Ok(JsonValue::Number(i.into())),
        toml::Value::Float(f) => Ok(serde_json::Number::from_f64(f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)),
        toml::Value::Boolean(b) => Ok(JsonValue::Bool(b)),
        toml::Value::Datetime(dt) => Ok(JsonValue::String(dt.to_string())),
        toml::Value::Array(arr) => {
            let json_arr: Result<Vec<JsonValue>> =
                arr.into_iter().map(toml_to_json_value).collect();
            Ok(JsonValue::Array(json_arr?))
        }
        toml::Value::Table(table) => {
            let mut json_obj = serde_json::Map::new();
            for (k, v) in table {
                json_obj.insert(k, toml_to_json_value(v)?);
            }
            Ok(JsonValue::Object(json_obj))
        }
    }
}

fn json_to_toml_value(json: &JsonValue) -> Result<toml::Value> {
    match json {
        JsonValue::Null => {
            // TOML doesn't have null, convert to empty string
            Ok(toml::Value::String(String::new()))
        }
        JsonValue::Bool(b) => Ok(toml::Value::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                bail!("Invalid number in JSON")
            }
        }
        JsonValue::String(s) => Ok(toml::Value::String(s.clone())),
        JsonValue::Array(arr) => {
            let toml_arr: Result<Vec<toml::Value>> = arr.iter().map(json_to_toml_value).collect();
            Ok(toml::Value::Array(toml_arr?))
        }
        JsonValue::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml_value(v)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

// ============================================================================
// CSV <-> JSON conversion
// ============================================================================

fn csv_to_json_value(content: &str) -> Result<JsonValue> {
    let data = csv_format::parse(content, true)?;

    let headers = data
        .headers
        .as_ref()
        .context("CSV must have headers for JSON conversion")?;

    let mut records = Vec::new();

    for row in &data.rows {
        let mut obj = serde_json::Map::new();
        for (i, cell) in row.iter().enumerate() {
            let key = headers
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("column_{}", i));

            // Try to parse as number or boolean
            let value = if let Ok(n) = cell.parse::<i64>() {
                JsonValue::Number(n.into())
            } else if let Ok(f) = cell.parse::<f64>() {
                serde_json::Number::from_f64(f)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::String(cell.clone()))
            } else if cell.eq_ignore_ascii_case("true") {
                JsonValue::Bool(true)
            } else if cell.eq_ignore_ascii_case("false") {
                JsonValue::Bool(false)
            } else if cell.is_empty() || cell.eq_ignore_ascii_case("null") {
                JsonValue::Null
            } else {
                JsonValue::String(cell.clone())
            };

            obj.insert(key, value);
        }
        records.push(JsonValue::Object(obj));
    }

    Ok(JsonValue::Array(records))
}

fn json_to_csv(value: &JsonValue) -> Result<String> {
    let array = value
        .as_array()
        .context("JSON must be an array for CSV conversion")?;

    if array.is_empty() {
        return Ok(String::new());
    }

    // Collect all keys from all objects to handle inconsistent schemas
    let mut all_keys = Vec::new();
    let mut key_set = std::collections::HashSet::new();

    for item in array {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                if key_set.insert(key.clone()) {
                    all_keys.push(key.clone());
                }
            }
        }
    }

    if all_keys.is_empty() {
        // Array of primitives - single column
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.write_record(["value"])?;
        for item in array {
            writer.write_record([json_value_to_string(item)])?;
        }
        let bytes = writer.into_inner()?;
        return String::from_utf8(bytes).context("Invalid UTF-8 in CSV output");
    }

    let mut writer = csv::Writer::from_writer(Vec::new());

    // Write headers
    writer.write_record(&all_keys)?;

    // Write data rows
    for item in array {
        let row: Vec<String> = all_keys
            .iter()
            .map(|key| item.get(key).map(json_value_to_string).unwrap_or_default())
            .collect();
        writer.write_record(&row)?;
    }

    let bytes = writer.into_inner()?;
    String::from_utf8(bytes).context("Invalid UTF-8 in CSV output")
}

fn json_value_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => s.clone(),
        JsonValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_value_to_string).collect();
            items.join(";")
        }
        JsonValue::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

// ============================================================================
// XML <-> JSON conversion
// ============================================================================

fn xml_to_json_value(content: &str) -> Result<JsonValue> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<(String, serde_json::Map<String, JsonValue>)> = Vec::new();
    let mut root: Option<JsonValue> = None;
    let mut current_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut attrs = serde_json::Map::new();

                // Parse attributes
                for attr in e.attributes().flatten() {
                    let key = format!("@{}", String::from_utf8_lossy(attr.key.as_ref()));
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    attrs.insert(key, JsonValue::String(value));
                }

                stack.push((name, attrs));
                current_text.clear();
            }
            Ok(Event::End(_)) => {
                if let Some((name, mut attrs)) = stack.pop() {
                    // Add text content if present
                    let trimmed_text = current_text.trim();
                    if !trimmed_text.is_empty() {
                        if attrs.is_empty() {
                            // Just text content, use string value
                            let value = parse_xml_text_value(trimmed_text);
                            if let Some((_, parent_attrs)) = stack.last_mut() {
                                add_to_xml_object(parent_attrs, &name, value);
                            } else {
                                let mut obj = serde_json::Map::new();
                                obj.insert(name, value);
                                root = Some(JsonValue::Object(obj));
                            }
                        } else {
                            // Has attributes, add text as #text
                            attrs.insert("#text".to_string(), parse_xml_text_value(trimmed_text));
                            let value = JsonValue::Object(attrs);
                            if let Some((_, parent_attrs)) = stack.last_mut() {
                                add_to_xml_object(parent_attrs, &name, value);
                            } else {
                                let mut obj = serde_json::Map::new();
                                obj.insert(name, value);
                                root = Some(JsonValue::Object(obj));
                            }
                        }
                    } else if !attrs.is_empty() {
                        let value = JsonValue::Object(attrs);
                        if let Some((_, parent_attrs)) = stack.last_mut() {
                            add_to_xml_object(parent_attrs, &name, value);
                        } else {
                            let mut obj = serde_json::Map::new();
                            obj.insert(name, value);
                            root = Some(JsonValue::Object(obj));
                        }
                    } else {
                        // Empty element
                        if let Some((_, parent_attrs)) = stack.last_mut() {
                            add_to_xml_object(parent_attrs, &name, JsonValue::Null);
                        } else {
                            let mut obj = serde_json::Map::new();
                            obj.insert(name, JsonValue::Null);
                            root = Some(JsonValue::Object(obj));
                        }
                    }
                    current_text.clear();
                }
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut attrs = serde_json::Map::new();

                for attr in e.attributes().flatten() {
                    let key = format!("@{}", String::from_utf8_lossy(attr.key.as_ref()));
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    attrs.insert(key, JsonValue::String(value));
                }

                let value = if attrs.is_empty() {
                    JsonValue::Null
                } else {
                    JsonValue::Object(attrs)
                };

                if let Some((_, parent_attrs)) = stack.last_mut() {
                    add_to_xml_object(parent_attrs, &name, value);
                } else {
                    let mut obj = serde_json::Map::new();
                    obj.insert(name, value);
                    root = Some(JsonValue::Object(obj));
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default();
                current_text.push_str(&text);
            }
            Ok(Event::CData(e)) => {
                let text = String::from_utf8_lossy(&e);
                current_text.push_str(&text);
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => bail!("XML parse error: {}", e),
        }
    }

    root.context("Empty XML document")
}

fn add_to_xml_object(obj: &mut serde_json::Map<String, JsonValue>, key: &str, value: JsonValue) {
    if let Some(existing) = obj.get_mut(key) {
        // Key already exists, convert to array or append to existing array
        match existing {
            JsonValue::Array(arr) => {
                arr.push(value);
            }
            _ => {
                let old = existing.take();
                *existing = JsonValue::Array(vec![old, value]);
            }
        }
    } else {
        obj.insert(key.to_string(), value);
    }
}

fn parse_xml_text_value(text: &str) -> JsonValue {
    // Try to parse as number or boolean
    if let Ok(n) = text.parse::<i64>() {
        JsonValue::Number(n.into())
    } else if let Ok(f) = text.parse::<f64>() {
        serde_json::Number::from_f64(f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::String(text.to_string()))
    } else if text.eq_ignore_ascii_case("true") {
        JsonValue::Bool(true)
    } else if text.eq_ignore_ascii_case("false") {
        JsonValue::Bool(false)
    } else if text.eq_ignore_ascii_case("null") {
        JsonValue::Null
    } else {
        JsonValue::String(text.to_string())
    }
}

fn json_to_xml(value: &JsonValue) -> Result<String> {
    let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

    match value {
        JsonValue::Object(obj) => {
            if obj.len() == 1 {
                // Single root element
                let (key, val) = obj.iter().next().unwrap();
                json_to_xml_element(&mut output, key, val, 0)?;
            } else {
                // Wrap in root element
                output.push_str("<root>\n");
                for (key, val) in obj {
                    json_to_xml_element(&mut output, key, val, 1)?;
                }
                output.push_str("</root>");
            }
        }
        JsonValue::Array(arr) => {
            output.push_str("<root>\n");
            for item in arr {
                json_to_xml_element(&mut output, "item", item, 1)?;
            }
            output.push_str("</root>");
        }
        _ => {
            output.push_str("<root>");
            output.push_str(&escape_xml(&json_value_to_string(value)));
            output.push_str("</root>");
        }
    }

    Ok(output)
}

fn json_to_xml_element(
    output: &mut String,
    tag: &str,
    value: &JsonValue,
    indent: usize,
) -> Result<()> {
    let indent_str = "  ".repeat(indent);

    // Skip attribute keys when processing as elements
    if tag.starts_with('@') {
        return Ok(());
    }

    match value {
        JsonValue::Null => {
            output.push_str(&format!("{}<{}/>\n", indent_str, tag));
        }
        JsonValue::Bool(b) => {
            output.push_str(&format!("{}<{}>{}</{}>\n", indent_str, tag, b, tag));
        }
        JsonValue::Number(n) => {
            output.push_str(&format!("{}<{}>{}</{}>\n", indent_str, tag, n, tag));
        }
        JsonValue::String(s) => {
            output.push_str(&format!(
                "{}<{}>{}</{}>\n",
                indent_str,
                tag,
                escape_xml(s),
                tag
            ));
        }
        JsonValue::Array(arr) => {
            for item in arr {
                json_to_xml_element(output, tag, item, indent)?;
            }
        }
        JsonValue::Object(obj) => {
            // Collect attributes and children
            let mut attrs = String::new();
            let mut children = Vec::new();
            let mut text_content = None;

            for (key, val) in obj {
                if let Some(attr_name) = key.strip_prefix('@') {
                    // Attribute
                    if let JsonValue::String(s) = val {
                        attrs.push_str(&format!(" {}=\"{}\"", attr_name, escape_xml_attr(s)));
                    } else {
                        attrs.push_str(&format!(
                            " {}=\"{}\"",
                            attr_name,
                            json_value_to_string(val)
                        ));
                    }
                } else if key == "#text" {
                    // Text content
                    text_content = Some(json_value_to_string(val));
                } else {
                    children.push((key.clone(), val.clone()));
                }
            }

            if children.is_empty() {
                if let Some(text) = text_content {
                    output.push_str(&format!(
                        "{}<{}{}>{}</{}>\n",
                        indent_str,
                        tag,
                        attrs,
                        escape_xml(&text),
                        tag
                    ));
                } else {
                    output.push_str(&format!("{}<{}{}/>\n", indent_str, tag, attrs));
                }
            } else {
                output.push_str(&format!("{}<{}{}>\n", indent_str, tag, attrs));
                if let Some(text) = text_content {
                    output.push_str(&format!("{}  {}\n", indent_str, escape_xml(&text)));
                }
                for (key, val) in children {
                    json_to_xml_element(output, &key, &val, indent + 1)?;
                }
                output.push_str(&format!("{}</{}>\n", indent_str, tag));
            }
        }
    }

    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_xml_attr(s: &str) -> String {
    escape_xml(s).replace('"', "&quot;").replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_yaml() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result = convert(json, Format::Json, Format::Yaml).unwrap();
        assert!(result.contains("name:"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_yaml_to_json() {
        let yaml = "name: test\nvalue: 42";
        let result = convert(yaml, Format::Yaml, Format::Json).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
    }

    #[test]
    fn test_json_to_toml() {
        let json = r#"{"section": {"key": "value"}}"#;
        let result = convert(json, Format::Json, Format::Toml).unwrap();
        assert!(result.contains("[section]"));
        assert!(result.contains("key"));
    }

    #[test]
    fn test_json_to_csv() {
        let json = r#"[{"name": "a", "value": 1}, {"name": "b", "value": 2}]"#;
        let result = convert(json, Format::Json, Format::Csv).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_csv_to_json() {
        let csv = "name,value\na,1\nb,2";
        let result = convert(csv, Format::Csv, Format::Json).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"a\""));
    }
}

//! Format auto-detection

use std::path::Path;

/// Supported data formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
    Toml,
    Csv,
    Xml,
}

impl Format {
    /// Get format name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Toml => "toml",
            Format::Csv => "csv",
            Format::Xml => "xml",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detect format from file extension
pub fn detect_from_extension(path: &Path) -> Option<Format> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "json" => Some(Format::Json),
        "yaml" | "yml" => Some(Format::Yaml),
        "toml" => Some(Format::Toml),
        "csv" | "tsv" => Some(Format::Csv),
        "xml" | "xhtml" | "svg" | "xsd" | "xsl" => Some(Format::Xml),
        _ => None,
    }
}

/// Detect format from content by analyzing the structure
pub fn detect_from_content(content: &str) -> Option<Format> {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Check for XML (starts with < or XML declaration)
    if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
        // Verify it looks like valid XML structure
        if trimmed.contains("</") || trimmed.contains("/>") {
            return Some(Format::Xml);
        }
    }

    // Check for JSON (starts with { or [)
    let first_char = trimmed.chars().next()?;
    if first_char == '{' || first_char == '[' {
        // Try to parse as JSON to verify
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return Some(Format::Json);
        }
    }

    // Check for TOML characteristics
    // TOML typically has [section] headers or key = "value" patterns
    if is_likely_toml(trimmed) {
        return Some(Format::Toml);
    }

    // Check for CSV (contains commas and consistent column count)
    if is_likely_csv(trimmed) {
        return Some(Format::Csv);
    }

    // Check for YAML (has : with proper spacing, or starts with ---)
    if is_likely_yaml(trimmed) {
        return Some(Format::Yaml);
    }

    None
}

/// Detect format using both file path and content
pub fn detect(path: Option<&Path>, content: &str) -> Option<Format> {
    // First try to detect from file extension
    if let Some(p) = path {
        if let Some(format) = detect_from_extension(p) {
            return Some(format);
        }
    }

    // Fall back to content-based detection
    detect_from_content(content)
}

fn is_likely_toml(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();

    for line in &lines {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // TOML section headers: [section] or [[array]]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return true;
        }

        // TOML key-value: key = value
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            // TOML keys are bare (alphanumeric + _) or quoted
            if !key.is_empty()
                && (key
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                    || (key.starts_with('"') && key.ends_with('"')))
            {
                return true;
            }
        }
    }

    false
}

fn is_likely_csv(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.len() < 2 {
        return false;
    }

    // Count delimiters (comma, tab, semicolon)
    let delimiters = [',', '\t', ';'];

    for delimiter in delimiters {
        let counts: Vec<usize> = lines.iter().map(|l| l.matches(delimiter).count()).collect();

        // Check if all lines have the same number of delimiters
        if counts.iter().all(|&c| c > 0 && c == counts[0]) {
            return true;
        }
    }

    false
}

fn is_likely_yaml(content: &str) -> bool {
    let trimmed = content.trim();

    // YAML document separator
    if trimmed.starts_with("---") {
        return true;
    }

    for line in content.lines() {
        let trimmed_line = line.trim();

        // Skip empty lines and comments
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        // YAML list item
        if trimmed_line.starts_with("- ") {
            return true;
        }

        // YAML key: value pattern
        if let Some(colon_pos) = trimmed_line.find(':') {
            let after_colon = &trimmed_line[colon_pos + 1..];
            // In YAML, colon is followed by space or newline
            if after_colon.is_empty() || after_colon.starts_with(' ') {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_json() {
        assert_eq!(
            detect_from_content(r#"{"key": "value"}"#),
            Some(Format::Json)
        );
        assert_eq!(detect_from_content(r#"[1, 2, 3]"#), Some(Format::Json));
    }

    #[test]
    fn test_detect_yaml() {
        assert_eq!(detect_from_content("key: value"), Some(Format::Yaml));
        assert_eq!(detect_from_content("---\nkey: value"), Some(Format::Yaml));
    }

    #[test]
    fn test_detect_toml() {
        assert_eq!(
            detect_from_content("[section]\nkey = \"value\""),
            Some(Format::Toml)
        );
    }

    #[test]
    fn test_detect_xml() {
        assert_eq!(
            detect_from_content("<?xml version=\"1.0\"?><root/>"),
            Some(Format::Xml)
        );
        assert_eq!(detect_from_content("<root></root>"), Some(Format::Xml));
    }

    #[test]
    fn test_detect_csv() {
        assert_eq!(
            detect_from_content("a,b,c\n1,2,3\n4,5,6"),
            Some(Format::Csv)
        );
    }

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(
            detect_from_extension(Path::new("test.json")),
            Some(Format::Json)
        );
        assert_eq!(
            detect_from_extension(Path::new("test.yaml")),
            Some(Format::Yaml)
        );
        assert_eq!(
            detect_from_extension(Path::new("test.yml")),
            Some(Format::Yaml)
        );
        assert_eq!(
            detect_from_extension(Path::new("test.toml")),
            Some(Format::Toml)
        );
        assert_eq!(
            detect_from_extension(Path::new("test.csv")),
            Some(Format::Csv)
        );
        assert_eq!(
            detect_from_extension(Path::new("test.xml")),
            Some(Format::Xml)
        );
    }
}

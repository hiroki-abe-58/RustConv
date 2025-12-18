//! Syntax highlighting for various data formats

use colored::Colorize;

/// Highlight JSON output with colors
pub fn highlight_json(json: &str) -> String {
    let mut result = String::new();
    let mut chars = json.chars().peekable();
    let mut in_string = false;
    let mut is_key = false;
    let mut current_token = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !in_string => {
                in_string = true;
                is_key = false;
                current_token.clear();
                current_token.push(ch);
            }
            '"' if in_string => {
                current_token.push(ch);
                in_string = false;

                // Check if this is a key (followed by ':')
                let mut temp_chars = chars.clone();
                while let Some(&next) = temp_chars.peek() {
                    if next.is_whitespace() {
                        temp_chars.next();
                    } else {
                        is_key = next == ':';
                        break;
                    }
                }

                if is_key {
                    result.push_str(&current_token.cyan().to_string());
                } else {
                    result.push_str(&current_token.green().to_string());
                }
                current_token.clear();
            }
            _ if in_string => {
                current_token.push(ch);
            }
            ':' => {
                result.push_str(&":".white().to_string());
            }
            '{' | '}' | '[' | ']' => {
                result.push_str(&ch.to_string().yellow().to_string());
            }
            ',' => {
                result.push_str(&",".white().to_string());
            }
            _ if ch.is_ascii_digit() || ch == '-' || ch == '.' => {
                current_token.push(ch);
                // Collect the entire number
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit()
                        || next == '.'
                        || next == 'e'
                        || next == 'E'
                        || next == '+'
                        || next == '-'
                    {
                        current_token.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                // Check if it's actually a number
                if current_token.parse::<f64>().is_ok() {
                    result.push_str(&current_token.magenta().to_string());
                } else {
                    result.push_str(&current_token);
                }
                current_token.clear();
            }
            't' | 'f' => {
                // Check for true/false
                current_token.push(ch);
                let expected = if ch == 't' { "rue" } else { "alse" };
                for expected_ch in expected.chars() {
                    if let Some(&next) = chars.peek() {
                        if next == expected_ch {
                            current_token.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }
                if current_token == "true" || current_token == "false" {
                    result.push_str(&current_token.blue().bold().to_string());
                } else {
                    result.push_str(&current_token);
                }
                current_token.clear();
            }
            'n' => {
                // Check for null
                current_token.push(ch);
                for expected_ch in "ull".chars() {
                    if let Some(&next) = chars.peek() {
                        if next == expected_ch {
                            current_token.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }
                if current_token == "null" {
                    result.push_str(&current_token.red().to_string());
                } else {
                    result.push_str(&current_token);
                }
                current_token.clear();
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

/// Highlight YAML output with colors
pub fn highlight_yaml(yaml: &str) -> String {
    let mut result = String::new();

    for line in yaml.lines() {
        let highlighted_line = highlight_yaml_line(line);
        result.push_str(&highlighted_line);
        result.push('\n');
    }

    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

fn highlight_yaml_line(line: &str) -> String {
    // Handle comments
    if line.trim_start().starts_with('#') {
        return line.dimmed().to_string();
    }

    // Handle document markers
    if line == "---" || line == "..." {
        return line.yellow().to_string();
    }

    // Check for key-value pairs
    if let Some(colon_pos) = find_yaml_colon(line) {
        let (key_part, rest) = line.split_at(colon_pos);
        let colon_and_value = rest;

        let mut result = String::new();

        // Preserve leading whitespace and handle list markers
        let trimmed = key_part.trim_start();
        let indent = &key_part[..key_part.len() - trimmed.len()];
        result.push_str(indent);

        if let Some(rest) = trimmed.strip_prefix("- ") {
            result.push_str(&"- ".yellow().to_string());
            result.push_str(&rest.cyan().to_string());
        } else if let Some(rest) = trimmed.strip_prefix('-') {
            result.push_str(&"-".yellow().to_string());
            result.push_str(&rest.cyan().to_string());
        } else {
            result.push_str(&trimmed.cyan().to_string());
        }

        // Add colon
        result.push_str(&":".white().to_string());

        // Highlight value if present
        if colon_and_value.len() > 1 {
            let value = &colon_and_value[1..];
            result.push_str(&highlight_yaml_value(value));
        }

        result
    } else if line.trim_start().starts_with("- ") {
        // Handle list items without keys
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        let value = &trimmed[2..];

        format!("{}{}{}", indent, "- ".yellow(), highlight_yaml_value(value))
    } else {
        line.to_string()
    }
}

fn find_yaml_colon(line: &str) -> Option<usize> {
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for (i, ch) in line.char_indices() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ':' if !in_quotes => {
                // Check if it's followed by space, newline, or end of string
                let next_char = line.chars().nth(i + 1);
                if next_char.is_none() || next_char == Some(' ') || next_char == Some('\n') {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn highlight_yaml_value(value: &str) -> String {
    let trimmed = value.trim();

    // Preserve leading space
    let leading_space = if value.starts_with(' ') { " " } else { "" };

    let highlighted = match trimmed {
        "true" | "false" | "yes" | "no" | "on" | "off" => trimmed.blue().bold().to_string(),
        "null" | "~" => trimmed.red().to_string(),
        s if s.starts_with('"') && s.ends_with('"') => s.green().to_string(),
        s if s.starts_with('\'') && s.ends_with('\'') => s.green().to_string(),
        s if s.parse::<f64>().is_ok() => s.magenta().to_string(),
        "" => String::new(),
        s => s.green().to_string(),
    };

    format!("{}{}", leading_space, highlighted)
}

/// Highlight TOML output with colors
pub fn highlight_toml(toml: &str) -> String {
    let mut result = String::new();

    for line in toml.lines() {
        let highlighted_line = highlight_toml_line(line);
        result.push_str(&highlighted_line);
        result.push('\n');
    }

    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

fn highlight_toml_line(line: &str) -> String {
    let trimmed = line.trim();

    // Handle comments
    if trimmed.starts_with('#') {
        return line.dimmed().to_string();
    }

    // Handle section headers [section] or [[array]]
    if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
        let indent = &line[..line.len() - trimmed.len()];
        return format!("{}{}", indent, trimmed.yellow().bold());
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let indent = &line[..line.len() - trimmed.len()];
        return format!("{}{}", indent, trimmed.yellow().bold());
    }

    // Handle key = value
    if let Some(eq_pos) = trimmed.find('=') {
        let indent = &line[..line.len() - trimmed.len()];
        let key = trimmed[..eq_pos].trim();
        let value = trimmed[eq_pos + 1..].trim();

        let highlighted_value = highlight_toml_value(value);
        format!(
            "{}{} {} {}",
            indent,
            key.cyan(),
            "=".white(),
            highlighted_value
        )
    } else {
        line.to_string()
    }
}

fn highlight_toml_value(value: &str) -> String {
    let trimmed = value.trim();

    // String (double or single quoted)
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with("'''") && trimmed.ends_with("'''"))
        || (trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\""))
    {
        return trimmed.green().to_string();
    }

    // Boolean
    if trimmed == "true" || trimmed == "false" {
        return trimmed.blue().bold().to_string();
    }

    // Array
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return highlight_toml_array(trimmed);
    }

    // Inline table
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return trimmed.yellow().to_string();
    }

    // Number (integer or float)
    if trimmed.parse::<f64>().is_ok() || trimmed.parse::<i64>().is_ok() {
        return trimmed.magenta().to_string();
    }

    // Date/time (ISO 8601)
    if trimmed.contains('-') && trimmed.contains(':') {
        return trimmed.cyan().to_string();
    }

    trimmed.to_string()
}

fn highlight_toml_array(array: &str) -> String {
    let mut result = String::new();
    let mut chars = array.chars().peekable();
    let mut in_string = false;
    let mut string_char = '"';

    while let Some(ch) = chars.next() {
        match ch {
            '[' | ']' if !in_string => {
                result.push_str(&ch.to_string().yellow().to_string());
            }
            '"' | '\'' if !in_string => {
                in_string = true;
                string_char = ch;
                result.push_str(&ch.to_string().green().to_string());
            }
            c if in_string && c == string_char => {
                in_string = false;
                result.push_str(&c.to_string().green().to_string());
            }
            _ if in_string => {
                result.push_str(&ch.to_string().green().to_string());
            }
            ',' if !in_string => {
                result.push_str(&",".white().to_string());
            }
            _ if !in_string && (ch.is_ascii_digit() || ch == '-' || ch == '.') => {
                let mut num = String::new();
                num.push(ch);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '.' || next == '_' {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                result.push_str(&num.magenta().to_string());
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

/// Highlight CSV table output with colors
pub fn highlight_csv(csv: &str, raw: bool) -> String {
    if raw {
        // For raw CSV, just highlight headers if present
        return highlight_csv_raw(csv);
    }

    // For table format
    let mut result = String::new();
    let lines: Vec<&str> = csv.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with('+') && line.ends_with('+') {
            // Separator line
            result.push_str(&line.dimmed().to_string());
        } else if line.starts_with('|') && line.ends_with('|') {
            // Data line
            if i == 1 {
                // Header row (usually second line after first separator)
                result.push_str(&highlight_csv_header_row(line));
            } else {
                result.push_str(&highlight_csv_data_row(line));
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

fn highlight_csv_raw(csv: &str) -> String {
    let mut result = String::new();
    let mut first_line = true;

    for line in csv.lines() {
        if first_line {
            // Header line
            result.push_str(&line.cyan().bold().to_string());
            first_line = false;
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    if result.ends_with('\n') {
        result.pop();
    }

    result
}

fn highlight_csv_header_row(line: &str) -> String {
    let mut result = String::new();

    for part in line.split('|') {
        if part.is_empty() {
            result.push('|');
        } else {
            result.push_str(&part.cyan().bold().to_string());
            result.push('|');
        }
    }

    // Remove trailing |
    if result.ends_with('|') {
        result.pop();
    }

    result
}

fn highlight_csv_data_row(line: &str) -> String {
    let mut result = String::new();
    let parts: Vec<&str> = line.split('|').collect();

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            if i < parts.len() - 1 {
                result.push_str(&"|".dimmed().to_string());
            }
        } else {
            let trimmed = part.trim();
            let highlighted = if trimmed.parse::<f64>().is_ok() {
                format!(
                    "{}{}{}",
                    &part[..part.len() - trimmed.len() - (part.len() - part.trim_end().len())],
                    trimmed.magenta(),
                    &part[part.trim_end().len()..]
                )
            } else {
                part.to_string()
            };
            result.push_str(&highlighted);
            if i < parts.len() - 1 {
                result.push_str(&"|".dimmed().to_string());
            }
        }
    }

    result
}

/// Highlight XML output with colors
pub fn highlight_xml(xml: &str) -> String {
    let mut result = String::new();
    let mut chars = xml.chars().peekable();
    let mut in_tag = false;
    let mut in_string = false;
    let mut in_comment = false;
    let mut string_char = '"';

    while let Some(ch) = chars.next() {
        // Check for comment start
        if ch == '<' && !in_string {
            let peek: String = chars.clone().take(3).collect();
            if peek.starts_with("!--") {
                in_comment = true;
                result.push_str(&"<".dimmed().to_string());
                continue;
            }
        }

        // Check for comment end
        if in_comment {
            result.push_str(&ch.to_string().dimmed().to_string());
            if ch == '>' {
                // Check if preceded by --
                let len = result.len();
                if len >= 3 {
                    in_comment = false;
                }
            }
            continue;
        }

        match ch {
            '<' if !in_string => {
                in_tag = true;
                result.push_str(&"<".yellow().to_string());
            }
            '>' if !in_string && in_tag => {
                in_tag = false;
                result.push_str(&">".yellow().to_string());
            }
            '/' if in_tag && !in_string => {
                result.push_str(&"/".yellow().to_string());
            }
            '=' if in_tag && !in_string => {
                result.push_str(&"=".white().to_string());
            }
            '"' | '\'' if in_tag && !in_string => {
                in_string = true;
                string_char = ch;
                result.push_str(&ch.to_string().green().to_string());
            }
            c if in_string && c == string_char => {
                in_string = false;
                result.push_str(&c.to_string().green().to_string());
            }
            _ if in_string => {
                result.push_str(&ch.to_string().green().to_string());
            }
            '?' if in_tag && !in_string => {
                result.push_str(&"?".magenta().to_string());
            }
            '!' if in_tag && !in_string => {
                result.push_str(&"!".magenta().to_string());
            }
            _ if in_tag && !in_string && !ch.is_whitespace() => {
                // Tag name or attribute name
                let mut name = String::new();
                name.push(ch);
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == ':' || next == '-' || next == '_' {
                        name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Check if it's a tag name (first word after <) or attribute
                // Simple heuristic: if previous non-whitespace was < or /, it's a tag name
                if is_tag_name_context(&result) {
                    result.push_str(&name.cyan().to_string());
                } else {
                    result.push_str(&name.yellow().to_string());
                }
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

fn is_tag_name_context(result: &str) -> bool {
    for ch in result.chars().rev() {
        if ch.is_whitespace() {
            continue;
        }
        return ch == '<' || ch == '/';
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_json_basic() {
        let json = r#"{"name": "test", "value": 42, "active": true, "data": null}"#;
        let _ = highlight_json(json);
    }

    #[test]
    fn test_highlight_yaml_basic() {
        let yaml = "name: test\nvalue: 42\nactive: true";
        let _ = highlight_yaml(yaml);
    }

    #[test]
    fn test_highlight_toml_basic() {
        let toml = "[section]\nkey = \"value\"\nnum = 42";
        let _ = highlight_toml(toml);
    }

    #[test]
    fn test_highlight_xml_basic() {
        let xml = r#"<?xml version="1.0"?><root><item>test</item></root>"#;
        let _ = highlight_xml(xml);
    }

    #[test]
    fn test_highlight_csv_basic() {
        let csv = "+---+---+\n| a | b |\n+---+---+\n| 1 | 2 |\n+---+---+";
        let _ = highlight_csv(csv, false);
    }
}

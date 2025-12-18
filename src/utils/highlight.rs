//! Syntax highlighting for JSON and YAML output

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_json_basic() {
        // Just verify it doesn't panic
        let json = r#"{"name": "test", "value": 42, "active": true, "data": null}"#;
        let _ = highlight_json(json);
    }

    #[test]
    fn test_highlight_yaml_basic() {
        // Just verify it doesn't panic
        let yaml = "name: test\nvalue: 42\nactive: true";
        let _ = highlight_yaml(yaml);
    }
}

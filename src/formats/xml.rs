//! XML format handling

use anyhow::{Context, Result};
use quick_xml::events::{BytesDecl, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::fs;
use std::io::{self, Cursor, Read};
use std::path::Path;

/// Read input from file or stdin
pub fn read_input(path: Option<&Path>) -> Result<String> {
    match path {
        Some(p) => {
            fs::read_to_string(p).with_context(|| format!("Failed to read file: {}", p.display()))
        }
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            Ok(buffer)
        }
    }
}

/// Validate XML by parsing it
pub fn validate(content: &str) -> Result<()> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(_) => continue,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ))
            }
        }
    }
    Ok(())
}

/// Format XML with proper indentation
pub fn to_pretty(content: &str) -> Result<String> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                writer
                    .write_event(Event::Start(e))
                    .context("Failed to write XML start element")?;
            }
            Ok(Event::End(e)) => {
                writer
                    .write_event(Event::End(e))
                    .context("Failed to write XML end element")?;
            }
            Ok(Event::Empty(e)) => {
                writer
                    .write_event(Event::Empty(e))
                    .context("Failed to write XML empty element")?;
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().context("Failed to unescape XML text")?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    writer
                        .write_event(Event::Text(BytesText::new(trimmed)))
                        .context("Failed to write XML text")?;
                }
            }
            Ok(Event::CData(e)) => {
                writer
                    .write_event(Event::CData(e))
                    .context("Failed to write XML CDATA")?;
            }
            Ok(Event::Comment(e)) => {
                writer
                    .write_event(Event::Comment(e))
                    .context("Failed to write XML comment")?;
            }
            Ok(Event::Decl(e)) => {
                // Preserve XML declaration
                let version = e.version().context("Failed to get XML version")?;
                let encoding = e.encoding().transpose().context("Failed to get encoding")?;
                let standalone = e
                    .standalone()
                    .transpose()
                    .context("Failed to get standalone")?;

                let mut decl = BytesDecl::new(
                    std::str::from_utf8(&version).unwrap_or("1.0"),
                    encoding
                        .as_ref()
                        .map(|e| std::str::from_utf8(e).unwrap_or("UTF-8")),
                    standalone
                        .as_ref()
                        .map(|s| std::str::from_utf8(s).unwrap_or("no")),
                );
                // Use the original declaration attributes if parsing fails
                if version.is_empty() {
                    decl = BytesDecl::new("1.0", Some("UTF-8"), None);
                }
                writer
                    .write_event(Event::Decl(decl))
                    .context("Failed to write XML declaration")?;
            }
            Ok(Event::PI(e)) => {
                writer
                    .write_event(Event::PI(e))
                    .context("Failed to write XML processing instruction")?;
            }
            Ok(Event::DocType(e)) => {
                writer
                    .write_event(Event::DocType(e))
                    .context("Failed to write XML doctype")?;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ))
            }
        }
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).context("Invalid UTF-8 in XML output")
}

/// Minify XML (remove unnecessary whitespace)
pub fn to_compact(content: &str) -> Result<String> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                writer
                    .write_event(Event::Start(e))
                    .context("Failed to write XML element")?;
            }
            Ok(Event::End(e)) => {
                writer
                    .write_event(Event::End(e))
                    .context("Failed to write XML element")?;
            }
            Ok(Event::Empty(e)) => {
                writer
                    .write_event(Event::Empty(e))
                    .context("Failed to write XML element")?;
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().context("Failed to unescape XML text")?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    writer
                        .write_event(Event::Text(BytesText::new(trimmed)))
                        .context("Failed to write XML text")?;
                }
            }
            Ok(Event::Decl(e)) => {
                writer
                    .write_event(Event::Decl(e))
                    .context("Failed to write XML declaration")?;
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                writer
                    .write_event(event)
                    .context("Failed to write XML event")?;
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ))
            }
        }
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).context("Invalid UTF-8 in XML output")
}

//! CSV format handling

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// CSV data representation
#[derive(Debug, Clone)]
pub struct CsvData {
    pub headers: Option<Vec<String>>,
    pub rows: Vec<Vec<String>>,
}

impl CsvData {
    /// Create new CsvData with headers
    pub fn with_headers(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self {
            headers: Some(headers),
            rows,
        }
    }

    /// Create new CsvData without headers
    pub fn without_headers(rows: Vec<Vec<String>>) -> Self {
        Self {
            headers: None,
            rows,
        }
    }
}

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

/// Parse CSV string into CsvData
pub fn parse(content: &str, has_headers: bool) -> Result<CsvData> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content.as_bytes());

    let headers = if has_headers {
        Some(
            reader
                .headers()
                .context("Failed to read CSV headers")?
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    } else {
        None
    };

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }

    Ok(CsvData { headers, rows })
}

/// Convert CsvData to formatted table string
pub fn to_table(data: &CsvData) -> Result<String> {
    if data.rows.is_empty() && data.headers.is_none() {
        return Ok(String::new());
    }

    // Calculate column widths
    let num_cols = data
        .headers
        .as_ref()
        .map(|h| h.len())
        .unwrap_or_else(|| data.rows.first().map(|r| r.len()).unwrap_or(0));

    let mut col_widths = vec![0usize; num_cols];

    // Account for headers
    if let Some(headers) = &data.headers {
        for (i, h) in headers.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(h.len());
            }
        }
    }

    // Account for data rows
    for row in &data.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    let mut output = String::new();

    // Helper to create a separator line
    let separator: String = col_widths
        .iter()
        .map(|&w| "-".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("+");
    let separator = format!("+{}+", separator);

    // Print headers if present
    if let Some(headers) = &data.headers {
        output.push_str(&separator);
        output.push('\n');

        let header_row: String = headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let width = col_widths.get(i).copied().unwrap_or(0);
                format!(" {:^width$} ", h, width = width)
            })
            .collect::<Vec<_>>()
            .join("|");
        output.push_str(&format!("|{}|", header_row));
        output.push('\n');

        output.push_str(&separator);
        output.push('\n');
    } else {
        output.push_str(&separator);
        output.push('\n');
    }

    // Print data rows
    for row in &data.rows {
        let data_row: String = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = col_widths.get(i).copied().unwrap_or(0);
                format!(" {:<width$} ", cell, width = width)
            })
            .collect::<Vec<_>>()
            .join("|");
        output.push_str(&format!("|{}|", data_row));
        output.push('\n');
    }

    output.push_str(&separator);

    Ok(output)
}

/// Convert CsvData back to CSV format
pub fn to_csv(data: &CsvData) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());

    if let Some(headers) = &data.headers {
        writer
            .write_record(headers)
            .context("Failed to write CSV headers")?;
    }

    for row in &data.rows {
        writer
            .write_record(row)
            .context("Failed to write CSV record")?;
    }

    let bytes = writer
        .into_inner()
        .context("Failed to finalize CSV output")?;
    String::from_utf8(bytes).context("Invalid UTF-8 in CSV output")
}

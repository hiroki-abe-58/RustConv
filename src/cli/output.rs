//! Output formatting utilities

use std::io::{self, Write};

/// Write output to stdout
pub fn write_output(content: &str) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", content)?;
    Ok(())
}

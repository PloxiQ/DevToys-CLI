//! Bounded UTF-8 input for tools that consume files or streams.
use std::io::Read;
use std::path::Path;

use crate::error::{Result, ToolError};

pub const MIB: usize = 1024 * 1024;

pub fn check(input: &str, limit: usize, label: &str) -> Result<()> {
    if input.len() > limit {
        return Err(
            ToolError::new(format!("{label} exceeds the {limit}-byte limit."))
                .with_hint("Use a smaller input."),
        );
    }
    Ok(())
}

pub fn read(reader: impl Read, limit: usize, label: &str) -> Result<String> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            ToolError::new(format!("Could not read {label}: {error}."))
                .with_hint("Check that the input is readable.")
        })?;
    if bytes.len() > limit {
        return Err(
            ToolError::new(format!("{label} exceeds the {limit}-byte limit."))
                .with_hint("Use a smaller input."),
        );
    }
    String::from_utf8(bytes).map_err(|_| {
        ToolError::new(format!("{label} is not valid UTF-8."))
            .with_hint("Save the input as UTF-8 text before trying again.")
    })
}

pub fn read_file(path: &Path, limit: usize) -> Result<String> {
    let label = format!("`{}`", path.display());
    let file = std::fs::File::open(path).map_err(|error| {
        ToolError::new(format!("Could not open {label}: {error}."))
            .with_hint("Check that the file exists and is readable.")
    })?;
    read(file, limit, &label)
}

#[cfg(test)]
mod tests {
    #[test]
    fn bounds_reads_and_rejects_invalid_utf8() {
        assert!(super::read(&b"abcd"[..], 3, "input").is_err());
        assert!(super::read(&b"\xff"[..], 3, "input").is_err());
        assert_eq!(
            super::read(&b"abc"[..], 3, "input").ok().as_deref(),
            Some("abc")
        );
    }
}

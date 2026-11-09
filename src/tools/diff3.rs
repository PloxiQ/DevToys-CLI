//! Bounded, offline three-way UTF-8 text merging.
use super::limits;
use crate::error::{Result, ToolError};

pub const MAX_INPUT: usize = 256 * 1024;
pub const MAX_LINES: usize = 10_000;
pub const MAX_OUTPUT: usize = 8 * limits::MIB;

#[derive(Debug)]
pub struct MergeResult {
    pub text: String,
    pub conflicted: bool,
}

pub fn merge(base: &str, ours: &str, theirs: &str, labels: [&str; 3]) -> Result<MergeResult> {
    for (label, input) in [("Base", base), ("Ours", ours), ("Theirs", theirs)] {
        limits::check(input, MAX_INPUT, label)?;
        if input.lines().count() > MAX_LINES {
            return Err(
                ToolError::new(format!("{label} exceeds {MAX_LINES} lines."))
                    .with_hint("Split large files before merging."),
            );
        }
    }
    for label in labels {
        if label.len() > 128 || label.chars().any(char::is_control) {
            return Err(ToolError::new("Invalid conflict label.")
                .with_hint("Use at most 128 UTF-8 bytes without control characters."));
        }
    }
    // Choose a marker length absent from the input, so custom labels never
    // rewrite marker-looking lines belonging to the user's actual content.
    let mut marker_len = 7;
    loop {
        let markers = marker_lines(marker_len, ["original", "ours", "theirs"]);
        if ![base, ours, theirs]
            .iter()
            .flat_map(|s| s.lines())
            .any(|line| {
                markers
                    .iter()
                    .any(|marker| line == marker.trim_end_matches('\n'))
            })
        {
            break;
        }
        marker_len += 1;
        if marker_len > 64 {
            return Err(ToolError::new("Too many nested conflict markers.")
                .with_hint("Resolve existing conflict markers before merging again."));
        }
    }
    let mut options = diffy::MergeOptions::new();
    options
        .set_conflict_marker_length(marker_len)
        .set_incomplete_hunk_style(diffy::IncompleteHunkStyle::Git);
    let (text, conflicted) = match options.merge(base, ours, theirs) {
        Ok(text) => (text, false),
        Err(text) => (text, true),
    };
    limits::check(&text, MAX_OUTPUT, "Merge result")?;
    if !conflicted {
        return Ok(MergeResult { text, conflicted });
    }
    let markers = marker_lines(marker_len, ["original", "ours", "theirs"]);
    let replacements = marker_lines(7, labels);
    let mut output = String::new();
    for line in text.split_inclusive('\n') {
        let replacement = markers
            .iter()
            .position(|m| m == line)
            .map_or(line, |i| replacements[i].as_str());
        if output.len() + replacement.len() > MAX_OUTPUT {
            return Err(ToolError::new("Merge result exceeds 8 MiB.")
                .with_hint("Split large files before merging."));
        }
        output.push_str(replacement);
    }
    Ok(MergeResult {
        text: output,
        conflicted,
    })
}

fn marker_lines(length: usize, labels: [&str; 3]) -> [String; 4] {
    [
        format!("{} {}\n", "<".repeat(length), labels[1]),
        format!("{} {}\n", "|".repeat(length), labels[0]),
        format!("{}\n", "=".repeat(length)),
        format!("{} {}\n", ">".repeat(length), labels[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    const LABELS: [&str; 3] = ["base", "ours", "theirs"];

    #[test]
    fn merges_disjoint_edits_and_identical_inputs() -> Result<()> {
        let result = merge("a\nb\nc\n", "A\nb\nc\n", "a\nb\nC\n", LABELS)?;
        assert!(!result.conflicted);
        assert_eq!(result.text, "A\nb\nC\n");
        for text in ["", "a\r\nb\r\n", "no final newline"] {
            assert_eq!(merge(text, text, text, LABELS)?.text, text);
        }
        assert_eq!(
            merge("a\r\nb\r\nc", "A\r\nb\r\nc", "a\r\nb\r\nC", LABELS)?.text,
            "A\r\nb\r\nC"
        );
        Ok(())
    }

    #[test]
    fn conflicts_with_labels_and_missing_newline() -> Result<()> {
        let result = merge("base", "left", "right", ["ancestor", "local", "remote"])?;
        assert!(result.conflicted);
        assert_eq!(
            result.text,
            "<<<<<<< local\nleft\n||||||| ancestor\nbase\n=======\nright\n>>>>>>> remote\n"
        );
        Ok(())
    }

    #[test]
    fn preserves_literal_markers_and_bounds_inputs() -> Result<()> {
        let base = "<<<<<<< ours\nvalue\n";
        let result = merge(
            base,
            "<<<<<<< ours\nleft\n",
            "<<<<<<< ours\nright\n",
            ["b", "o", "t"],
        )?;
        assert!(result.text.starts_with("<<<<<<< ours\n<<<<<<< o\n"));
        assert!(merge(&"a".repeat(MAX_INPUT + 1), "", "", LABELS).is_err());
        assert!(merge(&"\n".repeat(MAX_LINES + 1), "", "", LABELS).is_err());
        assert!(merge("a", "b", "c", ["bad\nlabel", "o", "t"]).is_err());
        Ok(())
    }
}

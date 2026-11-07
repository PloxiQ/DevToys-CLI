use std::collections::HashMap;

use regex::{Regex, RegexBuilder};

use crate::error::{Result, ToolError};

#[derive(Debug, Clone, Default)]
pub struct RegexOptions {
    pub case_insensitive: bool,
    pub multi_line: bool,
    pub dot_matches_new_line: bool,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchInfo {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub groups: Vec<Option<String>>,
    pub named_groups: Vec<(String, Option<String>)>,
}

#[derive(Debug, Clone)]
pub struct RegexReport {
    pub pattern: String,
    pub matches: Vec<MatchInfo>,
    pub replaced: Option<String>,
}

pub fn test(pattern: &str, text: &str, options: &RegexOptions) -> Result<RegexReport> {
    let regex = build_regex(pattern, options)?;
    let mut matches = Vec::new();

    for captures in regex.captures_iter(text) {
        let Some(whole) = captures.get(0) else {
            continue;
        };
        let groups = (1..captures.len())
            .map(|index| captures.get(index).map(|value| value.as_str().to_owned()))
            .collect::<Vec<_>>();

        let mut names = HashMap::new();
        for (index, name) in regex.capture_names().enumerate() {
            if let Some(name) = name {
                names.insert(index, name.to_owned());
            }
        }
        let named_groups = names
            .into_iter()
            .map(|(index, name)| {
                let value = captures.get(index).map(|value| value.as_str().to_owned());
                (name, value)
            })
            .collect();

        matches.push(MatchInfo {
            start: whole.start(),
            end: whole.end(),
            text: whole.as_str().to_owned(),
            groups,
            named_groups,
        });
    }

    let replaced = options
        .replacement
        .as_ref()
        .map(|replacement| regex.replace_all(text, replacement.as_str()).into_owned());

    Ok(RegexReport {
        pattern: pattern.to_owned(),
        matches,
        replaced,
    })
}

fn build_regex(pattern: &str, options: &RegexOptions) -> Result<Regex> {
    RegexBuilder::new(pattern)
        .case_insensitive(options.case_insensitive)
        .multi_line(options.multi_line)
        .dot_matches_new_line(options.dot_matches_new_line)
        .build()
        .map_err(|error| {
            ToolError::new(format!("Invalid regular expression: {error}."))
                .with_hint("Check parentheses, character classes and escape sequences.")
        })
}

/// Returns a 1-based line and character column for a byte offset.
#[must_use]
pub fn line_column(text: &str, byte_offset: usize) -> (usize, usize) {
    let end = byte_offset.min(text.len());
    let before = &text[..end];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(1, |segment| segment.chars().count() + 1);
    (line, column)
}

#[cfg(test)]
mod tests {
    use super::{line_column, test, RegexOptions};

    #[test]
    fn finds_matches_and_groups() -> Result<(), Box<dyn std::error::Error>> {
        let report = test(
            r"(\d+)-(\d+)",
            "call 12-34 and 56-78",
            &RegexOptions::default(),
        )?;
        assert_eq!(report.matches.len(), 2);
        assert_eq!(report.matches[0].start, 5);
        assert_eq!(report.matches[0].text, "12-34");
        assert_eq!(report.matches[0].groups[0].as_deref(), Some("12"));
        assert_eq!(report.matches[0].groups[1].as_deref(), Some("34"));
        Ok(())
    }

    #[test]
    fn supports_named_groups_and_replacement() -> Result<(), Box<dyn std::error::Error>> {
        let options = RegexOptions {
            replacement: Some("${last}, ${first}".to_owned()),
            ..RegexOptions::default()
        };
        let report = test(r"(?P<first>\w+) (?P<last>\w+)", "Ada Lovelace", &options)?;
        assert_eq!(report.matches.len(), 1);
        assert!(report.matches[0]
            .named_groups
            .iter()
            .any(|(name, value)| name == "first" && value.as_deref() == Some("Ada")));
        assert_eq!(report.replaced.as_deref(), Some("Lovelace, Ada"));
        Ok(())
    }

    #[test]
    fn supports_flags_and_no_matches() -> Result<(), Box<dyn std::error::Error>> {
        let options = RegexOptions {
            case_insensitive: true,
            ..RegexOptions::default()
        };
        assert_eq!(test("rust", "RUST rust", &options)?.matches.len(), 2);
        assert!(test("nope", "text", &options)?.matches.is_empty());
        Ok(())
    }

    #[test]
    fn reports_line_and_column() {
        assert_eq!(line_column("a\nbc", 2), (2, 1));
        assert_eq!(line_column("abc", 2), (1, 3));
    }

    #[test]
    fn rejects_invalid_regex() {
        assert!(test("(", "text", &RegexOptions::default()).is_err());
    }
}

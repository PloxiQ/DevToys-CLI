use std::collections::HashSet;

use crate::error::{Result, ToolError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub characters: usize,
    pub characters_without_whitespace: usize,
    pub words: usize,
    pub sentences: usize,
    pub paragraphs: usize,
    pub lines: usize,
    pub bytes: usize,
    pub longest_line: usize,
    pub reading_minutes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SortOptions {
    pub reverse: bool,
    pub unique: bool,
    pub ignore_case: bool,
    pub trim: bool,
}

#[must_use]
pub fn stats(input: &str) -> Stats {
    let characters = input.chars().count();
    let characters_without_whitespace = input
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    let words = input.split_whitespace().count();
    let lines = input.lines().count();
    let longest_line = input
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let reading_minutes = words.div_ceil(200);

    Stats {
        characters,
        characters_without_whitespace,
        words,
        sentences: count_sentences(input),
        paragraphs: count_paragraphs(input),
        lines,
        bytes: input.len(),
        longest_line,
        reading_minutes,
    }
}

#[must_use]
pub fn format_stats(input: &str) -> String {
    let stats = stats(input);
    let reading = if stats.words == 0 {
        "0 min".to_owned()
    } else {
        format!("~{} min", stats.reading_minutes)
    };

    [
        format!(
            "Characters : {} ({} without whitespace)",
            stats.characters, stats.characters_without_whitespace
        ),
        format!("Words      : {}", stats.words),
        format!("Sentences  : {}", stats.sentences),
        format!("Paragraphs : {}", stats.paragraphs),
        format!("Lines      : {}", stats.lines),
        format!("Longest    : {} characters", stats.longest_line),
        format!("Bytes      : {} (UTF-8)", stats.bytes),
        format!("Reading    : {reading}"),
    ]
    .join("\n")
}

fn count_sentences(input: &str) -> usize {
    let mut sentences = 0;
    let mut has_content = false;
    for character in input.chars() {
        if !character.is_whitespace() {
            has_content = true;
        }
        if matches!(character, '.' | '!' | '?') && has_content {
            sentences += 1;
            has_content = false;
        }
    }
    if has_content {
        sentences += 1;
    }
    sentences
}

fn count_paragraphs(input: &str) -> usize {
    let mut paragraphs = 0;
    let mut in_paragraph = false;
    for line in input.lines() {
        if line.trim().is_empty() {
            if in_paragraph {
                paragraphs += 1;
                in_paragraph = false;
            }
        } else {
            in_paragraph = true;
        }
    }
    if in_paragraph {
        paragraphs += 1;
    }
    paragraphs
}

#[must_use]
pub fn sort_lines(input: &str, options: SortOptions) -> String {
    let mut lines: Vec<String> = input.lines().map(str::to_owned).collect();
    if options.trim {
        for line in &mut lines {
            *line = line.trim().to_owned();
        }
    }

    if options.ignore_case {
        lines.sort_by(|left, right| {
            left.to_lowercase()
                .cmp(&right.to_lowercase())
                .then_with(|| left.cmp(right))
        });
    } else {
        lines.sort();
    }

    if options.unique {
        lines.dedup_by(|left, right| {
            if options.ignore_case {
                left.to_lowercase() == right.to_lowercase()
            } else {
                left == right
            }
        });
    }
    if options.reverse {
        lines.reverse();
    }
    lines.join("\n")
}

pub fn parse_sort_options(value: &str) -> Result<SortOptions> {
    let mut options = SortOptions::default();
    for option in value
        .split([',', ' ', ';'])
        .map(str::trim)
        .filter(|option| !option.is_empty())
    {
        match option.to_ascii_lowercase().as_str() {
            "reverse" | "desc" | "descending" => options.reverse = true,
            "unique" | "dedupe" | "deduplicate" => options.unique = true,
            "ignore-case" | "ignorecase" | "ci" | "case-insensitive" => {
                options.ignore_case = true;
            }
            "trim" => options.trim = true,
            other => {
                return Err(
                    ToolError::new(format!("Unknown sort option `{other}`.")).with_hint(
                        "Use unique, reverse, ignore-case and/or trim, separated by commas.",
                    ),
                );
            }
        }
    }
    Ok(options)
}

/// Removes duplicate lines while keeping the first occurrence.
#[must_use]
pub fn dedupe_lines(input: &str, ignore_case: bool, trim: bool) -> String {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for raw in input.lines() {
        let line = if trim { raw.trim() } else { raw };
        let key = if ignore_case {
            line.to_lowercase()
        } else {
            line.to_owned()
        };
        if seen.insert(key) {
            output.push(line.to_owned());
        }
    }
    output.join("\n")
}

pub fn parse_dedupe_options(value: &str) -> Result<(bool, bool)> {
    let mut ignore_case = false;
    let mut trim = false;
    for option in value
        .split([',', ' ', ';'])
        .map(str::trim)
        .filter(|option| !option.is_empty())
    {
        match option.to_ascii_lowercase().as_str() {
            "ignore-case" | "ignorecase" | "ci" | "case-insensitive" => ignore_case = true,
            "trim" => trim = true,
            other => {
                return Err(ToolError::new(format!("Unknown dedupe option `{other}`."))
                    .with_hint("Use ignore-case and/or trim, separated by commas."));
            }
        }
    }
    Ok((ignore_case, trim))
}

/// Produces a lowercase ASCII slug suitable for URLs and file names.
#[must_use]
pub fn slugify(input: &str) -> String {
    let transliterated = deunicode::deunicode(input);
    let mut output = String::new();
    let mut separator = false;
    for character in transliterated.chars() {
        if character.is_ascii_alphanumeric() {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.push(character.to_ascii_lowercase());
            separator = false;
        } else {
            separator = !output.is_empty();
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        dedupe_lines, format_stats, parse_sort_options, slugify, sort_lines, stats, SortOptions,
    };

    #[test]
    fn counts_text_statistics() {
        let value = stats("Hello world!\nSecond sentence?\n\nNew paragraph.");
        assert_eq!(value.characters, 45);
        assert_eq!(value.words, 6);
        assert_eq!(value.sentences, 3);
        assert_eq!(value.paragraphs, 2);
        assert_eq!(value.lines, 4);
        assert_eq!(value.bytes, 45);
        assert!(format_stats("one two").contains("Words      : 2"));
    }

    #[test]
    fn sorts_and_deduplicates_lines() {
        let input = "banana\nApple\napple\ncherry";
        let output = sort_lines(
            input,
            SortOptions {
                unique: true,
                ignore_case: true,
                ..SortOptions::default()
            },
        );
        assert_eq!(output, "Apple\nbanana\ncherry");

        let output = sort_lines(
            "b\na\nc",
            SortOptions {
                reverse: true,
                ..SortOptions::default()
            },
        );
        assert_eq!(output, "c\nb\na");
    }

    #[test]
    fn parses_sort_options() {
        let options = parse_sort_options("reverse, unique, ignore-case").unwrap_or_default();
        assert!(options.reverse);
        assert!(options.unique);
        assert!(options.ignore_case);
        assert!(parse_sort_options("bogus").is_err());
    }

    #[test]
    fn deduplicates_and_slugifies() {
        assert_eq!(dedupe_lines("a\nb\na\nB", true, false), "a\nb");
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("Crème brûlée"), "creme-brulee");
        assert_eq!(slugify("  ***  "), "");
    }
}

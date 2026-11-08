use crate::error::{Result, ToolError};

const MAX_CELLS: usize = 4_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Equal,
    Delete,
    Insert,
}

#[derive(Debug, Clone, Copy)]
struct Op<'a> {
    kind: Kind,
    text: &'a str,
}

/// Produces a unified line diff. Equal inputs return a short message.
pub fn diff(left: &str, right: &str, context: usize) -> Result<String> {
    let left_lines: Vec<&str> = left.lines().collect();
    let right_lines: Vec<&str> = right.lines().collect();

    let rows = left_lines.len() + 1;
    let columns = right_lines.len() + 1;
    let cells = rows
        .checked_mul(columns)
        .ok_or_else(|| ToolError::new("The inputs are too large to diff."))?;
    if cells > MAX_CELLS {
        return Err(ToolError::new(format!(
            "The inputs are too large to diff ({} x {} lines).",
            left_lines.len(),
            right_lines.len()
        ))
        .with_hint("Limit each side to roughly 2,000 lines."));
    }

    let mut matrix = vec![0u32; cells];
    for row in 1..rows {
        for column in 1..columns {
            matrix[row * columns + column] = if left_lines[row - 1] == right_lines[column - 1] {
                matrix[(row - 1) * columns + column - 1] + 1
            } else {
                matrix[(row - 1) * columns + column].max(matrix[row * columns + column - 1])
            };
        }
    }

    let mut operations = Vec::with_capacity(left_lines.len() + right_lines.len());
    let (mut row, mut column) = (left_lines.len(), right_lines.len());
    while row > 0 || column > 0 {
        if row > 0 && column > 0 && left_lines[row - 1] == right_lines[column - 1] {
            operations.push(Op {
                kind: Kind::Equal,
                text: left_lines[row - 1],
            });
            row -= 1;
            column -= 1;
        } else if column > 0
            && (row == 0
                || matrix[row * columns + column - 1] >= matrix[(row - 1) * columns + column])
        {
            operations.push(Op {
                kind: Kind::Insert,
                text: right_lines[column - 1],
            });
            column -= 1;
        } else {
            operations.push(Op {
                kind: Kind::Delete,
                text: left_lines[row - 1],
            });
            row -= 1;
        }
    }
    operations.reverse();

    if !operations
        .iter()
        .any(|operation| operation.kind != Kind::Equal)
    {
        return Ok("No differences.".to_owned());
    }

    let positions = line_positions(&operations);
    let changes: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, operation)| operation.kind != Kind::Equal)
        .map(|(index, _)| index)
        .collect();

    let mut groups = Vec::new();
    let Some(first) = changes.first().copied() else {
        return Ok("No differences.".to_owned());
    };
    let mut start = first.saturating_sub(context);
    let mut end = first + 1;
    for index in changes.iter().copied().skip(1) {
        if index <= end.saturating_add(context.saturating_mul(2)) {
            end = index + 1;
        } else {
            groups.push((start, end.saturating_add(context).min(operations.len())));
            start = index.saturating_sub(context);
            end = index + 1;
        }
    }
    groups.push((start, end.saturating_add(context).min(operations.len())));

    let mut output = String::from("--- left\n+++ right\n");
    for (group_start, group_end) in groups {
        let (old_start, new_start) = positions[group_start];
        let old_count = operations[group_start..group_end]
            .iter()
            .filter(|operation| matches!(operation.kind, Kind::Equal | Kind::Delete))
            .count();
        let new_count = operations[group_start..group_end]
            .iter()
            .filter(|operation| matches!(operation.kind, Kind::Equal | Kind::Insert))
            .count();
        let old_display = if old_count == 0 {
            old_start.saturating_sub(1)
        } else {
            old_start
        };
        let new_display = if new_count == 0 {
            new_start.saturating_sub(1)
        } else {
            new_start
        };
        output.push_str(&format!(
            "@@ -{old_display},{old_count} +{new_display},{new_count} @@\n"
        ));
        for operation in &operations[group_start..group_end] {
            let marker = match operation.kind {
                Kind::Equal => ' ',
                Kind::Delete => '-',
                Kind::Insert => '+',
            };
            output.push(marker);
            output.push_str(operation.text);
            output.push('\n');
        }
    }

    Ok(output.trim_end_matches('\n').to_owned())
}

fn line_positions(operations: &[Op<'_>]) -> Vec<(usize, usize)> {
    let mut positions = Vec::with_capacity(operations.len() + 1);
    let mut old_line = 1;
    let mut new_line = 1;
    for operation in operations {
        positions.push((old_line, new_line));
        match operation.kind {
            Kind::Equal => {
                old_line += 1;
                new_line += 1;
            }
            Kind::Delete => old_line += 1,
            Kind::Insert => new_line += 1,
        }
    }
    positions.push((old_line, new_line));
    positions
}

#[cfg(test)]
mod tests {
    use super::diff;

    #[test]
    fn reports_identical_inputs() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(diff("a\nb", "a\nb", 3)?, "No differences.");
        Ok(())
    }

    #[test]
    fn reports_deletions_and_insertions() -> Result<(), Box<dyn std::error::Error>> {
        let output = diff("one\ntwo\nthree", "one\nthree\nfour", 1)?;
        assert!(output.contains("-two"));
        assert!(output.contains("+four"));
        assert!(output.contains("@@"));
        Ok(())
    }

    #[test]
    fn includes_context_and_handles_empty_sides() -> Result<(), Box<dyn std::error::Error>> {
        let output = diff("a\nb\nc\nd\ne", "a\nb\nc\nX\ne", 2)?;
        assert!(output.contains(" c"));
        assert!(output.contains("-d"));
        assert!(output.contains("+X"));

        let output = diff("", "new", 3)?;
        assert!(output.contains("@@ -0,0 +1,1 @@"));
        assert!(output.contains("+new"));
        Ok(())
    }
}

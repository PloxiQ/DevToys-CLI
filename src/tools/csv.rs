use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::error::{Result, ToolError};

pub fn to_json(input: &str, delimiter: u8, has_headers: bool) -> Result<String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(has_headers)
        .flexible(false)
        .from_reader(input.as_bytes());

    let headers = if has_headers {
        let headers = reader
            .headers()
            .map_err(|error| csv_error("read CSV headers", &error))?;
        let mut seen = HashSet::new();
        let mut values = Vec::with_capacity(headers.len());
        for (index, header) in headers.iter().enumerate() {
            let header = if index == 0 {
                header.trim_start_matches('\u{feff}')
            } else {
                header
            };
            if !seen.insert(header.to_owned()) {
                return Err(ToolError::new(format!(
                    "CSV header `{header}` appears more than once."
                ))
                .with_hint("Rename duplicate columns so each JSON key is unique."));
            }
            values.push(header.to_owned());
        }
        Some(values)
    } else {
        None
    };

    let mut rows = Vec::new();
    for (index, record) in reader.records().enumerate() {
        let record = record.map_err(|error| csv_error("read CSV row", &error))?;
        if let Some(headers) = &headers {
            if record.len() != headers.len() {
                return Err(ToolError::new(format!(
                    "CSV row {} has {} columns; the header has {}.",
                    index + 2,
                    record.len(),
                    headers.len()
                ))
                .with_hint("Every row must have the same number of columns as the header."));
            }
            let mut object = Map::new();
            for (column, value) in headers.iter().zip(record.iter()) {
                object.insert(column.clone(), Value::String(value.to_owned()));
            }
            rows.push(Value::Object(object));
        } else {
            rows.push(Value::Array(
                record
                    .iter()
                    .map(|value| Value::String(value.to_owned()))
                    .collect(),
            ));
        }
    }

    serde_json::to_string_pretty(&Value::Array(rows))
        .map_err(|error| ToolError::new(format!("Could not serialize JSON: {error}.")))
}

pub fn from_json(input: &str, delimiter: u8) -> Result<String> {
    let value: Value = serde_json::from_str(input).map_err(|error| {
        ToolError::new(format!("Invalid JSON: {error}."))
            .with_hint("Pass a JSON array of objects or a JSON array of arrays.")
    })?;
    let Value::Array(rows) = value else {
        return Err(ToolError::new("JSON input must be an array.")
            .with_hint("Example: `[{\"name\":\"Alice\",\"age\":30}]` or `[[\"Alice\",30]]`."));
    };
    if rows.is_empty() {
        return Ok(String::new());
    }

    if rows.iter().all(Value::is_object) {
        return objects_to_csv(&rows, delimiter);
    }
    if rows.iter().all(Value::is_array) {
        return arrays_to_csv(&rows, delimiter);
    }

    Err(
        ToolError::new("JSON rows must be all objects or all arrays, not a mix.")
            .with_hint("Use an array of objects with keys, or an array of arrays."),
    )
}

fn objects_to_csv(rows: &[Value], delimiter: u8) -> Result<String> {
    let mut headers = Vec::new();
    let mut seen = HashSet::new();
    for row in rows {
        let Some(object) = row.as_object() else {
            continue;
        };
        for key in object.keys() {
            if seen.insert(key.clone()) {
                headers.push(key.clone());
            }
        }
    }
    if headers.is_empty() {
        return Err(ToolError::new("The JSON objects have no keys.")
            .with_hint("Add at least one property, for example `[{\"name\":\"Alice\"}]`."));
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(Vec::new());
    writer
        .write_record(&headers)
        .map_err(|error| csv_error("write CSV header", &error))?;

    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| ToolError::new("JSON rows must all be objects."))?;
        let values = headers
            .iter()
            .map(|header| object.get(header).map_or_else(String::new, value_to_cell))
            .collect::<Vec<_>>();
        writer
            .write_record(&values)
            .map_err(|error| csv_error("write CSV row", &error))?;
    }

    finish_writer(writer)
}

fn arrays_to_csv(rows: &[Value], delimiter: u8) -> Result<String> {
    let width = rows.first().and_then(Value::as_array).map_or(0, Vec::len);
    for (index, row) in rows.iter().enumerate() {
        let Some(array) = row.as_array() else {
            continue;
        };
        if array.len() != width {
            return Err(ToolError::new(format!(
                "JSON row {} has {} values; row 1 has {width}.",
                index + 1,
                array.len()
            ))
            .with_hint("Every array row must have the same number of values."));
        }
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(Vec::new());
    for row in rows {
        let Some(array) = row.as_array() else {
            continue;
        };
        let values = array.iter().map(value_to_cell).collect::<Vec<_>>();
        writer
            .write_record(&values)
            .map_err(|error| csv_error("write CSV row", &error))?;
    }

    finish_writer(writer)
}

fn value_to_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| String::new())
        }
    }
}

fn finish_writer(writer: csv::Writer<Vec<u8>>) -> Result<String> {
    let bytes = writer
        .into_inner()
        .map_err(|error| ToolError::new(format!("Could not finish CSV output: {error}.")))?;
    let text =
        String::from_utf8(bytes).map_err(|_| ToolError::new("CSV output was not valid UTF-8."))?;
    Ok(text.trim_end_matches('\n').to_owned())
}

fn csv_error(action: &str, error: &csv::Error) -> ToolError {
    let position = error
        .position()
        .map(|position| format!(" at line {}", position.line()))
        .unwrap_or_default();
    ToolError::new(format!("Could not {action}{position}: {error}."))
        .with_hint("Check the delimiter and make sure every record is valid CSV.")
}

/// Parses a one character CSV delimiter; `\\t` and `tab` are accepted.
pub fn parse_delimiter(value: &str) -> Result<u8> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(b',');
    }
    let character = match trimmed.to_ascii_lowercase().as_str() {
        "\\t" | "tab" => '\t',
        "\\0" => '\0',
        _ => {
            let mut characters = trimmed.chars();
            let Some(character) = characters.next() else {
                return Ok(b',');
            };
            if characters.next().is_some() {
                return Err(
                    ToolError::new("The CSV delimiter must be exactly one character.")
                        .with_hint("Use `,`, `;`, `|`, a tab, or `\\t`."),
                );
            }
            character
        }
    };
    if !character.is_ascii() {
        return Err(ToolError::new(
            "The CSV delimiter must be an ASCII character.",
        ));
    }
    Ok(character as u8)
}

#[cfg(test)]
mod tests {
    use super::{from_json, parse_delimiter, to_json};

    #[test]
    fn converts_csv_to_json() -> Result<(), Box<dyn std::error::Error>> {
        let output = to_json("name,age\nAlice,30\nBob,25", b',', true)?;
        assert!(output.contains("\"name\": \"Alice\""));
        assert!(output.contains("\"age\": \"30\""));
        assert!(output.starts_with('['));
        Ok(())
    }

    #[test]
    fn handles_quoted_values_and_headless_csv() -> Result<(), Box<dyn std::error::Error>> {
        let output = to_json("\"a,b\",\"line\nbreak\"", b',', false)?;
        assert!(output.contains("a,b"));
        assert!(output.contains("line\\nbreak"));
        Ok(())
    }

    #[test]
    fn converts_json_to_csv() -> Result<(), Box<dyn std::error::Error>> {
        let output = from_json(
            r#"[{"name":"Alice","age":30},{"name":"Bob","age":null}]"#,
            b',',
        )?;
        assert!(output.contains("name"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        Ok(())
    }

    #[test]
    fn parses_delimiters() {
        assert_eq!(parse_delimiter(",").unwrap_or_default(), b',');
        assert_eq!(parse_delimiter("\\t").unwrap_or_default(), b'\t');
        assert_eq!(parse_delimiter("TAB").unwrap_or_default(), b'\t');
        assert_eq!(parse_delimiter("X").unwrap_or_default(), b'X');
        assert_eq!(parse_delimiter("x").unwrap_or_default(), b'x');
        assert!(parse_delimiter("--").is_err());
        assert!(from_json("{}", b',').is_err());
    }
}

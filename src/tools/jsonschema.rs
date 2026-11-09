//! JSON Schema validation with no external retrieval.
use super::limits;
use crate::error::{Result, ToolError};
use serde_json::Value;

pub const MAX_INPUT: usize = limits::MIB;
pub const MAX_ERRORS: usize = 100;

struct NoExternalResources;

impl ::jsonschema::Retrieve for NoExternalResources {
    fn retrieve(
        &self,
        _uri: &::jsonschema::Uri<String>,
    ) -> std::result::Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Err("External schema references are disabled; use internal # references.".into())
    }
}

#[derive(Debug)]
pub struct ValidationReport {
    pub valid: bool,
    pub text: String,
}

pub fn validate(schema: &str, instance: &str) -> Result<ValidationReport> {
    let schema = parse(schema, "Schema")?;
    let instance = parse(instance, "JSON instance")?;
    let validator = ::jsonschema::options()
        .with_retriever(NoExternalResources)
        .with_pattern_options(::jsonschema::PatternOptions::fancy_regex()
            .backtrack_limit(100_000).size_limit(limits::MIB).dfa_size_limit(limits::MIB))
        .build(&schema).map_err(|error| {
            ToolError::new(format!("Invalid schema: {error}."))
                .with_hint("Use a valid JSON Schema with internal references; external resources are disabled.")
        })?;
    let mut errors = validator.iter_errors(&instance);
    let mut lines = Vec::new();
    for error in errors.by_ref().take(MAX_ERRORS) {
        // Bound displayed messages independently of the input size. Paths are
        // JSON-quoted so the root pointer ("") and control characters are clear.
        let message = error.to_string();
        let mut short: String = message.chars().take(2048).collect();
        if short.len() < message.len() {
            short.push('…');
        }
        lines.push(format!(
            "{}: {short}",
            serde_json::to_string(&error.instance_path().to_string())
                .map_err(|e| ToolError::new(format!("Could not format validation path: {e}.")))?
        ));
    }
    let valid = lines.is_empty();
    if errors.next().is_some() {
        lines.push(format!("Further errors omitted (limit {MAX_ERRORS})."));
    }
    Ok(ValidationReport {
        valid,
        text: if valid {
            "Valid.".to_owned()
        } else {
            lines.join("\n")
        },
    })
}

fn parse(text: &str, label: &str) -> Result<Value> {
    limits::check(text, MAX_INPUT, label)?;
    serde_json::from_str(text).map_err(|error| {
        ToolError::new(format!("Invalid {label} JSON: {error}."))
            .with_hint("Provide valid JSON within the 1 MiB size and parser nesting limits.")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_types_required_enum_pattern_and_arrays() -> Result<()> {
        let schema = r#"{"type":"object","required":["name"],"properties":{"name":{"type":"string","pattern":"^[A-Z]"},"kind":{"enum":["a","b"]},"items":{"type":"array","items":{"type":"integer"}}}}"#;
        assert!(validate(schema, r#"{"name":"Ada","kind":"a","items":[1,2]}"#)?.valid);
        for input in [
            "42",
            "{}",
            r#"{"name":"lower"}"#,
            r#"{"name":"A","kind":"c"}"#,
            r#"{"name":"A","items":["bad"]}"#,
        ] {
            assert!(!validate(schema, input)?.valid, "{input}");
        }
        assert!(validate(schema, r#"{"name":"A","items":["bad"]}"#)?
            .text
            .contains("/items/0"));
        assert!(validate(r#"{"type":"integer"}"#, "true")?
            .text
            .starts_with("\"\":"));
        Ok(())
    }

    #[test]
    fn internal_refs_and_precise_numbers() -> Result<()> {
        let schema = r##"{"$defs":{"number":{"type":"integer","minimum":123456789012345678901234567890}},"$ref":"#/$defs/number"}"##;
        assert!(validate(schema, "123456789012345678901234567890")?.valid);
        assert!(!validate(schema, "123456789012345678901234567889")?.valid);
        Ok(())
    }

    #[test]
    fn rejects_invalid_schema_and_external_resources() {
        for schema in [
            "{",
            r#"{"type":123}"#,
            r#"{"$ref":"https://example.invalid/schema"}"#,
            r#"{"$ref":"file:///secret.json"}"#,
            r#"{"$ref":"other.json"}"#,
        ] {
            assert!(validate(schema, "{}").is_err(), "{schema}");
        }
        assert!(validate("{}", "{").is_err());
        assert!(validate(&" ".repeat(MAX_INPUT + 1), "{}").is_err());
        assert!(validate("{}", &" ".repeat(MAX_INPUT + 1)).is_err());
    }

    #[test]
    fn caps_validation_errors() -> Result<()> {
        let instance = format!("[{}]", vec!["false"; 101].join(","));
        let report = validate(r#"{"items":{"type":"string"}}"#, &instance)?;
        assert!(!report.valid);
        assert_eq!(report.text.lines().count(), 101);
        assert!(report.text.contains("Further errors omitted"));
        Ok(())
    }
}

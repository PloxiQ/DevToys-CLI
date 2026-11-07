use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::Value;

use crate::error::{Result, ToolError};
use crate::tools::timestamp::relative;

pub const SIGNATURE_WARNING: &str =
    "Signature is NOT verified. This tool decodes the token for inspection only.";

#[derive(Debug, Clone)]
pub struct DecodedJwt {
    pub header: Value,
    pub payload: Value,
    pub header_raw: String,
    pub payload_raw: String,
    pub signature_length: usize,
}

impl DecodedJwt {
    /// Registered claims and header fields formatted for humans.
    #[must_use]
    pub fn claims(&self) -> Vec<(String, String)> {
        let mut output = Vec::new();
        for key in ["alg", "typ", "kid"] {
            if let Some(value) = self.header.get(key) {
                output.push((key.to_owned(), value_to_text(value)));
            }
        }
        for key in ["iss", "sub", "aud", "exp", "iat", "nbf", "jti"] {
            if let Some(value) = self.payload.get(key) {
                let text = if matches!(key, "exp" | "iat" | "nbf") {
                    timestamp_claim(value).unwrap_or_else(|| value_to_text(value))
                } else {
                    value_to_text(value)
                };
                output.push((key.to_owned(), text));
            }
        }
        output
    }
}

/// Decodes a compact JWS/JWT without verifying its signature.
pub fn decode(token: &str) -> Result<DecodedJwt> {
    let compact: String = token
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    if compact.is_empty() {
        return Err(ToolError::new("No JWT provided.")
            .with_hint("Pass a compact JWT or pipe one via stdin."));
    }

    let parts: Vec<&str> = compact.split('.').collect();
    if parts.len() != 3 {
        let hint = if parts.len() == 5 {
            "This looks like a JWE (5 parts). DevToys CLI currently inspects signed JWTs (3 parts)."
        } else {
            "A compact JWT has three dot-separated parts: header.payload.signature."
        };
        return Err(ToolError::new(format!(
            "Expected a JWT with 3 parts, found {}.",
            parts.len()
        ))
        .with_hint(hint));
    }

    let header_raw = decode_segment(parts[0], "header")?;
    let payload_raw = decode_segment(parts[1], "payload")?;
    let header = parse_json(&header_raw, "header")?;
    let payload = parse_json(&payload_raw, "payload")?;

    Ok(DecodedJwt {
        header,
        payload,
        header_raw,
        payload_raw,
        signature_length: parts[2].len(),
    })
}

fn decode_segment(segment: &str, name: &str) -> Result<String> {
    if segment.trim().is_empty() {
        return Err(ToolError::new(format!("The JWT {name} segment is empty."))
            .with_hint("The token may be truncated."));
    }
    let cleaned = segment.trim_end_matches('=');
    let bytes = URL_SAFE_NO_PAD
        .decode(cleaned)
        .or_else(|_| STANDARD_NO_PAD.decode(cleaned))
        .map_err(|error| {
            ToolError::new(format!("Could not decode the JWT {name}: {error}.")).with_hint(
                "JWTs use base64url encoding. Check that the token was copied completely.",
            )
        })?;
    String::from_utf8(bytes).map_err(|_| {
        ToolError::new(format!("The JWT {name} is not valid UTF-8."))
            .with_hint("The token may be malformed or not be a JWT.")
    })
}

fn parse_json(raw: &str, name: &str) -> Result<Value> {
    serde_json::from_str(raw).map_err(|error| {
        ToolError::new(format!("The JWT {name} is not valid JSON: {error}."))
            .with_hint("The token may be corrupted; JWT header and payload must be JSON.")
    })
}

fn value_to_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    }
}

fn timestamp_claim(value: &Value) -> Option<String> {
    let seconds = value.as_i64()?;
    let time = DateTime::<Utc>::from_timestamp(seconds, 0)?;
    Some(format!(
        "{} ({})",
        time.to_rfc3339_opts(SecondsFormat::Secs, true),
        relative(time, Utc::now())
    ))
}

#[cfg(test)]
mod tests {
    use super::decode;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;

    #[test]
    fn decodes_header_and_payload() -> Result<(), Box<dyn std::error::Error>> {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD
            .encode(r#"{"sub":"1234567890","name":"Ada Lovelace","iat":1700000000}"#);
        let token = format!("{header}.{payload}.signature");
        let decoded = decode(&token)?;
        assert_eq!(decoded.header["alg"], "HS256");
        assert_eq!(decoded.payload["name"], "Ada Lovelace");
        assert_eq!(decoded.signature_length, 9);
        assert!(decoded
            .claims()
            .iter()
            .any(|(key, _)| key.as_str() == "iat"));
        Ok(())
    }

    #[test]
    fn rejects_malformed_tokens() {
        assert!(decode("").is_err());
        assert!(decode("abc.def").is_err());
        assert!(decode("abc.def.ghi.klm").is_err());
        assert!(decode("!!!.e30.sig").is_err());
    }
}

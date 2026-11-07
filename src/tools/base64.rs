use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use base64::Engine as _;

use crate::error::{Result, ToolError};

/// Encodes UTF-8 text as standard Base64 with padding.
#[must_use]
pub fn encode_text(input: &str) -> String {
    STANDARD.encode(input.as_bytes())
}

/// Encodes a hex string (whitespace, `:`, `-` and `_` separators allowed) as Base64.
pub fn encode_hex(input: &str) -> Result<String> {
    let bytes = parse_hex(input)?;
    Ok(STANDARD.encode(bytes))
}

/// Decodes Base64 to UTF-8 text.
pub fn decode_text(input: &str) -> Result<String> {
    let bytes = decode_bytes(input)?;
    String::from_utf8(bytes).map_err(|_| {
        ToolError::new("The decoded bytes are not valid UTF-8.")
            .with_hint("Use `--hex` or the `Base64 -> HEX` tool to inspect the raw bytes.")
    })
}

/// Decodes Base64 and returns lowercase hex.
pub fn decode_hex(input: &str) -> Result<String> {
    let bytes = decode_bytes(input)?;
    Ok(bytes_to_hex(&bytes))
}

/// Decodes standard or URL-safe Base64. Padding and whitespace are optional.
pub fn decode_bytes(input: &str) -> Result<Vec<u8>> {
    let cleaned: String = input
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    let unpadded = cleaned.trim_end_matches('=');
    if unpadded.len() % 4 == 1 {
        return Err(
            ToolError::new("Invalid Base64: the final block is incomplete.")
                .with_hint("Base64 length must be a multiple of 4, except for trailing padding."),
        );
    }
    if unpadded.is_empty() {
        return Ok(Vec::new());
    }

    // Accept both alphabets: standard (`+`/`/`) and URL-safe (`-`/`_`).
    STANDARD_NO_PAD
        .decode(unpadded)
        .or_else(|_| URL_SAFE_NO_PAD.decode(unpadded))
        .map_err(|error| {
            ToolError::new(format!("Invalid Base64: {error}."))
                .with_hint("Check for characters outside the Base64 alphabet or a truncated input.")
        })
}

/// Parses a hexadecimal string into bytes. `0x`, spaces, `:`, `-` and `_` are ignored.
pub fn parse_hex(input: &str) -> Result<Vec<u8>> {
    let mut text = input.trim();
    if let Some(stripped) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        text = stripped;
    }

    let mut digits = String::with_capacity(text.len());
    for character in text.chars() {
        if character.is_ascii_hexdigit() {
            digits.push(character);
        } else if character.is_ascii_whitespace() || matches!(character, ':' | '-' | '_') {
            continue;
        } else {
            return Err(
                ToolError::new(format!("Invalid hex character `{character}`.")).with_hint(
                    "Hex input may contain 0-9, a-f and separators such as spaces or colons.",
                ),
            );
        }
    }

    if !digits.len().is_multiple_of(2) {
        return Err(ToolError::new("Invalid hex: an odd number of digits.")
            .with_hint("Every byte needs exactly two hex digits, for example `48656c6c6f`."));
    }

    let mut bytes = Vec::with_capacity(digits.len() / 2);
    let (pairs, _) = digits.as_bytes().as_chunks::<2>();
    for pair in pairs {
        let high = hex_value(pair[0]).ok_or_else(|| ToolError::new("Invalid hex input."))?;
        let low = hex_value(pair[1]).ok_or_else(|| ToolError::new("Invalid hex input."))?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{decode_bytes, decode_hex, decode_text, encode_hex, encode_text};
    use crate::error::Result;

    #[test]
    fn text_round_trip() -> Result<()> {
        let encoded = encode_text("Hello, DevToys!");
        assert_eq!(encoded, "SGVsbG8sIERldlRveXMh");
        assert_eq!(decode_text(&encoded)?, "Hello, DevToys!");
        Ok(())
    }

    #[test]
    fn hex_round_trip() -> Result<()> {
        let encoded = encode_hex("48 65:6c-6c_6f")?;
        assert_eq!(encoded, "SGVsbG8=");
        assert_eq!(decode_hex(&encoded)?, "48656c6c6f");
        Ok(())
    }

    #[test]
    fn accepts_url_safe_and_unpadded_input() -> Result<()> {
        assert_eq!(decode_bytes("SGVsbG8")?, b"Hello");
        assert_eq!(decode_bytes("SGVsbG8=")?, b"Hello");
        Ok(())
    }

    #[test]
    fn rejects_bad_input() {
        assert!(decode_bytes("abcde").is_err());
        assert!(decode_text("//79").is_err());
        assert!(encode_hex("xyz").is_err());
        assert!(encode_hex("abc").is_err());
    }
}

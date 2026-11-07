use crate::error::{Result, ToolError};

/// Percent-encodes text for use in a URL (RFC 3986 unreserved characters stay as-is).
#[must_use]
pub fn encode(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(input.len());
    for &byte in input.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(byte as char);
        } else {
            output.push('%');
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
    output
}

/// Decodes percent-encoded text. `+` is treated as a space, form-style.
pub fn decode(input: &str) -> Result<String> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(
                        ToolError::new(format!("Incomplete percent escape at byte {i}."))
                            .with_hint("A percent escape needs two hex digits, for example `%20`."),
                    );
                }
                let high = hex_value(bytes[i + 1]).ok_or_else(|| {
                    ToolError::new(format!("Invalid percent escape at byte {i}."))
                        .with_hint("A percent escape needs two hex digits, for example `%20`.")
                })?;
                let low = hex_value(bytes[i + 2]).ok_or_else(|| {
                    ToolError::new(format!("Invalid percent escape at byte {i}."))
                        .with_hint("A percent escape needs two hex digits, for example `%20`.")
                })?;
                output.push((high << 4) | low);
                i += 3;
            }
            b'+' => {
                output.push(b' ');
                i += 1;
            }
            byte => {
                output.push(byte);
                i += 1;
            }
        }
    }

    String::from_utf8(output).map_err(|_| {
        ToolError::new("The decoded URL is not valid UTF-8.")
            .with_hint("Check that multi-byte characters are fully percent-encoded.")
    })
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};
    use crate::error::Result;

    #[test]
    fn encodes_reserved_characters() {
        assert_eq!(encode("a b+c/?"), "a%20b%2Bc%2F%3F");
        assert_eq!(encode("Привет"), "%D0%9F%D1%80%D0%B8%D0%B2%D0%B5%D1%82");
    }

    #[test]
    fn decodes_percent_escapes_and_plus() -> Result<()> {
        assert_eq!(decode("a%20b+c%2Fd")?, "a b c/d");
        Ok(())
    }

    #[test]
    fn round_trips_unicode() -> Result<()> {
        let text = "https://example.com/поиск?q=rust cli&ok=yes";
        assert_eq!(decode(&encode(text))?, text);
        Ok(())
    }

    #[test]
    fn rejects_broken_escapes() {
        assert!(decode("%2").is_err());
        assert!(decode("%GG").is_err());
        assert!(decode("%FF").is_err());
    }
}

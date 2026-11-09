//! RFC 4648 Base32 with strict padding and trailing-bit validation.
use super::limits;
use crate::error::{Result, ToolError};

pub const MAX_INPUT: usize = limits::MIB;
const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn encode(input: &str, padding: bool, lower: bool) -> Result<String> {
    limits::check(input, MAX_INPUT, "Base32 input")?;
    let mut output = String::with_capacity(input.len().div_ceil(5) * 8);
    let (mut buffer, mut bits) = (0u16, 0u32);
    for byte in input.bytes() {
        buffer = (buffer << 8) | u16::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            output.push(ALPHABET[usize::from((buffer >> bits) & 31)] as char);
        }
        buffer &= (1 << bits) - 1;
    }
    if bits > 0 {
        output.push(ALPHABET[usize::from((buffer << (5 - bits)) & 31)] as char);
    }
    if padding {
        while !output.len().is_multiple_of(8) {
            output.push('=');
        }
    }
    if lower {
        output.make_ascii_lowercase();
    }
    Ok(output)
}

pub fn decode(input: &str) -> Result<String> {
    limits::check(input, MAX_INPUT, "Base32 input")?;
    let cleaned: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let end = cleaned
        .iter()
        .position(|b| *b == b'=')
        .unwrap_or(cleaned.len());
    let padding = cleaned.len() - end;
    let expected_padding = match end % 8 {
        0 => 0,
        2 => 6,
        4 => 4,
        5 => 3,
        7 => 1,
        _ => return Err(invalid("incomplete final block")),
    };
    if padding > 0 && (padding != expected_padding || cleaned[end..].iter().any(|b| *b != b'=')) {
        return Err(invalid("bad padding"));
    }
    let mut output = Vec::with_capacity(end * 5 / 8);
    let (mut buffer, mut bits) = (0u16, 0u32);
    for byte in &cleaned[..end] {
        let value = match byte.to_ascii_uppercase() {
            b'A'..=b'Z' => byte.to_ascii_uppercase() - b'A',
            b'2'..=b'7' => byte - b'2' + 26,
            _ => return Err(invalid("character outside A-Z and 2-7")),
        };
        buffer = (buffer << 5) | u16::from(value);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
        }
        buffer &= (1 << bits) - 1;
    }
    if buffer != 0 {
        return Err(invalid("non-zero trailing bits"));
    }
    String::from_utf8(output).map_err(|_| {
        ToolError::new("Decoded Base32 is not valid UTF-8.")
            .with_hint("Decode a Base32 value containing UTF-8 text.")
    })
}

fn invalid(reason: &str) -> ToolError {
    ToolError::new(format!("Invalid Base32: {reason}."))
        .with_hint("Use RFC 4648 letters A-Z and digits 2-7, with either complete padding or none.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_4648_vectors() -> Result<()> {
        for (plain, encoded) in [
            ("", ""),
            ("f", "MY======"),
            ("fo", "MZXQ===="),
            ("foo", "MZXW6==="),
            ("foob", "MZXW6YQ="),
            ("fooba", "MZXW6YTB"),
            ("foobar", "MZXW6YTBOI======"),
        ] {
            assert_eq!(encode(plain, true, false)?, encoded);
            assert_eq!(decode(encoded)?, plain);
            assert_eq!(decode(encoded.trim_end_matches('='))?, plain);
            assert_eq!(
                encode(plain, false, true)?,
                encoded.trim_end_matches('=').to_ascii_lowercase()
            );
        }
        Ok(())
    }

    #[test]
    fn unicode_and_mixed_case() -> Result<()> {
        let input = "Привіт 🌍 e\u{301}";
        assert_eq!(decode(&encode(input, true, true)?)?, input);
        assert_eq!(decode(" mZ Xw6\r\n===\t")?, "foo");
        Ok(())
    }

    #[test]
    fn rejects_malformed_and_large_input() {
        for input in [
            "=",
            "MY=",
            "MY=======",
            "MY======A",
            "M",
            "MZX",
            "MZXW6Y",
            "MZ======",
            "M0======",
            "é",
            "74======",
        ] {
            assert!(decode(input).is_err(), "{input}");
        }
        assert!(encode(&"a".repeat(MAX_INPUT + 1), true, false).is_err());
        assert!(decode(&"A".repeat(MAX_INPUT + 1)).is_err());
    }
}

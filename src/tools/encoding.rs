use crate::error::{Result, ToolError};

const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";
const HEX_UPPER: &[u8; 16] = b"0123456789ABCDEF";

#[must_use]
pub fn hex_encode(input: &str, uppercase: bool) -> String {
    hex_encode_bytes(input.as_bytes(), uppercase)
}

#[must_use]
pub fn hex_encode_bytes(bytes: &[u8], uppercase: bool) -> String {
    let alphabet = if uppercase { HEX_UPPER } else { HEX_LOWER };
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(alphabet[(byte >> 4) as usize] as char);
        output.push(alphabet[(byte & 0x0f) as usize] as char);
    }
    output
}

pub fn hex_decode(input: &str) -> Result<String> {
    let bytes = hex_decode_bytes(input)?;
    String::from_utf8(bytes).map_err(|_| {
        ToolError::new("The decoded bytes are not valid UTF-8.")
            .with_hint("Use `devtoys base64 decode --hex` or a hex viewer to inspect binary data.")
    })
}

pub fn hex_decode_bytes(input: &str) -> Result<Vec<u8>> {
    let cleaned: String = input
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .chars()
        .filter(|character| !character.is_whitespace() && !matches!(character, '_' | ':' | '-'))
        .collect();

    if cleaned.is_empty() {
        return Ok(Vec::new());
    }
    if !cleaned.len().is_multiple_of(2) {
        return Err(ToolError::new(format!(
            "Hex input has an odd number of digits ({}).",
            cleaned.len()
        ))
        .with_hint("Each byte is two hex digits, for example `48 65 6c 6c 6f`."));
    }

    let mut bytes = Vec::with_capacity(cleaned.len() / 2);
    let (pairs, _) = cleaned.as_bytes().as_chunks::<2>();
    for (index, pair) in pairs.iter().enumerate() {
        let high = hex_digit(pair[0], index * 2)?;
        let low = hex_digit(pair[1], index * 2 + 1)?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_digit(byte: u8, position: usize) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        other => Err(ToolError::new(format!(
            "`{}` is not a hex digit at position {}.",
            other as char,
            position + 1
        ))
        .with_hint("Hex digits are 0-9, a-f and A-F.")),
    }
}

#[must_use]
pub fn binary_encode(input: &str) -> String {
    let mut output = String::with_capacity(input.len() * 8);
    for byte in input.as_bytes() {
        for bit in (0..8).rev() {
            output.push(if byte & (1 << bit) == 0 { '0' } else { '1' });
        }
    }
    output
}

pub fn binary_decode(input: &str) -> Result<String> {
    let cleaned: String = input
        .trim()
        .trim_start_matches("0b")
        .trim_start_matches("0B")
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '_')
        .collect();

    if cleaned.is_empty() {
        return Ok(String::new());
    }
    if !cleaned.len().is_multiple_of(8) {
        return Err(ToolError::new(format!(
            "Binary input has {} bits; it must be a multiple of 8.",
            cleaned.len()
        ))
        .with_hint("Each byte is eight bits, for example `01001000 01101001`."));
    }

    let mut bytes = Vec::with_capacity(cleaned.len() / 8);
    let (chunks, _) = cleaned.as_bytes().as_chunks::<8>();
    for (index, chunk) in chunks.iter().enumerate() {
        let mut value = 0u8;
        for (offset, byte) in chunk.iter().enumerate() {
            match byte {
                b'0' => {}
                b'1' => value |= 1 << (7 - offset),
                other => {
                    return Err(ToolError::new(format!(
                        "`{}` is not a binary digit at position {}.",
                        *other as char,
                        index * 8 + offset + 1
                    ))
                    .with_hint("Binary input may only contain 0 and 1."));
                }
            }
        }
        bytes.push(value);
    }

    String::from_utf8(bytes).map_err(|_| {
        ToolError::new("The decoded bytes are not valid UTF-8.").with_hint(
            "Use `devtoys base64 encode --hex` or a binary viewer to inspect binary data.",
        )
    })
}

#[must_use]
pub fn html_encode(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
    output
}

pub fn html_decode(input: &str) -> Result<String> {
    let characters: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;

    while index < characters.len() {
        if characters[index] == '&' {
            if let Some((decoded, consumed)) = decode_entity(&characters[index + 1..]) {
                output.push(decoded);
                index += 1 + consumed;
                continue;
            }
        }
        output.push(characters[index]);
        index += 1;
    }
    Ok(output)
}

fn decode_entity(characters: &[char]) -> Option<(char, usize)> {
    let semicolon = characters
        .iter()
        .take(32)
        .position(|character| *character == ';')?;
    let name: String = characters[..semicolon].iter().collect();
    let decoded = if let Some(number) = name.strip_prefix('#') {
        let value = if let Some(hex) = number
            .strip_prefix('x')
            .or_else(|| number.strip_prefix('X'))
        {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            number.parse::<u32>().ok()?
        };
        if value == 0 {
            '\u{fffd}'
        } else {
            char::from_u32(value)?
        }
    } else {
        named_entity(&name)?
    };
    Some((decoded, semicolon + 1))
}

fn named_entity(name: &str) -> Option<char> {
    let lower = name.to_ascii_lowercase();
    let value = match lower.as_str() {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "copy" => '©',
        "reg" => '®',
        "trade" => '™',
        "hellip" => '…',
        "mdash" => '—',
        "ndash" => '–',
        "lsquo" => '‘',
        "rsquo" => '’',
        "ldquo" => '“',
        "rdquo" => '”',
        "laquo" => '«',
        "raquo" => '»',
        "times" => '×',
        "divide" => '÷',
        "plusmn" => '±',
        "deg" => '°',
        "micro" => 'µ',
        "para" => '¶',
        "sect" => '§',
        "middot" => '·',
        "bull" => '•',
        "dagger" => '†',
        "ddagger" => '‡',
        "euro" => '€',
        "pound" => '£',
        "yen" => '¥',
        "cent" => '¢',
        "larr" => '←',
        "rarr" => '→',
        "uarr" => '↑',
        "darr" => '↓',
        "harr" => '↔',
        "spades" => '♠',
        "clubs" => '♣',
        "hearts" => '♥',
        "diams" => '♦',
        "check" => '✓',
        "cross" => '✗',
        "star" => '☆',
        "starf" => '★',
        "female" => '♀',
        "male" => '♂',
        "sun" => '☀',
        "phone" => '☎',
        "email" => '✉',
        "frac12" => '½',
        "frac14" => '¼',
        "frac34" => '¾',
        "sup2" => '²',
        "sup3" => '³',
        "ordm" => 'º',
        "ordf" => 'ª',
        "iquest" => '¿',
        "iexcl" => '¡',
        "szlig" => 'ß',
        "aacute" => 'á',
        "agrave" => 'à',
        "acirc" => 'â',
        "auml" => 'ä',
        "atilde" => 'ã',
        "aring" => 'å',
        "aelig" => 'æ',
        "ccedil" => 'ç',
        "eacute" => 'é',
        "egrave" => 'è',
        "ecirc" => 'ê',
        "euml" => 'ë',
        "iacute" => 'í',
        "igrave" => 'ì',
        "icirc" => 'î',
        "iuml" => 'ï',
        "ntilde" => 'ñ',
        "oacute" => 'ó',
        "ograve" => 'ò',
        "ocirc" => 'ô',
        "ouml" => 'ö',
        "otilde" => 'õ',
        "oslash" => 'ø',
        "uacute" => 'ú',
        "ugrave" => 'ù',
        "ucirc" => 'û',
        "uuml" => 'ü',
        "yacute" => 'ý',
        "yuml" => 'ÿ',
        "alpha" => 'α',
        "beta" => 'β',
        "gamma" => 'γ',
        "delta" => 'δ',
        "pi" => 'π',
        "sigma" => 'σ',
        "omega" => 'ω',
        _ => return None,
    };

    if name
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        && value.is_lowercase()
    {
        let mut upper = value.to_uppercase();
        return upper.next();
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::{binary_decode, binary_encode, hex_decode, hex_encode, html_decode, html_encode};

    #[test]
    fn hex_round_trips_and_formats() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(hex_encode("Hello", false), "48656c6c6f");
        assert_eq!(hex_encode("Hello", true), "48656C6C6F");
        assert_eq!(hex_decode("48 65 6c 6c 6f")?, "Hello");
        assert_eq!(hex_decode("0x48656C6C6F")?, "Hello");
        assert!(hex_decode("abc").is_err());
        assert!(hex_decode("ff").is_err());
        Ok(())
    }

    #[test]
    fn binary_round_trips() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(binary_encode("Hi"), "0100100001101001");
        assert_eq!(binary_decode("01001000 01101001")?, "Hi");
        assert!(binary_decode("0101").is_err());
        assert!(binary_decode("01001002").is_err());
        Ok(())
    }

    #[test]
    fn html_round_trips_and_decodes_entities() -> Result<(), Box<dyn std::error::Error>> {
        let original = r#"<a href="x">Tom & Jerry's</a>"#;
        let encoded = html_encode(original);
        assert!(encoded.contains("&lt;a"));
        assert!(encoded.contains("&amp;"));
        assert_eq!(html_decode(&encoded)?, original);
        assert_eq!(html_decode("&#65;&#x42;&amp;")?, "AB&");
        assert_eq!(html_decode("&unknown;")?, "&unknown;");
        assert_eq!(html_decode("&Aacute;")?, "Á");
        Ok(())
    }
}

use crate::error::{Result, ToolError};

const BASES: [(u32, &str); 5] = [
    (2, "Binary"),
    (8, "Octal"),
    (10, "Decimal"),
    (16, "Hexadecimal"),
    (36, "Base 36"),
];

/// Converts an integer between bases 2 and 36.
///
/// When `to` is omitted, the value is shown in binary, octal, decimal,
/// hexadecimal and base 36.
pub fn convert_base(
    input: &str,
    from: Option<u32>,
    to: Option<u32>,
    uppercase: bool,
) -> Result<String> {
    let value = parse_integer(input, from)?;

    if let Some(base) = to {
        validate_base(base)?;
        return Ok(format_digits(value.1, base, uppercase, value.0));
    }

    let mut lines = Vec::with_capacity(BASES.len());
    for (base, label) in BASES {
        lines.push(format!(
            "{label:<12}: {}",
            format_digits(value.1, base, uppercase, value.0)
        ));
    }
    Ok(lines.join("\n"))
}

fn validate_base(base: u32) -> Result<()> {
    if !(2..=36).contains(&base) {
        return Err(ToolError::new(format!(
            "Base must be between 2 and 36, got {base}."
        )));
    }
    Ok(())
}

/// Returns `(negative, magnitude)`.
fn parse_integer(input: &str, from: Option<u32>) -> Result<(bool, u128)> {
    let trimmed = input.trim().replace('_', "");
    if trimmed.is_empty() {
        return Err(ToolError::new("No number provided.")
            .with_hint("Pass an integer such as `255`, `0xff`, `0b1010` or `0o755`."));
    }

    let (negative, unsigned) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed.strip_prefix('+').unwrap_or(&trimmed)),
    };
    if unsigned.is_empty() {
        return Err(ToolError::new("No digits provided after the sign."));
    }

    let (base, digits) = match from {
        Some(base) => {
            validate_base(base)?;
            (base, strip_prefix(unsigned, base))
        }
        None => {
            if let Some(rest) = unsigned
                .strip_prefix("0x")
                .or_else(|| unsigned.strip_prefix("0X"))
            {
                (16, rest)
            } else if let Some(rest) = unsigned
                .strip_prefix("0b")
                .or_else(|| unsigned.strip_prefix("0B"))
            {
                (2, rest)
            } else if let Some(rest) = unsigned
                .strip_prefix("0o")
                .or_else(|| unsigned.strip_prefix("0O"))
            {
                (8, rest)
            } else {
                (10, unsigned)
            }
        }
    };

    if digits.is_empty() {
        return Err(ToolError::new("No digits provided.")
            .with_hint("Prefix binary with `0b`, octal with `0o`, and hexadecimal with `0x`."));
    }

    let mut value = 0u128;
    for (index, character) in digits.chars().enumerate() {
        let Some(digit) = character.to_digit(base) else {
            return Err(
                ToolError::new(format!("`{character}` is not a valid base-{base} digit."))
                    .with_hint(format!(
                        "Digit {} is invalid. Use digits 0-{} for base {base}.",
                        index + 1,
                        (base - 1).min(35)
                    )),
            );
        };
        value = value
            .checked_mul(u128::from(base))
            .and_then(|current| current.checked_add(u128::from(digit)))
            .ok_or_else(|| {
                ToolError::new("The number is too large to convert (maximum is u128).")
            })?;
    }

    Ok((negative && value != 0, value))
}

fn strip_prefix(value: &str, base: u32) -> &str {
    match base {
        16 => value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
            .unwrap_or(value),
        2 => value
            .strip_prefix("0b")
            .or_else(|| value.strip_prefix("0B"))
            .unwrap_or(value),
        8 => value
            .strip_prefix("0o")
            .or_else(|| value.strip_prefix("0O"))
            .unwrap_or(value),
        _ => value,
    }
}

fn format_digits(mut value: u128, base: u32, uppercase: bool, negative: bool) -> String {
    if value == 0 {
        return "0".to_owned();
    }

    const LOWER: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    const UPPER: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let alphabet = if uppercase { UPPER } else { LOWER };
    let mut digits = Vec::new();
    while value > 0 {
        let digit = (value % u128::from(base)) as usize;
        digits.push(alphabet[digit] as char);
        value /= u128::from(base);
    }
    if negative {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

/// Converts a byte count or a human-readable size such as `1.5 MiB`.
pub fn convert_bytes(input: &str, binary_only: bool) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ToolError::new("No byte count provided.")
            .with_hint("Pass a value such as `1536`, `1.5 MB` or `2 GiB`."));
    }

    let split = trimmed
        .char_indices()
        .find(|(_, character)| {
            !character.is_ascii_digit() && *character != '.' && *character != ','
        })
        .map_or(trimmed.len(), |(index, _)| index);
    let number = trimmed[..split].replace(',', "");
    let unit = trimmed[split..].trim().to_ascii_lowercase();

    if number.is_empty() {
        return Err(ToolError::new("A byte count must start with a number.")
            .with_hint("Examples: `1536`, `1.5 MB`, `2 GiB`."));
    }

    let value: f64 = number.parse().map_err(|_| {
        ToolError::new(format!("`{number}` is not a valid number."))
            .with_hint("Use a plain number or decimal, such as `1536` or `1.5 MB`.")
    })?;
    if !value.is_finite() || value < 0.0 {
        return Err(ToolError::new(
            "Byte counts must be finite and non-negative.",
        ));
    }

    let multiplier = byte_multiplier(&unit)?;
    let bytes = value * multiplier;
    if bytes > u128::MAX as f64 {
        return Err(ToolError::new("The byte count is too large to display."));
    }
    let bytes = bytes.round() as u128;

    let mut lines = vec![format!("Bytes       : {}", format_thousands(bytes))];
    if !binary_only {
        lines.push(format!("SI (1000)   : {}", human_bytes(bytes, 1000.0)));
    }
    lines.push(format!("IEC (1024)  : {}", human_bytes(bytes, 1024.0)));
    lines.push(format!(
        "Bits        : {}",
        format_thousands(bytes.saturating_mul(8))
    ));
    Ok(lines.join("\n"))
}

fn byte_multiplier(unit: &str) -> Result<f64> {
    let multiplier = match unit {
        "" | "b" | "byte" | "bytes" => 1.0,
        "k" | "kb" | "kilobyte" | "kilobytes" => 1_000.0,
        "m" | "mb" | "megabyte" | "megabytes" => 1_000_000.0,
        "g" | "gb" | "gigabyte" | "gigabytes" => 1_000_000_000.0,
        "t" | "tb" | "terabyte" | "terabytes" => 1_000_000_000_000.0,
        "p" | "pb" | "petabyte" | "petabytes" => 1_000_000_000_000_000.0,
        "kib" | "kibibyte" | "kibibytes" => 1024.0,
        "mib" | "mebibyte" | "mebibytes" => 1_048_576.0,
        "gib" | "gibibyte" | "gibibytes" => 1_073_741_824.0,
        "tib" | "tebibyte" | "tebibytes" => 1_099_511_627_776.0,
        "pib" | "pebibyte" | "pebibytes" => 1_125_899_906_842_624.0,
        _ => {
            return Err(ToolError::new(format!("Unknown size unit `{unit}`."))
                .with_hint("Use B, KB, MB, GB, TB, PB, KiB, MiB, GiB, TiB or PiB."));
        }
    };
    Ok(multiplier)
}

fn human_bytes(bytes: u128, base: f64) -> String {
    if bytes == 0 {
        return "0 B".to_owned();
    }
    let value = bytes as f64;
    let exponent = (value.ln() / base.ln()).floor().clamp(0.0, 5.0) as usize;
    if exponent == 0 {
        return format!("{} B", format_thousands(bytes));
    }
    let units = if (base - 1000.0).abs() < f64::EPSILON {
        ["B", "kB", "MB", "GB", "TB", "PB"]
    } else {
        ["B", "KiB", "MiB", "GiB", "TiB", "PiB"]
    };
    let scaled = value / base.powi(exponent as i32);
    format!("{scaled:.2} {}", units[exponent])
}

pub fn format_thousands(value: u128) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{convert_base, convert_bytes};

    #[test]
    fn converts_between_common_bases() -> Result<(), Box<dyn std::error::Error>> {
        let output = convert_base("255", None, None, false)?;
        assert!(output.contains("Binary      : 11111111"));
        assert!(output.contains("Octal       : 377"));
        assert!(output.contains("Decimal     : 255"));
        assert!(output.contains("Hexadecimal : ff"));

        assert_eq!(convert_base("0xff", None, Some(2), false)?, "11111111");
        assert_eq!(convert_base("1010", Some(2), Some(16), true)?, "A");
        assert_eq!(convert_base("-255", None, Some(16), false)?, "-ff");
        Ok(())
    }

    #[test]
    fn rejects_invalid_digits_and_bases() {
        assert!(convert_base("2", Some(2), None, false).is_err());
        assert!(convert_base("255", Some(1), None, false).is_err());
        assert!(convert_base("zz", Some(16), None, false).is_err());
    }

    #[test]
    fn converts_byte_units() -> Result<(), Box<dyn std::error::Error>> {
        let output = convert_bytes("1536", false)?;
        assert!(output.contains("Bytes       : 1,536"));
        assert!(output.contains("SI (1000)   : 1.54 kB"));
        assert!(output.contains("IEC (1024)  : 1.50 KiB"));

        let output = convert_bytes("1.5 MiB", true)?;
        assert!(output.contains("Bytes       : 1,572,864"));
        assert!(!output.contains("SI (1000)"));
        Ok(())
    }

    #[test]
    fn rejects_unknown_units() {
        assert!(convert_bytes("10 XB", false).is_err());
        assert!(convert_bytes("abc", false).is_err());
    }
}

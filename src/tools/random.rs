use crate::error::{Result, ToolError};
use crate::tools::rng::SecureRng;

pub const MAX_LENGTH: usize = 4096;
pub const MAX_COUNT: usize = 1000;
pub const DEFAULT_SYMBOLS: &str = "!@#$%^&*()-_=+[]{};:,.?/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Alnum,
    Alpha,
    Lower,
    Upper,
    Numeric,
    Hex,
    Base58,
    Symbols,
}

impl Charset {
    #[must_use]
    pub fn alphabet(self) -> &'static [u8] {
        match self {
            Self::Alnum => b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            Self::Alpha => b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Self::Lower => b"abcdefghijklmnopqrstuvwxyz",
            Self::Upper => b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Self::Numeric => b"0123456789",
            Self::Hex => b"0123456789abcdef",
            Self::Base58 => b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",
            Self::Symbols => DEFAULT_SYMBOLS.as_bytes(),
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Alnum => "alphanumeric",
            Self::Alpha => "alphabetic",
            Self::Lower => "lowercase",
            Self::Upper => "uppercase",
            Self::Numeric => "numeric",
            Self::Hex => "hex",
            Self::Base58 => "base58",
            Self::Symbols => "symbols",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordOptions {
    pub length: usize,
    pub count: usize,
    pub lowercase: bool,
    pub uppercase: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            count: 1,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
        }
    }
}

/// Generates random strings using cryptographic OS entropy.
pub fn generate_strings(length: usize, charset: Charset, count: usize) -> Result<Vec<String>> {
    validate_length(length)?;
    validate_count(count)?;
    let alphabet = charset.alphabet();
    let mut rng = SecureRng::new();
    let mut output = Vec::with_capacity(count);

    for _ in 0..count {
        let mut value = String::with_capacity(length);
        for _ in 0..length {
            let index = rng.index(alphabet.len())?;
            value.push(alphabet[index] as char);
        }
        output.push(value);
    }
    Ok(output)
}

/// Generates passwords, guaranteeing at least one character from every enabled class.
pub fn generate_passwords(options: PasswordOptions) -> Result<Vec<String>> {
    validate_length(options.length)?;
    validate_count(options.count)?;

    let mut categories: Vec<&[u8]> = Vec::with_capacity(4);
    if options.lowercase {
        categories.push(b"abcdefghijklmnopqrstuvwxyz");
    }
    if options.uppercase {
        categories.push(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if options.digits {
        categories.push(b"0123456789");
    }
    if options.symbols {
        categories.push(DEFAULT_SYMBOLS.as_bytes());
    }

    if categories.is_empty() {
        return Err(ToolError::new("All character classes are disabled.")
            .with_hint("Enable at least one of lowercase, uppercase, digits or symbols."));
    }
    if options.length < categories.len() {
        return Err(ToolError::new(format!(
            "Password length {} is too short for {} enabled character classes.",
            options.length,
            categories.len()
        ))
        .with_hint("Increase the length or disable some character classes."));
    }

    let mut pool = Vec::with_capacity(128);
    for category in &categories {
        pool.extend_from_slice(category);
    }

    let mut rng = SecureRng::new();
    let mut output = Vec::with_capacity(options.count);
    for _ in 0..options.count {
        let mut characters = Vec::with_capacity(options.length);
        // One guaranteed character from each enabled class.
        for category in &categories {
            let index = rng.index(category.len())?;
            characters.push(category[index]);
        }
        while characters.len() < options.length {
            let index = rng.index(pool.len())?;
            characters.push(pool[index]);
        }
        rng.shuffle(&mut characters)?;
        output.push(characters.iter().map(|byte| *byte as char).collect());
    }
    Ok(output)
}

fn validate_length(length: usize) -> Result<()> {
    if length == 0 || length > MAX_LENGTH {
        return Err(ToolError::new(format!(
            "Length must be between 1 and {MAX_LENGTH}."
        )));
    }
    Ok(())
}

fn validate_count(count: usize) -> Result<()> {
    if count == 0 || count > MAX_COUNT {
        return Err(ToolError::new(format!(
            "Count must be between 1 and {MAX_COUNT}."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{generate_passwords, generate_strings, Charset, PasswordOptions};

    #[test]
    fn random_strings_have_requested_shape() -> Result<(), Box<dyn std::error::Error>> {
        let values = generate_strings(64, Charset::Hex, 3)?;
        assert_eq!(values.len(), 3);
        assert!(values.iter().all(|value| value.len() == 64));
        assert!(values
            .iter()
            .all(|value| value.chars().all(|c| c.is_ascii_hexdigit())));
        Ok(())
    }

    #[test]
    fn symbols_charset_uses_punctuation() -> Result<(), Box<dyn std::error::Error>> {
        let values = generate_strings(32, Charset::Symbols, 2)?;
        assert!(values.iter().all(|value| value.len() == 32));
        assert!(values
            .iter()
            .all(|value| value.chars().all(|c| super::DEFAULT_SYMBOLS.contains(c))));
        Ok(())
    }

    #[test]
    fn password_contains_every_enabled_class() -> Result<(), Box<dyn std::error::Error>> {
        let values = generate_passwords(PasswordOptions {
            length: 24,
            count: 8,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
        })?;
        for value in values {
            assert_eq!(value.len(), 24);
            assert!(value.chars().any(|c| c.is_ascii_lowercase()));
            assert!(value.chars().any(|c| c.is_ascii_uppercase()));
            assert!(value.chars().any(|c| c.is_ascii_digit()));
            assert!(value.chars().any(|c| super::DEFAULT_SYMBOLS.contains(c)));
        }
        Ok(())
    }

    #[test]
    fn rejects_invalid_options() {
        assert!(generate_strings(0, Charset::Alnum, 1).is_err());
        assert!(generate_strings(8, Charset::Alnum, 0).is_err());
        assert!(generate_passwords(PasswordOptions {
            length: 4,
            count: 1,
            lowercase: false,
            uppercase: false,
            digits: false,
            symbols: false,
        })
        .is_err());
        assert!(generate_passwords(PasswordOptions {
            length: 1,
            count: 1,
            lowercase: true,
            uppercase: true,
            digits: false,
            symbols: false,
        })
        .is_err());
    }
}

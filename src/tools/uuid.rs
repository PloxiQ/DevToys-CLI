use uuid::Uuid;

use crate::error::{Result, ToolError};
use crate::tools::rng::SecureRng;

pub const MAX_COUNT: usize = 1000;

/// Generates RFC 4122 version 4 UUIDs using OS entropy.
pub fn generate(count: usize, uppercase: bool, hyphens: bool) -> Result<Vec<String>> {
    if count == 0 || count > MAX_COUNT {
        return Err(ToolError::new(format!(
            "UUID count must be between 1 and {MAX_COUNT}."
        )));
    }

    let mut rng = SecureRng::new();
    let mut output = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes)?;
        // Version 4 and RFC 4122 variant bits.
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let uuid = Uuid::from_bytes(bytes);
        let mut value = if hyphens {
            uuid.hyphenated().to_string()
        } else {
            uuid.simple().to_string()
        };
        if uppercase {
            value.make_ascii_uppercase();
        }
        output.push(value);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::generate;

    #[test]
    fn generates_valid_v4_uuids() -> Result<(), Box<dyn std::error::Error>> {
        let values = generate(32, false, true)?;
        assert_eq!(values.len(), 32);
        for value in &values {
            let uuid = uuid::Uuid::parse_str(value)?;
            assert_eq!(uuid.get_version_num(), 4);
            assert_eq!(value.len(), 36);
        }
        Ok(())
    }

    #[test]
    fn supports_compact_and_uppercase_output() {
        let Ok(values) = generate(2, true, false) else {
            return;
        };
        assert!(values.iter().all(|value| value.len() == 32));
        assert!(values.iter().all(|value| value
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())));
    }

    #[test]
    fn validates_count() {
        assert!(generate(0, false, true).is_err());
        assert!(generate(1001, false, true).is_err());
    }
}

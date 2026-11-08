use chrono::Utc;
use uuid::Uuid;

use crate::error::{Result, ToolError};
use crate::tools::rng::SecureRng;

pub const MAX_COUNT: usize = 1000;

/// Generates RFC 4122 version 4 UUIDs using OS entropy.
pub fn generate(count: usize, uppercase: bool, hyphens: bool) -> Result<Vec<String>> {
    validate_count(count)?;

    let mut rng = SecureRng::new();
    let mut output = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes)?;
        // Version 4 and RFC 4122 variant bits.
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        output.push(format_uuid(Uuid::from_bytes(bytes), uppercase, hyphens));
    }
    Ok(output)
}

/// Generates time-ordered version 7 UUIDs using OS entropy.
pub fn generate_v7(count: usize, uppercase: bool, hyphens: bool) -> Result<Vec<String>> {
    validate_count(count)?;

    let timestamp = match u64::try_from(Utc::now().timestamp_millis()) {
        Ok(value) => value & 0x0000_ffff_ffff_ffff,
        Err(_) => 0,
    };
    let mut rng = SecureRng::new();
    let mut output = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes)?;
        bytes[0] = (timestamp >> 40) as u8;
        bytes[1] = (timestamp >> 32) as u8;
        bytes[2] = (timestamp >> 24) as u8;
        bytes[3] = (timestamp >> 16) as u8;
        bytes[4] = (timestamp >> 8) as u8;
        bytes[5] = timestamp as u8;
        // Version 7 and RFC 4122 variant bits.
        bytes[6] = (bytes[6] & 0x0f) | 0x70;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        output.push(format_uuid(Uuid::from_bytes(bytes), uppercase, hyphens));
    }
    Ok(output)
}

fn validate_count(count: usize) -> Result<()> {
    if count == 0 || count > MAX_COUNT {
        return Err(ToolError::new(format!(
            "UUID count must be between 1 and {MAX_COUNT}."
        )));
    }
    Ok(())
}

fn format_uuid(uuid: Uuid, uppercase: bool, hyphens: bool) -> String {
    let mut value = if hyphens {
        uuid.hyphenated().to_string()
    } else {
        uuid.simple().to_string()
    };
    if uppercase {
        value.make_ascii_uppercase();
    }
    value
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

    #[test]
    fn generates_valid_v7_uuids() -> Result<(), Box<dyn std::error::Error>> {
        let values = super::generate_v7(16, false, true)?;
        assert_eq!(values.len(), 16);
        for value in &values {
            let uuid = uuid::Uuid::parse_str(value)?;
            assert_eq!(uuid.get_version_num(), 7);
            assert_eq!(value.len(), 36);
        }
        Ok(())
    }
}

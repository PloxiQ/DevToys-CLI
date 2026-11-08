use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512};

use crate::error::{Result, ToolError};
use crate::tools::encoding::hex_encode_bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Sha256,
    Sha512,
}

impl Algorithm {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().replace('-', "").as_str() {
            "sha256" | "hs256" => Ok(Self::Sha256),
            "sha512" | "hs512" => Ok(Self::Sha512),
            other => Err(ToolError::new(format!("Unknown HMAC algorithm `{other}`."))
                .with_hint("Use sha256 or sha512.")),
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Sha256 => "SHA-256",
            Self::Sha512 => "SHA-512",
        }
    }
}

/// Computes a lowercase hex HMAC over the given bytes.
pub fn hash(algorithm: Algorithm, key: &[u8], input: &[u8]) -> Result<String> {
    let bytes = match algorithm {
        Algorithm::Sha256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key)
                .map_err(|_| ToolError::new("The HMAC key is not valid for SHA-256."))?;
            mac.update(input);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key)
                .map_err(|_| ToolError::new("The HMAC key is not valid for SHA-512."))?;
            mac.update(input);
            mac.finalize().into_bytes().to_vec()
        }
    };
    Ok(hex_encode_bytes(&bytes, false))
}

#[cfg(test)]
mod tests {
    use super::{hash, Algorithm};

    #[test]
    fn matches_rfc_4231_vectors() -> Result<(), Box<dyn std::error::Error>> {
        let message = b"The quick brown fox jumps over the lazy dog";
        assert_eq!(
            hash(Algorithm::Sha256, b"key", message)?,
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
        assert_eq!(
            hash(Algorithm::Sha512, b"key", message)?,
            "b42af09057bac1e2d41708e48a902e09b5ff7f12ab428a4fe86653c73dd248fb\
             82f948a549f7b791a5b41915ee4d1ec3935357e4e2317250d0372afa2ebeeb3a"
        );
        Ok(())
    }

    #[test]
    fn parses_algorithm_names() {
        assert_eq!(
            Algorithm::parse("SHA-256").map(Algorithm::name),
            Ok("SHA-256")
        );
        assert!(Algorithm::parse("md5").is_err());
    }
}

use md5::Md5;
use sha2::{Digest, Sha256, Sha512};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Md5,
    Sha256,
    Sha512,
}

impl Algorithm {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha256 => "SHA-256",
            Self::Sha512 => "SHA-512",
        }
    }
}

/// Returns a lowercase hex digest for the given UTF-8 text.
#[must_use]
pub fn hash(algorithm: Algorithm, input: &str) -> String {
    hash_bytes(algorithm, input.as_bytes())
}

/// Returns a lowercase hex digest for arbitrary bytes.
#[must_use]
pub fn hash_bytes(algorithm: Algorithm, input: &[u8]) -> String {
    match algorithm {
        Algorithm::Md5 => digest::<Md5>(input),
        Algorithm::Sha256 => digest::<Sha256>(input),
        Algorithm::Sha512 => digest::<Sha512>(input),
    }
}

fn digest<D: Digest>(input: &[u8]) -> String {
    let bytes = D::digest(input);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes.as_slice() {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{hash, Algorithm};

    #[test]
    fn md5_known_vector() {
        assert_eq!(hash(Algorithm::Md5, ""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(
            hash(
                Algorithm::Md5,
                "The quick brown fox jumps over the lazy dog"
            ),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    }

    #[test]
    fn sha_known_vectors() {
        assert_eq!(
            hash(Algorithm::Sha256, "abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hash(Algorithm::Sha512, "abc"),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn hashes_arbitrary_bytes() {
        assert_eq!(
            super::hash_bytes(Algorithm::Sha256, b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}

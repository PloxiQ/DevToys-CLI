use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::{Result, ToolError};

/// Thin wrapper around the OS entropy source with fallible APIs.
pub(crate) struct SecureRng {
    inner: OsRng,
}

impl SecureRng {
    pub fn new() -> Self {
        Self { inner: OsRng }
    }

    pub fn fill(&mut self, bytes: &mut [u8]) -> Result<()> {
        self.inner.try_fill_bytes(bytes).map_err(|error| {
            ToolError::new(format!(
                "The operating system random source failed: {error}"
            ))
            .with_hint("Retry the command. If the problem persists, check your OS entropy source.")
        })
    }

    /// Returns a uniformly distributed index in `0..len`.
    pub fn index(&mut self, len: usize) -> Result<usize> {
        if len == 0 {
            return Err(ToolError::new(
                "Cannot pick a random value from an empty set.",
            ));
        }
        let len = len as u64;
        // Rejection sampling avoids modulo bias.
        let zone = u64::MAX - (u64::MAX % len);
        loop {
            let mut bytes = [0u8; 8];
            self.fill(&mut bytes)?;
            let value = u64::from_le_bytes(bytes);
            if value < zone {
                return Ok((value % len) as usize);
            }
        }
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) -> Result<()> {
        for i in (1..items.len()).rev() {
            let j = self.index(i + 1)?;
            items.swap(i, j);
        }
        Ok(())
    }
}

impl Default for SecureRng {
    fn default() -> Self {
        Self::new()
    }
}

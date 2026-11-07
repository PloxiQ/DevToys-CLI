//! DevToys CLI - a fast, offline developer toolbox for the terminal.
//!
//! The library exposes every utility as a side-effect-free function so the
//! CLI, the TUI and third-party integrations all share the same behaviour.

#![forbid(unsafe_code)]

pub mod cli;
pub mod error;
pub mod style;
pub mod tools;

#[cfg(feature = "tui")]
pub mod tui;

pub use error::{Result, ToolError};

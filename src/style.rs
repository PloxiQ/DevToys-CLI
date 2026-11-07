//! Tiny ANSI styling helpers - no color crate required.

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const BRIGHT_BLACK: &str = "\x1b[90m";

/// Wraps `text` in an ANSI escape sequence when styling is enabled.
#[must_use]
pub fn paint(enabled: bool, code: &str, text: &str) -> String {
    if enabled {
        format!("{code}{text}{RESET}")
    } else {
        text.to_owned()
    }
}

pub fn bold(enabled: bool, text: &str) -> String {
    paint(enabled, BOLD, text)
}

pub fn dim(enabled: bool, text: &str) -> String {
    paint(enabled, DIM, text)
}

pub fn red(enabled: bool, text: &str) -> String {
    paint(enabled, RED, text)
}

pub fn green(enabled: bool, text: &str) -> String {
    paint(enabled, GREEN, text)
}

pub fn yellow(enabled: bool, text: &str) -> String {
    paint(enabled, YELLOW, text)
}

pub fn cyan(enabled: bool, text: &str) -> String {
    paint(enabled, CYAN, text)
}

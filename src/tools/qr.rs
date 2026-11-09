//! QR generation without image codecs or network access.
use super::limits;
use crate::error::{Result, ToolError};
use qrcode::render::svg;
use qrcode::{Color, EcLevel, QrCode};

// Version 40-L fits at most 7089 numeric characters; the encoder applies
// the tighter capacity for the selected mode and error correction level.
pub const MAX_INPUT: usize = 7089;

pub fn ecc(value: &str) -> Result<EcLevel> {
    match value.trim().to_ascii_uppercase().as_str() {
        "L" => Ok(EcLevel::L),
        "M" => Ok(EcLevel::M),
        "Q" => Ok(EcLevel::Q),
        "H" => Ok(EcLevel::H),
        _ => {
            Err(ToolError::new("Invalid QR error correction level.")
                .with_hint("Choose L, M, Q or H."))
        }
    }
}

pub fn generate(input: &str, level: &str, invert: bool, as_svg: bool) -> Result<String> {
    limits::check(input, MAX_INPUT, "QR input")?;
    if input.is_empty() {
        return Err(ToolError::new("QR input is empty.").with_hint("Provide text to encode."));
    }
    let code =
        QrCode::with_error_correction_level(input.as_bytes(), ecc(level)?).map_err(|error| {
            ToolError::new(format!("Could not encode QR data: {error}."))
                .with_hint("Shorten the input or choose a lower ECC level, such as L.")
        })?;
    if as_svg {
        let (dark, light) = if invert {
            ("#fff", "#000")
        } else {
            ("#000", "#fff")
        };
        return Ok(code
            .render::<svg::Color>()
            .quiet_zone(true)
            .dark_color(svg::Color(dark))
            .light_color(svg::Color(light))
            .build());
    }
    // Four modules of quiet zone on every side, two vertical pixels per cell.
    let width = code.width() + 8;
    let dark = |x: usize, y: usize| {
        let value = x >= 4
            && y >= 4
            && x < width - 4
            && y < width - 4
            && code[(x - 4, y - 4)] == Color::Dark;
        value ^ invert
    };
    let mut output = String::with_capacity(width * width.div_ceil(2) * 3);
    for y in (0..width).step_by(2) {
        for x in 0..width {
            output.push(match (dark(x, y), dark(x, y + 1)) {
                (false, false) => ' ',
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
            });
        }
        output.push('\n');
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_size_and_quiet_zone() -> Result<()> {
        let text = generate("HELLO WORLD", "M", false, false)?;
        assert_eq!(text.lines().count(), 15);
        assert!(text.lines().all(|line| line.chars().count() == 29));
        assert!(text.lines().take(2).all(|line| line.trim().is_empty()));
        assert!(text
            .lines()
            .all(|line| line.starts_with("    ") && line.ends_with("    ")));
        Ok(())
    }

    #[test]
    fn svg_ecc_unicode_and_inversion() -> Result<()> {
        for level in ["L", "M", "Q", "H"] {
            let svg = generate("Привіт 🌍", level, false, true)?;
            assert!(svg.starts_with("<?xml"));
            assert!(svg.contains("<svg"));
        }
        assert_ne!(
            generate("hello", "L", false, true)?,
            generate("hello", "H", false, true)?
        );
        let inverted = generate("hello", "M", true, false)?;
        assert!(inverted
            .lines()
            .next()
            .is_some_and(|line| line.chars().all(|c| c == '█')));
        assert!(generate("hello", "M", true, true)?.contains("fill=\"#000\""));
        Ok(())
    }

    #[test]
    fn rejects_empty_oversized_and_bad_ecc() {
        assert!(generate("", "M", false, false).is_err());
        assert!(generate(&"a".repeat(MAX_INPUT + 1), "L", false, false).is_err());
        assert!(generate(&"a".repeat(3000), "H", false, false).is_err());
        assert!(generate("hello", "invalid", false, false).is_err());
    }
}

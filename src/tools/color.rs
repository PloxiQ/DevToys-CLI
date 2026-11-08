use crate::error::{Result, ToolError};

#[derive(Debug, Clone, Copy)]
struct Rgba {
    red: u8,
    green: u8,
    blue: u8,
    alpha: f64,
}

#[derive(Debug, Clone, Copy)]
struct Hsl {
    hue: f64,
    saturation: f64,
    lightness: f64,
    alpha: f64,
}

#[derive(Debug, Clone, Copy)]
struct Hsv {
    hue: f64,
    saturation: f64,
    value: f64,
    alpha: f64,
}

/// Parses a CSS-style color and prints every common representation.
pub fn convert(input: &str) -> Result<String> {
    let color = parse(input)?;
    let hsl = rgb_to_hsl(color);
    let hsv = rgb_to_hsv(color);
    let luminance = relative_luminance(color);
    let black_contrast = contrast(luminance, 0.0);
    let white_contrast = contrast(luminance, 1.0);

    let hex = if color.alpha >= 1.0 {
        format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
    } else {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.red,
            color.green,
            color.blue,
            (color.alpha * 255.0).round() as u8
        )
    };

    let mut lines = vec![
        format!("Hex        : {hex}"),
        format!(
            "RGB        : rgb({}, {}, {})",
            color.red, color.green, color.blue
        ),
    ];

    if color.alpha < 1.0 {
        lines.push(format!(
            "RGBA       : rgba({}, {}, {}, {:.2})",
            color.red, color.green, color.blue, color.alpha
        ));
    }

    lines.push(format!(
        "HSL        : hsl({}, {:.0}%, {:.0}%)",
        hsl.hue.round(),
        hsl.saturation * 100.0,
        hsl.lightness * 100.0
    ));
    lines.push(format!(
        "HSV        : hsv({}, {:.0}%, {:.0}%)",
        hsv.hue.round(),
        hsv.saturation * 100.0,
        hsv.value * 100.0
    ));
    lines.push(format!("Luminance  : {luminance:.3}"));
    lines.push(format!(
        "Contrast   : {black_contrast:.2}:1 vs black, {white_contrast:.2}:1 vs white"
    ));

    let (best, ratio) = if black_contrast >= white_contrast {
        ("black", black_contrast)
    } else {
        ("white", white_contrast)
    };
    lines.push(format!("Best text  : {best} ({ratio:.2}:1)"));
    Ok(lines.join("\n"))
}

fn parse(input: &str) -> Result<Rgba> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ToolError::new("No color provided.")
            .with_hint("Try `#58a6ff`, `rgb(88, 166, 255)` or `hsl(212, 100%, 67%)`."));
    }

    if let Some(color) = named_color(&trimmed.to_ascii_lowercase()) {
        return Ok(color);
    }

    let lower = trimmed.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix('#') {
        return parse_hex(hex);
    }
    if let Some(value) = lower
        .strip_prefix("rgba(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_rgb(value, true);
    }
    if let Some(value) = lower
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_rgb(value, false);
    }
    if let Some(value) = lower
        .strip_prefix("hsla(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_hsl(value, true);
    }
    if let Some(value) = lower
        .strip_prefix("hsl(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_hsl(value, false);
    }
    if let Some(value) = lower
        .strip_prefix("hsva(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_hsv(value, true);
    }
    if let Some(value) = lower
        .strip_prefix("hsv(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_hsv(value, false);
    }

    if lower.len() == 6 && lower.chars().all(|character| character.is_ascii_hexdigit()) {
        return parse_hex(&lower);
    }

    let parts: Vec<&str> = lower.split(',').map(str::trim).collect();
    if parts.len() == 3 {
        return parse_rgb(&lower, false);
    }

    Err(
        ToolError::new(format!("Could not parse color `{input}`.")).with_hint(
            "Use a hex color, `rgb(...)`, `hsl(...)`, `hsv(...)` or a common CSS color name.",
        ),
    )
}

fn parse_hex(hex: &str) -> Result<Rgba> {
    if !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(ToolError::new(format!("`{hex}` is not a valid hex color."))
            .with_hint("Hex colors use digits 0-9 and letters a-f."));
    }
    let digits: Vec<u8> = hex
        .chars()
        .map(|character| character.to_digit(16).map_or(0, |digit| digit as u8))
        .collect();

    let (red, green, blue, alpha) = match digits.as_slice() {
        [r, g, b] => (r * 17, g * 17, b * 17, 1.0),
        [r, g, b, a] => (r * 17, g * 17, b * 17, f64::from(a * 17) / 255.0),
        [r1, r2, g1, g2, b1, b2] => (r1 * 16 + r2, g1 * 16 + g2, b1 * 16 + b2, 1.0),
        [r1, r2, g1, g2, b1, b2, a1, a2] => (
            r1 * 16 + r2,
            g1 * 16 + g2,
            b1 * 16 + b2,
            f64::from(a1 * 16 + a2) / 255.0,
        ),
        _ => {
            return Err(ToolError::new(format!(
                "Hex colors must have 3, 4, 6 or 8 digits, got {}.",
                digits.len()
            ))
            .with_hint("Examples: `#0af`, `#58a6ff`, `#58a6ffcc`."));
        }
    };

    Ok(Rgba {
        red,
        green,
        blue,
        alpha,
    })
}

fn parse_rgb(value: &str, has_alpha: bool) -> Result<Rgba> {
    let parts: Vec<&str> = value.split(',').map(str::trim).collect();
    if parts.len() != if has_alpha { 4 } else { 3 } {
        return Err(
            ToolError::new("RGB colors need red, green and blue values.")
                .with_hint("Example: `rgb(88, 166, 255)` or `rgba(88, 166, 255, 0.8)`."),
        );
    }

    let red = parse_channel(parts[0])?;
    let green = parse_channel(parts[1])?;
    let blue = parse_channel(parts[2])?;
    let alpha = if has_alpha {
        parse_alpha(parts[3])?
    } else {
        1.0
    };
    Ok(Rgba {
        red,
        green,
        blue,
        alpha,
    })
}

fn parse_channel(value: &str) -> Result<u8> {
    if let Some(percent) = value.strip_suffix('%') {
        let percent: f64 = percent
            .trim()
            .parse()
            .map_err(|_| ToolError::new(format!("`{value}` is not a valid color percentage.")))?;
        if !(0.0..=100.0).contains(&percent) {
            return Err(ToolError::new(
                "Color percentages must be between 0 and 100.",
            ));
        }
        return Ok((percent * 2.55).round() as u8);
    }

    let channel: i64 = value
        .parse()
        .map_err(|_| ToolError::new(format!("`{value}` is not a valid color channel.")))?;
    if !(0..=255).contains(&channel) {
        return Err(ToolError::new("RGB channels must be between 0 and 255."));
    }
    Ok(channel as u8)
}

fn parse_alpha(value: &str) -> Result<f64> {
    let alpha: f64 = value
        .parse()
        .map_err(|_| ToolError::new(format!("`{value}` is not a valid alpha value.")))?;
    if !(0.0..=1.0).contains(&alpha) {
        return Err(ToolError::new("Alpha must be between 0 and 1."));
    }
    Ok(alpha)
}

fn parse_hsl(value: &str, has_alpha: bool) -> Result<Rgba> {
    let parts: Vec<&str> = value.split(',').map(str::trim).collect();
    if parts.len() != if has_alpha { 4 } else { 3 } {
        return Err(
            ToolError::new("HSL colors need hue, saturation and lightness.")
                .with_hint("Example: `hsl(212, 100%, 67%)` or `hsla(212, 100%, 67%, 0.8)`."),
        );
    }
    let hue = parse_hue(parts[0])?;
    let saturation = parse_percentage(parts[1], "Saturation")?;
    let lightness = parse_percentage(parts[2], "Lightness")?;
    let alpha = if has_alpha {
        parse_alpha(parts[3])?
    } else {
        1.0
    };
    Ok(hsl_to_rgb(Hsl {
        hue,
        saturation,
        lightness,
        alpha,
    }))
}

fn parse_hsv(value: &str, has_alpha: bool) -> Result<Rgba> {
    let parts: Vec<&str> = value.split(',').map(str::trim).collect();
    if parts.len() != if has_alpha { 4 } else { 3 } {
        return Err(ToolError::new("HSV colors need hue, saturation and value.")
            .with_hint("Example: `hsv(212, 65%, 100%)` or `hsva(212, 65%, 100%, 0.8)`."));
    }
    let hue = parse_hue(parts[0])?;
    let saturation = parse_percentage(parts[1], "Saturation")?;
    let value = parse_percentage(parts[2], "Value")?;
    let alpha = if has_alpha {
        parse_alpha(parts[3])?
    } else {
        1.0
    };
    Ok(hsv_to_rgb(Hsv {
        hue,
        saturation,
        value,
        alpha,
    }))
}

fn parse_hue(value: &str) -> Result<f64> {
    let trimmed = value.trim().trim_end_matches("deg").trim();
    let hue: f64 = trimmed
        .parse()
        .map_err(|_| ToolError::new(format!("`{value}` is not a valid hue.")))?;
    if !hue.is_finite() {
        return Err(ToolError::new("Hue must be a finite number."));
    }
    Ok(hue.rem_euclid(360.0))
}

fn parse_percentage(value: &str, label: &str) -> Result<f64> {
    let trimmed = value.trim().trim_end_matches('%').trim();
    let percent: f64 = trimmed
        .parse()
        .map_err(|_| ToolError::new(format!("`{value}` is not a valid {label}.")))?;
    if !(0.0..=100.0).contains(&percent) {
        return Err(ToolError::new(format!(
            "{label} must be between 0 and 100."
        )));
    }
    Ok(percent / 100.0)
}

fn hsl_to_rgb(hsl: Hsl) -> Rgba {
    let chroma = (1.0 - (2.0 * hsl.lightness - 1.0).abs()) * hsl.saturation;
    let section = hsl.hue / 60.0;
    let x = chroma * (1.0 - (section % 2.0 - 1.0).abs());
    let (red, green, blue) = match section as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = hsl.lightness - chroma / 2.0;
    Rgba {
        red: channel((red + m) * 255.0),
        green: channel((green + m) * 255.0),
        blue: channel((blue + m) * 255.0),
        alpha: hsl.alpha,
    }
}

fn hsv_to_rgb(hsv: Hsv) -> Rgba {
    let chroma = hsv.value * hsv.saturation;
    let section = hsv.hue / 60.0;
    let x = chroma * (1.0 - (section % 2.0 - 1.0).abs());
    let (red, green, blue) = match section as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = hsv.value - chroma;
    Rgba {
        red: channel((red + m) * 255.0),
        green: channel((green + m) * 255.0),
        blue: channel((blue + m) * 255.0),
        alpha: hsv.alpha,
    }
}

fn channel(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn rgb_to_hsl(color: Rgba) -> Hsl {
    let red = f64::from(color.red) / 255.0;
    let green = f64::from(color.green) / 255.0;
    let blue = f64::from(color.blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let lightness = (max + min) / 2.0;

    if delta.abs() < f64::EPSILON {
        return Hsl {
            hue: 0.0,
            saturation: 0.0,
            lightness,
            alpha: color.alpha,
        };
    }

    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if (max - red).abs() < f64::EPSILON {
        60.0 * (((green - blue) / delta) % 6.0)
    } else if (max - green).abs() < f64::EPSILON {
        60.0 * ((blue - red) / delta + 2.0)
    } else {
        60.0 * ((red - green) / delta + 4.0)
    };

    Hsl {
        hue: hue.rem_euclid(360.0),
        saturation,
        lightness,
        alpha: color.alpha,
    }
}

fn rgb_to_hsv(color: Rgba) -> Hsv {
    let red = f64::from(color.red) / 255.0;
    let green = f64::from(color.green) / 255.0;
    let blue = f64::from(color.blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;

    let hue = if delta.abs() < f64::EPSILON {
        0.0
    } else if (max - red).abs() < f64::EPSILON {
        60.0 * (((green - blue) / delta) % 6.0)
    } else if (max - green).abs() < f64::EPSILON {
        60.0 * ((blue - red) / delta + 2.0)
    } else {
        60.0 * ((red - green) / delta + 4.0)
    };

    Hsv {
        hue: hue.rem_euclid(360.0),
        saturation: if max == 0.0 { 0.0 } else { delta / max },
        value: max,
        alpha: color.alpha,
    }
}

fn relative_luminance(color: Rgba) -> f64 {
    let channels = [color.red, color.green, color.blue].map(|value| {
        let value = f64::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
}

fn contrast(first: f64, second: f64) -> f64 {
    let lighter = first.max(second);
    let darker = first.min(second);
    (lighter + 0.05) / (darker + 0.05)
}

fn named_color(name: &str) -> Option<Rgba> {
    let (red, green, blue) = match name {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "lime" => (0, 255, 0),
        "blue" => (0, 0, 255),
        "yellow" => (255, 255, 0),
        "cyan" | "aqua" => (0, 255, 255),
        "magenta" | "fuchsia" => (255, 0, 255),
        "orange" => (255, 165, 0),
        "purple" => (128, 0, 128),
        "pink" => (255, 192, 203),
        "brown" => (165, 42, 42),
        "gray" | "grey" => (128, 128, 128),
        "silver" => (192, 192, 192),
        "maroon" => (128, 0, 0),
        "olive" => (128, 128, 0),
        "navy" => (0, 0, 128),
        "teal" => (0, 128, 128),
        "gold" => (255, 215, 0),
        "transparent" => {
            return Some(Rgba {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0.0,
            })
        }
        _ => return None,
    };
    Some(Rgba {
        red,
        green,
        blue,
        alpha: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::convert;

    #[test]
    fn converts_hex_to_every_format() -> Result<(), Box<dyn std::error::Error>> {
        let output = convert("#58a6ff")?;
        assert!(output.contains("Hex        : #58a6ff"));
        assert!(output.contains("RGB        : rgb(88, 166, 255)"));
        assert!(output.contains("HSL        : hsl(212, 100%, 67%)"));
        assert!(output.contains("Best text  : black"));
        Ok(())
    }

    #[test]
    fn parses_rgb_hsl_and_names() -> Result<(), Box<dyn std::error::Error>> {
        assert!(convert("rgb(255, 0, 0)")?.contains("Hex        : #ff0000"));
        assert!(convert("hsl(120, 100%, 25%)")?.contains("Hex        : #008000"));
        assert!(convert("rebeccapurple").is_err());
        assert!(convert("orange")?.contains("#ffa500"));
        Ok(())
    }

    #[test]
    fn supports_short_hex_and_alpha() -> Result<(), Box<dyn std::error::Error>> {
        assert!(convert("#0af")?.contains("Hex        : #00aaff"));
        let output = convert("#00000080")?;
        assert!(output.contains("RGBA       : rgba(0, 0, 0, 0.50)"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_colors() {
        assert!(convert("#12").is_err());
        assert!(convert("rgb(300, 0, 0)").is_err());
        assert!(convert("nope").is_err());
    }
}

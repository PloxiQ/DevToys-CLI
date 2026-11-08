use crate::error::{Result, ToolError};

/// Resolves a file extension to MIME types, or a MIME type to extensions.
pub fn lookup(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(
            ToolError::new("No MIME type or file extension provided.").with_hint(
                "Try `devtoys mime json`, `devtoys mime .png` or `devtoys mime application/json`.",
            ),
        );
    }

    let lowered = trimmed.to_ascii_lowercase();
    if lowered.contains('/') {
        let mime = lowered.split(';').next().unwrap_or(&lowered).trim();
        let Some(extensions) = mime_guess::get_mime_extensions_str(mime) else {
            return Err(
                ToolError::new(format!("No known extensions for MIME type `{mime}`.")).with_hint(
                    "Try a common type such as application/json, text/html or image/png.",
                ),
            );
        };
        let list = extensions
            .iter()
            .map(|extension| format!(".{extension}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Ok(format!("MIME type  : {mime}\nExtensions : {list}"));
    }

    let extension = trimmed
        .rsplit_once('.')
        .map_or(trimmed, |(_, extension)| extension)
        .trim_start_matches('.');
    if extension.is_empty() {
        return Err(ToolError::new("The file extension is empty."));
    }

    let guesses: Vec<&str> = mime_guess::from_ext(extension).iter_raw().collect();
    if guesses.is_empty() {
        return Err(
            ToolError::new(format!("No MIME type found for extension `.{extension}`.")).with_hint(
                "Try a common extension such as json, html, css, js, png, jpg, svg, pdf or zip.",
            ),
        );
    }

    Ok(format!(
        "Extension  : .{extension}\nMIME type  : {}\nAll matches: {}",
        guesses[0],
        guesses.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::lookup;

    #[test]
    fn maps_extensions_to_mime_types() -> Result<(), Box<dyn std::error::Error>> {
        assert!(lookup("json")?.contains("application/json"));
        assert!(lookup(".html")?.contains("text/html"));
        assert!(lookup("photo.png")?.contains("image/png"));
        Ok(())
    }

    #[test]
    fn maps_mime_types_back_to_extensions() -> Result<(), Box<dyn std::error::Error>> {
        let output = lookup("application/json")?;
        assert!(output.contains("Extensions"));
        assert!(output.contains(".json"));
        assert!(lookup("not/a-real-type").is_err());
        assert!(lookup("").is_err());
        Ok(())
    }
}

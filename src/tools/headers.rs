//! Offline inspection of HTTP header blocks; this is not an HTTP client.
use super::limits;
use crate::error::{Result, ToolError};

pub const MAX_INPUT: usize = limits::MIB;

pub fn parse(input: &str, redact: bool) -> Result<String> {
    limits::check(input, MAX_INPUT, "Header input")?;
    if input.trim().is_empty() {
        return Err(ToolError::new("No headers provided.")
            .with_hint("Paste lines such as `Content-Type: application/json`."));
    }
    let mut start = None;
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut ended = false;
    for (index, line) in input.lines().enumerate() {
        let number = index + 1;
        if line.is_empty() {
            ended = true;
            continue;
        }
        if ended {
            return Err(malformed(
                number,
                "unexpected content after the header block",
            ));
        }
        if line.chars().any(|c| c.is_control() && c != '\t') {
            return Err(malformed(number, "control character"));
        }
        if line.starts_with([' ', '\t']) {
            return Err(malformed(number, "folded header lines are not supported"));
        }
        if index == 0 && is_start_line(line) {
            start = Some(line.to_owned());
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| malformed(number, "expected Name: value"))?;
        if name.is_empty() || !name.bytes().all(token) {
            return Err(malformed(number, "invalid header name"));
        }
        let name = name
            .split('-')
            .map(|part| {
                let mut lower = part.to_ascii_lowercase();
                if let Some(first) = lower.get_mut(..1) {
                    first.make_ascii_uppercase();
                }
                lower
            })
            .collect::<Vec<_>>()
            .join("-");
        let sensitive = matches!(
            name.as_str(),
            "Authorization" | "Proxy-Authorization" | "Cookie" | "Set-Cookie" | "X-Api-Key"
        );
        let value = if redact && sensitive {
            "[redacted]"
        } else {
            value.trim_matches([' ', '\t'])
        };
        headers.push((name, value.to_owned()));
    }
    let mut lines = Vec::new();
    if let Some(start) = start {
        lines.push(start);
    }
    lines.extend(
        headers
            .into_iter()
            .map(|(name, value)| format!("{name}: {value}")),
    );
    Ok(lines.join("\n"))
}

fn token(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)
}

fn is_start_line(line: &str) -> bool {
    let parts: Vec<&str> = line.split(' ').collect();
    if parts.len() < 2 {
        return false;
    }
    let version = |s: &str| matches!(s, "HTTP/1.0" | "HTTP/1.1" | "HTTP/2" | "HTTP/3");
    if version(parts[0]) {
        return parts[1].len() == 3
            && parts[1].bytes().all(|b| b.is_ascii_digit())
            && parts[1].starts_with(['1', '2', '3', '4', '5']);
    }
    parts.len() == 3
        && !parts[0].is_empty()
        && parts[0].bytes().all(token)
        && !parts[1].is_empty()
        && version(parts[2])
}

fn malformed(line: usize, reason: &str) -> ToolError {
    ToolError::new(format!("Malformed header at line {line}: {reason}."))
        .with_hint("Use one `Name: value` per line; unfold continuation lines and omit the body.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_duplicates_and_crlf() -> Result<()> {
        assert_eq!(
            parse(
                "HTTP/1.1 200 OK\r\nset-cookie: a=1\r\nSet-Cookie:\tb=2 \t\r\n\r\n",
                false
            )?,
            "HTTP/1.1 200 OK\nSet-Cookie: a=1\nSet-Cookie: b=2"
        );
        assert_eq!(
            parse("GET / HTTP/1.1\nhOST: example.com", false)?,
            "GET / HTTP/1.1\nHost: example.com"
        );
        Ok(())
    }

    #[test]
    fn redacts_all_sensitive_headers() -> Result<()> {
        for name in [
            "authorization",
            "PROXY-AUTHORIZATION",
            "Cookie",
            "set-cookie",
            "x-api-key",
        ] {
            let output = parse(&format!("{name}: secret\n{name}: another"), true)?;
            assert_eq!(output.matches("[redacted]").count(), 2);
            assert!(!output.contains("secret"));
        }
        assert_eq!(parse("Host: example.com", true)?, "Host: example.com");
        Ok(())
    }

    #[test]
    fn rejects_malformed_folded_empty_and_huge() {
        for input in [
            "",
            " \t\r\n",
            "no colon",
            "Bad Name: value",
            ": value",
            "x: a\n folded",
            "x: a\n\nbody",
            "HTTP/1.1 nope",
            "x: a\rb",
        ] {
            let error = parse(input, false).err();
            assert!(error.is_some(), "{input:?}");
            assert!(error.and_then(|e| e.hint().map(str::to_owned)).is_some());
        }
        assert!(parse("x: ok\nbad", false)
            .err()
            .is_some_and(|e| e.message().contains("line 2")));
        assert!(parse(&"x".repeat(MAX_INPUT + 1), false).is_err());
    }
}

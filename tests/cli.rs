use std::error::Error;
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn devtoys() -> Command {
    Command::new(env!("CARGO_BIN_EXE_devtoys"))
}

fn run(args: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(devtoys().args(args).output()?)
}

fn text(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(bytes.to_vec())?)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn json_format_and_minify() -> Result<(), Box<dyn Error>> {
    let output = run(&["json", "format", r#"{"z":1,"a":[true,null]}"#])?;
    assert_success(&output);
    let stdout = text(&output.stdout)?;
    assert!(stdout.contains("\"z\": 1"));
    assert!(stdout.contains("\"a\": ["));
    assert!(
        !stdout.contains('\u{1b}'),
        "piped output must not contain ANSI"
    );

    let output = run(&["json", "minify", r#"{ "hello" : "world" }"#])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), r#"{"hello":"world"}"#);
    Ok(())
}

#[test]
fn reads_json_from_stdin() -> Result<(), Box<dyn Error>> {
    let mut child = devtoys()
        .args(["json", "minify"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(br#"{ "from": "stdin", "ok": true }"#)?;
    }
    let output = child.wait_with_output()?;
    assert_success(&output);
    assert_eq!(
        text(&output.stdout)?.trim(),
        r#"{"from":"stdin","ok":true}"#
    );
    Ok(())
}

#[test]
fn base64_and_hex_round_trip() -> Result<(), Box<dyn Error>> {
    let output = run(&["base64", "encode", "Hello, DevToys!"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "SGVsbG8sIERldlRveXMh");

    let output = run(&["base64", "decode", "SGVsbG8sIERldlRveXMh"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "Hello, DevToys!");

    let output = run(&["base64", "encode", "--hex", "48 65 6c 6c 6f"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "SGVsbG8=");

    let output = run(&["base64", "decode", "--hex", "SGVsbG8="])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "48656c6c6f");
    Ok(())
}

#[test]
fn hashes_and_jwt_decode() -> Result<(), Box<dyn Error>> {
    let output = run(&["hash", "sha256", "hello world"])?;
    assert_success(&output);
    assert_eq!(
        text(&output.stdout)?.trim(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );

    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let output = run(&["jwt", "decode", token, "--part", "payload"])?;
    assert_success(&output);
    let payload = text(&output.stdout)?;
    assert!(payload.contains("\"name\": \"John Doe\""));
    assert!(payload.contains("\"iat\": 1516239022"));
    Ok(())
}

#[test]
fn timestamp_cron_regex_and_uuid() -> Result<(), Box<dyn Error>> {
    let output = run(&["timestamp", "to-epoch", "2024-01-01T00:00:00Z"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "1704067200");

    let output = run(&["timestamp", "to-date", "1700000000"])?;
    assert_success(&output);
    assert!(text(&output.stdout)?.contains("2023-11-14T22:13:20Z"));

    let output = run(&["cron", "*/15 * * * *"])?;
    assert_success(&output);
    let cron = text(&output.stdout)?;
    assert!(cron.contains("Every 15 minutes."));
    assert!(cron.contains("Next runs:"));

    let output = run(&[
        "regex",
        "test",
        r"\b\w+@\w+\.\w+\b",
        "Contact dev@example.com or support@devtoys.dev",
    ])?;
    assert_success(&output);
    let regex = text(&output.stdout)?;
    assert!(regex.contains("Matches: 2"));
    assert!(regex.contains("dev@example.com"));

    let output = run(&["uuid", "--count", "3"])?;
    assert_success(&output);
    let uuid_output = text(&output.stdout)?;
    let uuids: Vec<&str> = uuid_output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(uuids.len(), 3);
    assert!(uuids
        .iter()
        .all(|uuid| uuid.len() == 36 && uuid.split('-').count() == 5));
    Ok(())
}

#[test]
fn generators_respect_count_and_format_options() -> Result<(), Box<dyn Error>> {
    let output = run(&["random", "16", "--charset", "hex", "--count", "3"])?;
    assert_success(&output);
    let random = text(&output.stdout)?;
    let values: Vec<&str> = random.lines().map(str::trim).collect();
    assert_eq!(values.len(), 3);
    assert!(values
        .iter()
        .all(|value| value.len() == 16 && value.chars().all(|c| c.is_ascii_hexdigit())));

    let output = run(&["random", "8", "--charset", "symbols"])?;
    assert_success(&output);
    let symbols = text(&output.stdout)?;
    let symbols = symbols.trim();
    assert_eq!(symbols.len(), 8);
    assert!(symbols.chars().all(|c| c.is_ascii_punctuation()));

    let output = run(&["password", "--length", "16", "--no-symbols"])?;
    assert_success(&output);
    let password = text(&output.stdout)?;
    let password = password.trim();
    assert_eq!(password.len(), 16);
    assert!(password.chars().any(|c| c.is_ascii_digit()));
    assert!(password.chars().any(|c| c.is_ascii_lowercase()));
    assert!(password.chars().any(|c| c.is_ascii_uppercase()));

    let output = run(&["uuid", "--no-hyphens", "--upper"])?;
    assert_success(&output);
    let uuid = text(&output.stdout)?;
    let uuid = uuid.trim();
    assert_eq!(uuid.len(), 32);
    assert!(uuid
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase()));
    Ok(())
}

#[test]
fn every_subcommand_help_succeeds() -> Result<(), Box<dyn Error>> {
    for command in [
        "json",
        "base64",
        "url",
        "timestamp",
        "cron",
        "case",
        "number",
        "bytes",
        "color",
        "text",
        "diff",
        "hex",
        "binary",
        "html",
        "csv",
        "http",
        "mime",
        "ip",
        "hmac",
        "uuid",
        "hash",
        "lorem",
        "random",
        "password",
        "jwt",
        "regex",
        "tui",
    ] {
        let output = run(&[command, "--help"])?;
        assert_success(&output);
    }
    Ok(())
}

#[test]
fn converts_case_numbers_bytes_and_colors() -> Result<(), Box<dyn Error>> {
    let output = run(&["case", "Hello world", "--style", "kebab"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "hello-world");

    let output = run(&["number", "255", "--to", "16"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "ff");

    let output = run(&["number", "0b1010", "--to", "10"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "10");

    let output = run(&["bytes", "1536"])?;
    assert_success(&output);
    let bytes = text(&output.stdout)?;
    assert!(bytes.contains("1,536"));
    assert!(bytes.contains("1.50 KiB"));

    let output = run(&["color", "#58a6ff"])?;
    assert_success(&output);
    let color = text(&output.stdout)?;
    assert!(color.contains("rgb(88, 166, 255)"));
    assert!(color.contains("hsl(212, 100%, 67%)"));
    Ok(())
}

#[test]
fn sorts_stats_slugs_and_diffs_text() -> Result<(), Box<dyn Error>> {
    let output = run(&["text", "sort", "--reverse", "b\na\nc"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "c\nb\na");

    let output = run(&["text", "dedupe", "--ignore-case", "a\nA\nb"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "a\nb");

    let output = run(&["text", "stats", "one two three"])?;
    assert_success(&output);
    assert!(text(&output.stdout)?.contains("Words      : 3"));

    let output = run(&["text", "slug", "Crème brûlée"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "creme-brulee");

    let output = run(&["diff", "one\ntwo\nthree", "one\nthree\nfour"])?;
    assert_success(&output);
    let diff = text(&output.stdout)?;
    assert!(diff.contains("-two"));
    assert!(diff.contains("+four"));

    let output = run(&["diff", "same", "same"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "No differences.");
    Ok(())
}

#[test]
fn encodes_and_decodes_hex_binary_html_and_url_safe_base64() -> Result<(), Box<dyn Error>> {
    let output = run(&["hex", "encode", "Hello"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "48656c6c6f");

    let output = run(&["hex", "decode", "48 65 6c 6c 6f"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "Hello");

    let output = run(&["binary", "encode", "Hi"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "0100100001101001");

    let output = run(&["binary", "decode", "01001000 01101001"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "Hi");

    let output = run(&["html", "encode", "Tom & Jerry"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "Tom &amp; Jerry");

    let output = run(&["html", "decode", "&lt;b&gt;Hi&lt;/b&gt;"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "<b>Hi</b>");

    let output = run(&["base64", "encode", "--url-safe", "Hello"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "SGVsbG8");
    Ok(())
}

#[test]
fn queries_json_and_converts_csv() -> Result<(), Box<dyn Error>> {
    let input = r#"{"user":{"name":"Ada"},"tools":["json","jwt"]}"#;
    let output = run(&["json", "get", "user.name", input])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "\"Ada\"");

    let output = run(&["json", "get", "tools[1]", "--raw", input])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "jwt");

    let output = run(&["csv", "to-json", "name,age\nAda,30"])?;
    assert_success(&output);
    let json = text(&output.stdout)?;
    assert!(json.contains("\"name\": \"Ada\""));
    assert!(json.contains("\"age\": \"30\""));

    let output = run(&["csv", "from-json", r#"[{"name":"Ada","age":30}]"#])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "age,name\n30,Ada");
    Ok(())
}

#[test]
fn inspects_urls_http_mime_and_ips() -> Result<(), Box<dyn Error>> {
    let output = run(&["url", "parse", "https://example.com/path?a=1&b=two#top"])?;
    assert_success(&output);
    let url = text(&output.stdout)?;
    assert!(url.contains("Scheme     : https"));
    assert!(url.contains("a = 1"));
    assert!(url.contains("Fragment   : top"));

    let output = run(&["http", "404"])?;
    assert_success(&output);
    assert!(text(&output.stdout)?.contains("Not Found"));

    let output = run(&["http", "--class", "4"])?;
    assert_success(&output);
    let class = text(&output.stdout)?;
    assert!(class.contains("400"));
    assert!(!class.contains("500"));

    let output = run(&["mime", "application/json"])?;
    assert_success(&output);
    assert!(text(&output.stdout)?.contains(".json"));

    let output = run(&["mime", "photo.png"])?;
    assert_success(&output);
    assert!(text(&output.stdout)?.contains("image/png"));

    let output = run(&["ip", "192.168.1.42/24"])?;
    assert_success(&output);
    let ip = text(&output.stdout)?;
    assert!(ip.contains("Network    : 192.168.1.0"));
    assert!(ip.contains("Usable     : 254"));
    Ok(())
}

#[test]
fn hashes_hmacs_files_and_generates_v7_uuids() -> Result<(), Box<dyn Error>> {
    let output = run(&["uuid", "--v7"])?;
    assert_success(&output);
    let uuid = text(&output.stdout)?;
    let uuid = uuid.trim();
    assert_eq!(uuid.len(), 36);
    assert_eq!(&uuid[14..15], "7");

    let output = run(&[
        "hmac",
        "sha256",
        "key",
        "The quick brown fox jumps over the lazy dog",
    ])?;
    assert_success(&output);
    assert_eq!(
        text(&output.stdout)?.trim(),
        "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
    );

    let path = std::env::temp_dir().join("devtoys-cli-hash-file-test.txt");
    std::fs::write(&path, "abc")?;
    let path_text = path.to_string_lossy().into_owned();
    let output = run(&["hash", "sha256", "--file", &path_text])?;
    assert_success(&output);
    assert_eq!(
        text(&output.stdout)?.trim(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[test]
fn invalid_input_fails_with_a_hint() -> Result<(), Box<dyn Error>> {
    let output = run(&["json", "format", "{oops}"])?;
    assert!(!output.status.success());
    let stderr = text(&output.stderr)?;
    assert!(stderr.contains("error:"));
    assert!(stderr.contains("hint:"));
    Ok(())
}

#[test]
fn help_is_available() -> Result<(), Box<dyn Error>> {
    let output = run(&["--help"])?;
    assert_success(&output);
    let help = text(&output.stdout)?;
    assert!(help.contains("Format, minify and inspect JSON"));
    assert!(help.contains("jwt"));
    Ok(())
}

use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn devtoys() -> Command {
    Command::new(env!("CARGO_BIN_EXE_devtoys"))
}

fn run(args: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(devtoys().args(args).output()?)
}

fn run_stdin(args: &[&str], input: &[u8]) -> Result<Output, Box<dyn Error>> {
    let mut child = devtoys()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(input) {
            if error.kind() != std::io::ErrorKind::BrokenPipe {
                return Err(error.into());
            }
        }
    }
    Ok(child.wait_with_output()?)
}

fn assert_input_error(output: &Output) {
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "errors must not contaminate stdout"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:"), "{stderr}");
    assert!(stderr.contains("hint:"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

struct ToolFiles(PathBuf);

impl ToolFiles {
    fn new() -> Result<Self, Box<dyn Error>> {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        loop {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("devtoys-cli-tools-{}-{id}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().into_owned()
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> Result<String, Box<dyn Error>> {
        let path = self.path(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    }
}

impl Drop for ToolFiles {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
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
        "base32",
        "headers",
        "diff3",
        "qr",
        "jsonschema",
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
    assert_eq!(text(&output.stdout)?.trim(), "name,age\nAda,30");
    Ok(())
}

#[test]
fn preserves_json_precision_csv_delimiters_and_ip_scopes() -> Result<(), Box<dyn Error>> {
    let input = r#"{"b":1,"a":{"z":2,"y":3},"id":123456789012345678901234567890}"#;
    let output = run(&["json", "get", "$", input])?;
    assert_success(&output);
    let json = text(&output.stdout)?;
    assert!(json.contains("123456789012345678901234567890"));
    let b = json.find("\"b\"").unwrap_or(usize::MAX);
    let a = json.find("\"a\"").unwrap_or(usize::MAX);
    let id = json.find("\"id\"").unwrap_or(usize::MAX);
    assert!(b < a && a < id, "key order changed: {json}");

    let output = run(&["csv", "from-json", r#"[{"one":1,"two":2}]"#, "-d", "X"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?.trim(), "oneXtwo\n1X2");

    let output = run(&["ip", "10.0.0.1 / 24"])?;
    assert_success(&output);
    assert!(text(&output.stdout)?.contains("Network    : 10.0.0.0"));

    for (ip, scope) in [
        ("100.64.0.1", "shared (CGNAT)"),
        ("198.18.0.1", "benchmarking"),
        ("240.0.0.1", "reserved"),
        ("192.0.0.1", "protocol assignment"),
        ("fec0::1", "site-local (deprecated)"),
        ("2002::1", "6to4"),
        ("64:ff9b::1", "NAT64"),
    ] {
        let output = run(&["ip", ip])?;
        assert_success(&output);
        let details = text(&output.stdout)?;
        assert!(
            details.contains(&format!("Scope      : {scope}")),
            "unexpected scope for {ip}: {details}"
        );
    }
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

#[test]
fn base32_arguments_stdin_flags_and_invalid_input() -> Result<(), Box<dyn Error>> {
    let output = run(&["base32", "encode", "foobar"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?, "MZXW6YTBOI======\n");
    let output = run(&["base32", "encode", "foobar", "--no-padding", "--lower"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?, "mzxw6ytboi\n");
    let output = run(&["base32", "decode", "mZ Xw6YtBoI======\r\n"])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?, "foobar\n");
    for explicit in [false, true] {
        let mut args = vec!["base32", "encode"];
        if explicit {
            args.push("-");
        }
        let output = run_stdin(&args, "Привіт 🌍".as_bytes())?;
        assert_success(&output);
        let encoded = text(&output.stdout)?;
        let mut args = vec!["base32", "decode"];
        if explicit {
            args.push("-");
        }
        let output = run_stdin(&args, encoded.as_bytes())?;
        assert_success(&output);
        assert_eq!(text(&output.stdout)?, "Привіт 🌍\n");
    }
    for malformed in ["A", "M0======", "MY=", "MY=======", "MZ======", "74======"] {
        assert_input_error(&run(&["base32", "decode", malformed])?);
        assert_input_error(&run_stdin(
            &["base32", "decode", "-"],
            malformed.as_bytes(),
        )?);
    }
    for action in ["encode", "decode"] {
        let output = run(&["base32", action, ""])?;
        assert_success(&output);
        assert_eq!(text(&output.stdout)?, "\n");
        assert_success(&run(&["base32", action, "--help"])?);
    }
    Ok(())
}

#[test]
fn headers_arguments_stdin_duplicates_and_redaction() -> Result<(), Box<dyn Error>> {
    let input = "HTTP/1.1 200 OK\r\ncontent-type:\ttext/plain \t\r\nset-cookie: a=1\r\nSet-Cookie: b=2\r\n\r\n";
    let expected = "HTTP/1.1 200 OK\nContent-Type: text/plain\nSet-Cookie: a=1\nSet-Cookie: b=2\n";
    let output = run(&["headers", input])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?, expected);
    for args in [&["headers"][..], &["headers", "-"][..]] {
        let output = run_stdin(args, input.as_bytes())?;
        assert_success(&output);
        assert_eq!(text(&output.stdout)?, expected);
    }
    let input = "GET / HTTP/1.1\nAuthorization: Bearer secret\nProxy-Authorization: proxy\nCookie: session=secret\nSet-Cookie: session=secret\nX-Api-Key: secret\nHost: example.com";
    for output in [
        run(&["headers", input, "--redact"])?,
        run_stdin(&["headers", "--redact", "-"], input.as_bytes())?,
    ] {
        assert_success(&output);
        let headers = text(&output.stdout)?;
        assert_eq!(headers.matches("[redacted]").count(), 5);
        assert!(!headers.contains("secret"));
        assert!(headers.contains("GET / HTTP/1.1"));
        assert!(headers.contains("Host: example.com"));
    }
    for input in [
        "",
        " \t\r\n",
        "missing colon",
        "Host: example.com\n folded",
        "bad name: value",
    ] {
        assert_input_error(&run(&["headers", input])?);
    }
    let output = run_stdin(&["headers", "-"], b"Host: example.com\nmalformed")?;
    assert_input_error(&output);
    assert!(text(&output.stderr)?.contains("line 2"));
    Ok(())
}

#[test]
fn diff3_merges_files_and_each_stdin_position() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    let base_text = "one\ntwo\nthree\n";
    let ours_text = "ONE\ntwo\nthree\n";
    let theirs_text = "one\ntwo\nTHREE\n";
    let base = files.write("base file.txt", base_text)?;
    let ours = files.write("ours файл.txt", ours_text)?;
    let theirs = files.write("theirs.txt", theirs_text)?;
    let args = [
        "diff3", "--base", &base, "--ours", &ours, "--theirs", &theirs,
    ];
    let output = run(&args)?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?, "ONE\ntwo\nTHREE\n");
    assert!(output.stderr.is_empty());
    for (index, stdin) in [(2, base_text), (4, ours_text), (6, theirs_text)] {
        let mut args = args;
        args[index] = "-";
        let output = run_stdin(&args, stdin.as_bytes())?;
        assert_success(&output);
        assert_eq!(text(&output.stdout)?, "ONE\ntwo\nTHREE\n");
        assert!(output.stderr.is_empty());
    }
    Ok(())
}

#[test]
fn diff3_conflicts_have_exit_one_and_custom_labels() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    let base = files.write("base", "ancestor")?;
    let ours = files.write("ours", "local")?;
    let theirs = files.write("theirs", "remote")?;
    let output = run(&[
        "diff3",
        "--base",
        &base,
        "--ours",
        &ours,
        "--theirs",
        &theirs,
        "--label-base",
        "original.txt",
        "--label-ours",
        "our.txt",
        "--label-theirs",
        "their.txt",
    ])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    assert_eq!(text(&output.stdout)?,
        "<<<<<<< our.txt\nlocal\n||||||| original.txt\nancestor\n=======\nremote\n>>>>>>> their.txt\n");
    Ok(())
}

#[test]
fn diff3_preserves_identical_crlf_empty_and_unterminated_files() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    for input in ["", "same", "same\n", "same\r\nlast\r\n", "same\r\nlast"] {
        let path = files.write("same", input)?;
        let output = run(&["diff3", "--base", &path, "--ours", &path, "--theirs", &path])?;
        assert_success(&output);
        assert_eq!(text(&output.stdout)?, input);
    }
    let base = files.write("base", "one\r\ntwo\r\nthree")?;
    let ours = files.write("ours", "ONE\r\ntwo\r\nthree")?;
    let theirs = files.write("theirs", "one\r\ntwo\r\nTHREE")?;
    let output = run(&[
        "diff3", "--base", &base, "--ours", &ours, "--theirs", &theirs,
    ])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?, "ONE\r\ntwo\r\nTHREE");
    Ok(())
}

#[test]
fn diff3_invalid_files_stdin_and_labels_have_hints() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    let good = files.write("good", "text")?;
    let invalid = files.write("invalid", [0xff, 0xfe])?;
    let huge = files.write("huge", vec![b'x'; 256 * 1024 + 1])?;
    for bad in [files.path("missing"), invalid, huge] {
        let output = run(&["diff3", "--base", &bad, "--ours", &good, "--theirs", &good])?;
        assert_input_error(&output);
    }
    assert_input_error(&run_stdin(
        &["diff3", "--base", "-", "--ours", &good, "--theirs", &good],
        b"\xff",
    )?);
    for args in [
        ["diff3", "--base", "-", "--ours", "-", "--theirs", &good],
        ["diff3", "--base", &good, "--ours", "-", "--theirs", "-"],
        ["diff3", "--base", "-", "--ours", &good, "--theirs", "-"],
    ] {
        assert_input_error(&run_stdin(&args, b"")?);
    }
    assert_input_error(&run(&[
        "diff3",
        "--base",
        &good,
        "--ours",
        &good,
        "--theirs",
        &good,
        "--label-base",
        "not\na label",
    ])?);
    Ok(())
}

#[test]
fn qr_arguments_stdin_ecc_svg_and_output_files() -> Result<(), Box<dyn Error>> {
    let output = run(&["qr", "HELLO WORLD"])?;
    assert_success(&output);
    let terminal = text(&output.stdout)?;
    assert_eq!(terminal.lines().count(), 15);
    assert!(terminal.lines().all(|line| line.chars().count() == 29));
    assert!(terminal.lines().take(2).all(|line| line.trim().is_empty()));
    for args in [&["qr"][..], &["qr", "-"][..]] {
        let output = run_stdin(args, b"HELLO WORLD")?;
        assert_success(&output);
        assert_eq!(text(&output.stdout)?, terminal);
    }
    for level in ["L", "M", "Q", "H", "h"] {
        let output = run(&["qr", "Привіт 🌍", "--ecc", level, "--svg"])?;
        assert_success(&output);
        let svg = text(&output.stdout)?;
        assert!(svg.starts_with("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(!svg.contains('\u{1b}'));
    }
    let inverted = run(&["qr", "HELLO WORLD", "--invert"])?;
    assert_success(&inverted);
    assert!(text(&inverted.stdout)?
        .lines()
        .next()
        .is_some_and(|line| line.chars().all(|c| c == '█')));
    let low = run(&["qr", "HELLO WORLD", "--svg", "--ecc", "L"])?;
    let high = run(&["qr", "HELLO WORLD", "--svg", "--ecc", "H"])?;
    assert_success(&low);
    assert_success(&high);
    assert_ne!(low.stdout, high.stdout);

    let files = ToolFiles::new()?;
    let svg_path = files.path("code.svg");
    let svg = run(&["qr", "hello", "--svg", "--invert"])?;
    assert_success(&svg);
    let output = run(&["qr", "hello", "--svg", "--invert", "--output", &svg_path])?;
    assert_success(&output);
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read(&svg_path)?, svg.stdout);
    let text_path = files.path("code.txt");
    let output = run_stdin(&["qr", "-", "--output", &text_path], b"HELLO WORLD")?;
    assert_success(&output);
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&text_path)?, terminal);
    Ok(())
}

#[test]
fn qr_invalid_inputs_and_unwritable_outputs_have_hints() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    assert_input_error(&run(&["qr", ""])?);
    assert_input_error(&run_stdin(&["qr", "-", "--ecc", "H"], &vec![b'a'; 3000])?);
    for path in [
        files.0.to_string_lossy().into_owned(),
        files.path("missing/code.svg"),
    ] {
        assert_input_error(&run(&["qr", "hello", "--svg", "--output", &path])?);
    }
    let output = run(&["qr", "hello", "--ecc", "invalid"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr)?.contains("possible values"));
    Ok(())
}

#[test]
fn jsonschema_arguments_stdin_and_instance_error_paths() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    let schema = files.write("schema.json", r#"{"type":"object","required":["name"],"properties":{"name":{"type":"string"},"items":{"type":"array","items":{"type":"integer"}}}}"#)?;
    let instance = r#"{"name":"Ada","items":[1,2]}"#;
    let output = run(&["jsonschema", "--schema", &schema, instance])?;
    assert_success(&output);
    assert_eq!(text(&output.stdout)?, "Valid.\n");
    for args in [
        vec!["jsonschema", "--schema", &schema],
        vec!["jsonschema", "--schema", &schema, "-"],
    ] {
        let output = run_stdin(&args, instance.as_bytes())?;
        assert_success(&output);
        assert_eq!(text(&output.stdout)?, "Valid.\n");
    }
    for invalid in [
        "42",
        "{}",
        r#"{"name":1}"#,
        r#"{"name":"Ada","items":[1,"bad"]}"#,
    ] {
        let output = run(&["jsonschema", "--schema", &schema, invalid])?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        assert!(!output.stdout.is_empty());
        assert!(!text(&output.stdout)?.contains("Valid."));
    }
    let output = run_stdin(
        &["jsonschema", "--schema", &schema, "-"],
        br#"{"name":"Ada","items":[1,"bad"]}"#,
    )?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stdout)?.contains("/items/1"));
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn jsonschema_internal_references_patterns_enums_and_precise_numbers() -> Result<(), Box<dyn Error>>
{
    let files = ToolFiles::new()?;
    let schema = files.write("schema.json", r##"{"$defs":{"person":{"type":"object","properties":{"name":{"type":"string","pattern":"^[A-Z]"},"role":{"enum":["engineer","writer"]}},"required":["name","role"]}},"$ref":"#/$defs/person"}"##)?;
    assert_success(&run(&[
        "jsonschema",
        "--schema",
        &schema,
        r#"{"name":"Ada","role":"engineer"}"#,
    ])?);
    for instance in [
        r#"{"name":"ada","role":"engineer"}"#,
        r#"{"name":"Ada","role":"admin"}"#,
    ] {
        let output = run(&["jsonschema", "--schema", &schema, instance])?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
    }
    let schema = files.write("number.json", r#"{"type":"integer","minimum":123456789012345678901234567890,"maximum":123456789012345678901234567890}"#)?;
    assert_success(&run(&[
        "jsonschema",
        "--schema",
        &schema,
        "123456789012345678901234567890",
    ])?);
    for instance in [
        "123456789012345678901234567889",
        "123456789012345678901234567891",
    ] {
        let output = run(&["jsonschema", "--schema", &schema, instance])?;
        assert_eq!(output.status.code(), Some(1));
        assert!(text(&output.stdout)?.starts_with("\"\":"));
    }
    Ok(())
}

#[test]
fn jsonschema_invalid_schemas_files_and_json_have_hints() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    for (name, contents) in [
        ("malformed.json", "{"),
        ("invalid.json", r#"{"type":123}"#),
        (
            "external.json",
            r#"{"$ref":"https://example.invalid/schema.json"}"#,
        ),
        ("file-ref.json", r#"{"$ref":"file:///secret.json"}"#),
        ("relative-ref.json", r#"{"$ref":"other.json"}"#),
    ] {
        let schema = files.write(name, contents)?;
        assert_input_error(&run(&["jsonschema", "--schema", &schema, "{}"])?);
    }
    let invalid_utf8 = files.write("invalid-utf8.json", [0xff])?;
    let huge = files.write("huge.json", vec![b' '; 1024 * 1024 + 1])?;
    for schema in [files.path("missing.json"), invalid_utf8, huge] {
        assert_input_error(&run(&["jsonschema", "--schema", &schema, "{}"])?);
    }
    let schema = files.write("good.json", "{}")?;
    for instance in ["", "{oops}"] {
        assert_input_error(&run(&["jsonschema", "--schema", &schema, instance])?);
    }
    Ok(())
}

#[test]
fn new_tool_stdin_reads_enforce_size_and_utf8_limits() -> Result<(), Box<dyn Error>> {
    let files = ToolFiles::new()?;
    let schema = files.write("schema.json", "{}")?;
    for args in [
        vec!["base32", "encode", "-"],
        vec!["base32", "decode", "-"],
        vec!["headers", "-"],
        vec!["qr", "-"],
        vec!["jsonschema", "--schema", &schema, "-"],
    ] {
        assert_input_error(&run_stdin(&args, b"\xff")?);
        let size = if args[0] == "qr" {
            7090
        } else {
            1024 * 1024 + 1
        };
        let output = run_stdin(&args, &vec![b'a'; size])?;
        assert_input_error(&output);
        assert!(text(&output.stderr)?.contains("limit"));
    }
    Ok(())
}

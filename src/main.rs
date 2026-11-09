use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use chrono::{SecondsFormat, Utc};
use clap::{CommandFactory, Parser};

use devtoys_cli::cli::{
    Base32Command, Base64Command, BinaryCommand, BytesArgs, CaseArgs, CharsetArg, Cli, ColorArgs,
    Command, CompletionShell, CompletionsArgs, CsvCommand, DiffArgs, HashAlgorithm, HexCommand,
    HmacAlgorithm, HmacArgs, HtmlCommand, HttpArgs, IpArgs, JsonCommand, JwtCommand, JwtPart,
    MimeArgs, NumberArgs, RegexCommand, TextCommand, TimestampCommand,
};
use devtoys_cli::error::{Result, ToolError};
use devtoys_cli::style;
use devtoys_cli::tools::hash::Algorithm;
use devtoys_cli::tools::random::Charset;
use devtoys_cli::tools::{
    base32, base64 as base64_tool, case as case_tool, color, cron::CronSchedule, csv as csv_tool,
    diff, diff3, encoding, hash, headers, hmac, http, ip, json, jsonschema, jwt, limits, lorem,
    mime, number, qr, random, regex as regex_tool, text, timestamp, url, uuid,
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let no_color = cli.no_color;
    match run(cli) {
        Ok(code) => code,
        Err(error) => {
            report_error(&error, error_color_enabled(no_color));
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    let color = stdout_color_enabled(cli.no_color);

    match cli.command {
        Some(Command::Base32 { command }) => run_base32(command),
        Some(Command::Headers(args)) => {
            let input = bounded_input(
                args.input,
                headers::MAX_INPUT,
                "devtoys headers 'Host: example.com'",
            )?;
            emit(&headers::parse(&input, args.redact)?)
        }
        Some(Command::Diff3(args)) => return run_diff3(args),
        Some(Command::Qr(args)) => {
            let input = bounded_input(args.input, qr::MAX_INPUT, "devtoys qr 'hello'")?;
            let output = qr::generate(&input, &args.ecc, args.invert, args.svg)?;
            if let Some(path) = args.output {
                std::fs::write(&path, output).map_err(|error| {
                    ToolError::new(format!("Could not write `{}`: {error}.", path.display()))
                        .with_hint("Choose a writable file path in an existing directory.")
                })
            } else {
                emit_exact(&output)
            }
        }
        Some(Command::JsonSchema(args)) => {
            let schema = limits::read_file(&args.schema, jsonschema::MAX_INPUT)?;
            let input = bounded_input(
                args.input,
                jsonschema::MAX_INPUT,
                "devtoys jsonschema --schema schema.json '{}'",
            )?;
            let report = jsonschema::validate(&schema, &input)?;
            emit(&report.text)?;
            return Ok(if report.valid {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            });
        }
        None => launch_tui(),
        Some(Command::Tui) => launch_tui(),
        Some(Command::Json { command }) => run_json(command, color),
        Some(Command::Base64 { command }) => run_base64(command),
        Some(Command::Url { command }) => run_url(command),
        Some(Command::Timestamp { command }) => run_timestamp(command),
        Some(Command::Cron(args)) => run_cron(args, color),
        Some(Command::Case(args)) => run_case(args),
        Some(Command::Number(args)) => run_number(args),
        Some(Command::Bytes(args)) => run_bytes(args),
        Some(Command::Color(args)) => run_color(args),
        Some(Command::Text { command }) => run_text(command),
        Some(Command::Diff(args)) => run_diff(args),
        Some(Command::Hex { command }) => run_hex(command),
        Some(Command::Binary { command }) => run_binary(command),
        Some(Command::Html { command }) => run_html(command),
        Some(Command::Csv { command }) => run_csv(command),
        Some(Command::Http(args)) => run_http(args),
        Some(Command::Mime(args)) => run_mime(args),
        Some(Command::Ip(args)) => run_ip(args),
        Some(Command::Hmac(args)) => run_hmac(args),
        Some(Command::Uuid(args)) => {
            let values = if args.v7 {
                uuid::generate_v7(usize::from(args.count), args.upper, !args.no_hyphens)?
            } else {
                uuid::generate(usize::from(args.count), args.upper, !args.no_hyphens)?
            };
            emit(&values.join("\n"))
        }
        Some(Command::Hash(args)) => {
            let algorithm = match args.algorithm {
                HashAlgorithm::Md5 => Algorithm::Md5,
                HashAlgorithm::Sha256 => Algorithm::Sha256,
                HashAlgorithm::Sha512 => Algorithm::Sha512,
            };
            if let Some(path) = args.file {
                let bytes = std::fs::read(&path).map_err(|error| {
                    ToolError::new(format!("Could not read `{}`: {error}.", path.display()))
                        .with_hint("Check that the path exists and is readable.")
                })?;
                emit(&hash::hash_bytes(algorithm, &bytes))
            } else {
                let input = input_or_stdin(args.input, "devtoys hash sha256 'hello world'")?;
                emit(&hash::hash(algorithm, &input))
            }
        }
        Some(Command::Lorem(args)) => {
            let text = lorem::generate(
                usize::from(args.paragraphs),
                usize::from(args.sentences),
                usize::from(args.words),
            )?;
            emit(&text)
        }
        Some(Command::Random(args)) => {
            let charset = match args.charset {
                CharsetArg::Alnum => Charset::Alnum,
                CharsetArg::Alpha => Charset::Alpha,
                CharsetArg::Lower => Charset::Lower,
                CharsetArg::Upper => Charset::Upper,
                CharsetArg::Numeric => Charset::Numeric,
                CharsetArg::Hex => Charset::Hex,
                CharsetArg::Base58 => Charset::Base58,
                CharsetArg::Symbols => Charset::Symbols,
            };
            let values = random::generate_strings(
                usize::from(args.length),
                charset,
                usize::from(args.count),
            )?;
            emit(&values.join("\n"))
        }
        Some(Command::Password(args)) => {
            let values = random::generate_passwords(random::PasswordOptions {
                length: usize::from(args.length),
                count: usize::from(args.count),
                lowercase: !args.no_lower,
                uppercase: !args.no_upper,
                digits: !args.no_digits,
                symbols: !args.no_symbols,
            })?;
            emit(&values.join("\n"))
        }
        Some(Command::Jwt { command }) => run_jwt(command, color),
        Some(Command::Regex { command }) => run_regex(command, color),
        Some(Command::Completions(args)) => run_completions(args),
    }
    .map(|()| ExitCode::SUCCESS)
}

fn run_base32(command: Base32Command) -> Result<()> {
    let output = match command {
        Base32Command::Encode(args) => {
            let input = bounded_input(
                args.input,
                base32::MAX_INPUT,
                "devtoys base32 encode 'hello'",
            )?;
            base32::encode(&input, !args.no_padding, args.lower)?
        }
        Base32Command::Decode(args) => {
            let input = bounded_input(
                args.input,
                base32::MAX_INPUT,
                "devtoys base32 decode NBSWY3DP",
            )?;
            base32::decode(&input)?
        }
    };
    emit(&output)
}

fn run_diff3(args: devtoys_cli::cli::Diff3Args) -> Result<ExitCode> {
    let paths = [&args.base, &args.ours, &args.theirs];
    if paths.iter().filter(|p| p.as_os_str() == "-").count() > 1 {
        return Err(ToolError::new("Only one diff3 input may come from stdin.")
            .with_hint("Use a file path for the other two inputs."));
    }
    let read = |path: &std::path::Path| {
        if path.as_os_str() == "-" {
            limits::read(io::stdin().lock(), diff3::MAX_INPUT, "stdin")
        } else {
            limits::read_file(path, diff3::MAX_INPUT)
        }
    };
    let result = diff3::merge(
        &read(&args.base)?,
        &read(&args.ours)?,
        &read(&args.theirs)?,
        [&args.label_base, &args.label_ours, &args.label_theirs],
    )?;
    emit_exact(&result.text)?;
    Ok(if result.conflicted {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn bounded_input(value: Option<String>, limit: usize, example: &str) -> Result<String> {
    match value {
        Some(value) if value != "-" => {
            limits::check(&value, limit, "Input")?;
            Ok(value)
        }
        None if io::stdin().is_terminal() => Err(ToolError::new("No input provided.")
            .with_hint(format!("Pass text or pipe stdin. Example: {example}"))),
        _ => limits::read(io::stdin().lock(), limit, "stdin"),
    }
}

fn emit_exact(text: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    match stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(ToolError::new(format!("Could not write stdout: {error}."))
            .with_hint("Check the output destination.")),
    }
}

fn run_json(command: JsonCommand, color: bool) -> Result<()> {
    match command {
        JsonCommand::Format(args) => {
            let input = input_or_stdin(args.input, "devtoys json format '{\"a\":1}'")?;
            let output = json::format(&input, usize::from(args.indent), color)?;
            emit(&output)
        }
        JsonCommand::Minify(args) => {
            let input = input_or_stdin(args.input, "devtoys json minify '{\"a\": 1}'")?;
            emit(&json::minify(&input)?)
        }
        JsonCommand::Get(args) => {
            let input = input_or_stdin(
                args.input,
                "devtoys json get user.name '{\"user\":{\"name\":\"Ada\"}}'",
            )?;
            emit(&json::query(&input, &args.path, args.raw)?)
        }
    }
}

fn run_base64(command: Base64Command) -> Result<()> {
    match command {
        Base64Command::Encode(args) => {
            let input = input_or_stdin(args.input, "devtoys base64 encode 'hello'")?;
            match (args.hex, args.url_safe) {
                (true, true) => emit(&base64_tool::encode_hex_url(&input)?),
                (true, false) => emit(&base64_tool::encode_hex(&input)?),
                (false, true) => emit(&base64_tool::encode_text_url(&input)),
                (false, false) => emit(&base64_tool::encode_text(&input)),
            }
        }
        Base64Command::Decode(args) => {
            let input = input_or_stdin(args.input, "devtoys base64 decode 'aGVsbG8='")?;
            if args.hex {
                emit(&base64_tool::decode_hex(&input)?)
            } else {
                emit(&base64_tool::decode_text(&input)?)
            }
        }
    }
}

fn run_url(command: devtoys_cli::cli::UrlCommand) -> Result<()> {
    match command {
        devtoys_cli::cli::UrlCommand::Encode(args) => {
            let input = input_or_stdin(args.input, "devtoys url encode 'a b'")?;
            emit(&url::encode(&input))
        }
        devtoys_cli::cli::UrlCommand::Decode(args) => {
            let input = input_or_stdin(args.input, "devtoys url decode 'a%20b'")?;
            emit(&url::decode(&input)?)
        }
        devtoys_cli::cli::UrlCommand::Parse(args) => {
            let input = input_or_stdin(
                args.input,
                "devtoys url parse 'https://example.com/path?q=rust'",
            )?;
            emit(&url::parse_info(&input)?)
        }
    }
}

fn run_timestamp(command: TimestampCommand) -> Result<()> {
    match command {
        TimestampCommand::Now { ms } => {
            let now = Utc::now();
            let value = if ms {
                now.timestamp_millis()
            } else {
                now.timestamp()
            };
            emit(&value.to_string())
        }
        TimestampCommand::ToDate(args) => {
            let input = input_or_stdin(args.timestamp, "devtoys timestamp to-date 1700000000")?;
            let forced = args.ms.then_some(timestamp::EpochUnit::Millis);
            let info = timestamp::parse_epoch(&input, forced)?;
            let output = if args.verbose {
                timestamp::describe_epoch(&info)
            } else if args.local {
                info.local.to_rfc3339_opts(SecondsFormat::AutoSi, false)
            } else {
                info.utc.to_rfc3339_opts(SecondsFormat::AutoSi, true)
            };
            emit(&output)
        }
        TimestampCommand::ToEpoch(args) => {
            let input = input_or_stdin(
                args.date,
                "devtoys timestamp to-epoch '2024-01-01 12:00:00'",
            )?;
            let value = timestamp::parse_date_to_utc(&input, args.utc)?;
            let output = if args.verbose {
                timestamp::describe_date(value)
            } else if args.ms {
                value.timestamp_millis().to_string()
            } else {
                value.timestamp().to_string()
            };
            emit(&output)
        }
    }
}

fn run_case(args: CaseArgs) -> Result<()> {
    let input = input_or_stdin(args.input, "devtoys case 'Hello world' --style kebab")?;
    let style = case_tool::CaseStyle::parse(&args.style)?;
    emit(&case_tool::convert(&input, style))
}

fn run_number(args: NumberArgs) -> Result<()> {
    let input = input_or_stdin(args.input, "devtoys number 255 --to 16")?;
    emit(&number::convert_base(
        &input, args.from, args.to, args.upper,
    )?)
}

fn run_bytes(args: BytesArgs) -> Result<()> {
    let input = input_or_stdin(args.input, "devtoys bytes '1.5 MiB'")?;
    emit(&number::convert_bytes(&input, args.binary)?)
}

fn run_color(args: ColorArgs) -> Result<()> {
    let input = input_or_stdin(args.input, "devtoys color '#58a6ff'")?;
    emit(&color::convert(&input)?)
}

fn run_text(command: TextCommand) -> Result<()> {
    match command {
        TextCommand::Stats(args) => {
            let input = input_or_stdin(args.input, "devtoys text stats 'Hello world'")?;
            emit(&text::format_stats(&input))
        }
        TextCommand::Sort(args) => {
            let input = input_or_stdin(args.input, "devtoys text sort --reverse")?;
            let options = text::SortOptions {
                reverse: args.reverse,
                unique: args.unique,
                ignore_case: args.ignore_case,
                trim: args.trim,
            };
            emit(&text::sort_lines(&input, options))
        }
        TextCommand::Dedupe(args) => {
            let input = input_or_stdin(args.input, "devtoys text dedupe --ignore-case")?;
            emit(&text::dedupe_lines(&input, args.ignore_case, args.trim))
        }
        TextCommand::Slug(args) => {
            let input = input_or_stdin(args.input, "devtoys text slug 'Crème brûlée'")?;
            emit(&text::slugify(&input))
        }
    }
}

fn run_diff(args: DiffArgs) -> Result<()> {
    let left_stdin = args.left_file.is_none() && args.left.as_deref() == Some("-");
    let right_stdin = args.right_file.is_none() && args.right.as_deref() == Some("-");
    if left_stdin && right_stdin {
        return Err(
            ToolError::new("Only one side of the diff can come from stdin.")
                .with_hint("Pass one side as an argument or with --left-file / --right-file."),
        );
    }

    let left = read_diff_side(
        args.left_file,
        args.left,
        "left",
        "devtoys diff 'old text' 'new text' or devtoys diff --left-file old.txt --right-file new.txt",
    )?;
    let right = read_diff_side(
        args.right_file,
        args.right,
        "right",
        "devtoys diff 'old text' 'new text' or devtoys diff --left-file old.txt --right-file new.txt",
    )?;
    emit(&diff::diff(&left, &right, usize::from(args.context))?)
}

fn read_diff_side(
    file: Option<PathBuf>,
    value: Option<String>,
    label: &str,
    example: &str,
) -> Result<String> {
    if let Some(path) = file {
        return read_text_file(&path);
    }
    match value {
        Some(value) if value == "-" => read_stdin(),
        Some(value) => Ok(value),
        None => Err(
            ToolError::new(format!("No {label} input provided.")).with_hint(format!(
            "Pass it as an argument, use `-` for stdin, or use --{label}-file. Example: {example}"
        )),
        ),
    }
}

fn read_text_file(path: &std::path::Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|error| {
        ToolError::new(format!("Could not read `{}`: {error}.", path.display()))
            .with_hint("Check that the file exists and contains UTF-8 text.")
    })
}

fn run_hex(command: HexCommand) -> Result<()> {
    match command {
        HexCommand::Encode(args) => {
            let input = input_or_stdin(args.input, "devtoys hex encode 'Hello'")?;
            emit(&encoding::hex_encode(&input, args.upper))
        }
        HexCommand::Decode(args) => {
            let input = input_or_stdin(args.input, "devtoys hex decode 48656c6c6f")?;
            emit(&encoding::hex_decode(&input)?)
        }
    }
}

fn run_binary(command: BinaryCommand) -> Result<()> {
    match command {
        BinaryCommand::Encode(args) => {
            let input = input_or_stdin(args.input, "devtoys binary encode 'Hi'")?;
            emit(&encoding::binary_encode(&input))
        }
        BinaryCommand::Decode(args) => {
            let input = input_or_stdin(args.input, "devtoys binary decode '01001000 01101001'")?;
            emit(&encoding::binary_decode(&input)?)
        }
    }
}

fn run_html(command: HtmlCommand) -> Result<()> {
    match command {
        HtmlCommand::Encode(args) => {
            let input = input_or_stdin(args.input, "devtoys html encode '<b>Hi</b>'")?;
            emit(&encoding::html_encode(&input))
        }
        HtmlCommand::Decode(args) => {
            let input = input_or_stdin(args.input, "devtoys html decode '&lt;b&gt;Hi&lt;/b&gt;'")?;
            emit(&encoding::html_decode(&input)?)
        }
    }
}

fn run_csv(command: CsvCommand) -> Result<()> {
    match command {
        CsvCommand::ToJson(args) => {
            let input = input_or_stdin(args.input, "devtoys csv to-json 'name,age'")?;
            let delimiter = csv_tool::parse_delimiter(&args.delimiter)?;
            emit(&csv_tool::to_json(&input, delimiter, !args.no_headers)?)
        }
        CsvCommand::FromJson(args) => {
            let input = input_or_stdin(args.input, "devtoys csv from-json '[{\"name\":\"Ada\"}]'")?;
            let delimiter = csv_tool::parse_delimiter(&args.delimiter)?;
            emit(&csv_tool::from_json(&input, delimiter)?)
        }
    }
}

fn run_http(args: HttpArgs) -> Result<()> {
    if args.list {
        return emit(&format_statuses(&http::list(None)?));
    }
    if let Some(class) = args.class {
        return emit(&format_statuses(&http::list(Some(class))?));
    }

    let input = input_or_stdin(args.code.map(|code| code.to_string()), "devtoys http 404")?;
    let code: u16 = input.trim().parse().map_err(|_| {
        ToolError::new(format!("`{}` is not an HTTP status code.", input.trim()))
            .with_hint("Use a number from 100 to 599, for example 404.")
    })?;
    emit(&http::format(http::lookup(code)?))
}

fn format_statuses(statuses: &[http::Status]) -> String {
    statuses
        .iter()
        .map(|status| format!("{:>3}  {}", status.code, status.reason))
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_mime(args: MimeArgs) -> Result<()> {
    let input = input_or_stdin(args.input, "devtoys mime application/json")?;
    emit(&mime::lookup(&input)?)
}

fn run_ip(args: IpArgs) -> Result<()> {
    let input = input_or_stdin(args.input, "devtoys ip 192.168.1.42/24")?;
    emit(&ip::inspect(&input)?)
}

fn run_hmac(args: HmacArgs) -> Result<()> {
    let input = input_or_stdin(args.input, "devtoys hmac sha256 secret 'message'")?;
    let algorithm = match args.algorithm {
        HmacAlgorithm::Sha256 => hmac::Algorithm::Sha256,
        HmacAlgorithm::Sha512 => hmac::Algorithm::Sha512,
    };
    emit(&hmac::hash(
        algorithm,
        args.key.as_bytes(),
        input.as_bytes(),
    )?)
}

fn run_cron(args: devtoys_cli::cli::CronArgs, color: bool) -> Result<()> {
    let expression = input_or_stdin(args.expression, "devtoys cron '*/15 * * * *'")?;
    let schedule = CronSchedule::parse(&expression)?;
    let mut lines = vec![
        format!(
            "{} {}",
            style::cyan(color, "Expression :"),
            schedule.expression()
        ),
        format!(
            "{} {}",
            style::cyan(color, "Description:"),
            schedule.describe()
        ),
        format!("{} local system time", style::cyan(color, "Timezone   :")),
    ];

    if args.next > 0 {
        let runs = schedule.next_runs(usize::from(args.next))?;
        lines.push(String::new());
        lines.push(style::bold(color, "Next runs:"));
        for (index, run) in runs.iter().enumerate() {
            lines.push(format!(
                "  {:>2}. {} ({})",
                index + 1,
                run.to_rfc3339_opts(SecondsFormat::Secs, false),
                run.format("%a")
            ));
        }
    }

    emit(&lines.join("\n"))
}

fn run_jwt(command: JwtCommand, color: bool) -> Result<()> {
    match command {
        JwtCommand::Decode(args) => {
            let token = input_or_stdin(args.token, "devtoys jwt decode <token>")?;
            let decoded = jwt::decode(&token)?;
            match args.part {
                JwtPart::Header => emit(&json::format(&decoded.header_raw, 2, color)?),
                JwtPart::Payload => emit(&json::format(&decoded.payload_raw, 2, color)?),
                JwtPart::Both => {
                    let mut output = String::new();
                    output.push_str(&style::bold(color, "HEADER"));
                    output.push('\n');
                    output.push_str(&json::format(&decoded.header_raw, 2, color)?);
                    output.push_str("\n\n");
                    output.push_str(&style::bold(color, "PAYLOAD"));
                    output.push('\n');
                    output.push_str(&json::format(&decoded.payload_raw, 2, color)?);

                    let claims = decoded.claims();
                    if !claims.is_empty() {
                        output.push_str("\n\n");
                        output.push_str(&style::bold(color, "CLAIMS"));
                        output.push('\n');
                        for (key, value) in claims {
                            output.push_str(&format!("{key:<5} {value}\n"));
                        }
                    }

                    output.push('\n');
                    output.push_str(&style::yellow(color, jwt::SIGNATURE_WARNING));
                    output.push('\n');
                    output.push_str(&format!(
                        "Signature: {} base64url characters.",
                        decoded.signature_length
                    ));
                    emit(&output)
                }
            }
        }
    }
}

fn run_regex(command: RegexCommand, color: bool) -> Result<()> {
    match command {
        RegexCommand::Test(args) => {
            let text = input_or_stdin(args.text, "devtoys regex test '\\d+' 'order 123'")?;
            let report = regex_tool::test(
                &args.pattern,
                &text,
                &regex_tool::RegexOptions {
                    case_insensitive: args.insensitive,
                    multi_line: args.multiline,
                    dot_matches_new_line: args.dotall,
                    replacement: args.replace,
                },
            )?;

            let mut output = format!(
                "{} {}\n{} {}\n\n",
                style::cyan(color, "Pattern:"),
                args.pattern,
                style::cyan(color, "Matches:"),
                report.matches.len()
            );

            if report.matches.is_empty() {
                output.push_str("No matches found.");
            } else {
                for (index, found) in report.matches.iter().enumerate() {
                    let (line, column) = regex_tool::line_column(&text, found.start);
                    output.push_str(&format!(
                        "{} {} (line {line}, column {column}, bytes {}..{})\n",
                        style::green(color, &format!("Match #{}", index + 1)),
                        style::dim(color, &format!("\"{}\"", visible_text(&found.text))),
                        found.start,
                        found.end
                    ));
                    for (group_index, group) in found.groups.iter().enumerate() {
                        let value = group
                            .as_deref()
                            .map_or_else(|| "(no match)".to_owned(), visible_text);
                        output.push_str(&format!(
                            "  {} {}\n",
                            style::dim(color, &format!("group {}:", group_index + 1)),
                            value
                        ));
                    }
                    for (name, group) in &found.named_groups {
                        let value = group
                            .as_deref()
                            .map_or_else(|| "(no match)".to_owned(), visible_text);
                        output.push_str(&format!(
                            "  {} {}\n",
                            style::dim(color, &format!("{name}:")),
                            value
                        ));
                    }
                }
                if output.ends_with('\n') {
                    output.pop();
                }
            }

            if let Some(replaced) = report.replaced {
                output.push_str("\n\n");
                output.push_str(&style::bold(color, "Replacement:"));
                output.push('\n');
                output.push_str(&replaced);
            }

            emit(&output)
        }
    }
}

fn run_completions(args: CompletionsArgs) -> Result<()> {
    let shell = match args.shell {
        CompletionShell::Bash => clap_complete::Shell::Bash,
        CompletionShell::Zsh => clap_complete::Shell::Zsh,
        CompletionShell::Fish => clap_complete::Shell::Fish,
        CompletionShell::Powershell => clap_complete::Shell::PowerShell,
        CompletionShell::Elvish => clap_complete::Shell::Elvish,
    };
    let mut command = Cli::command();
    let mut buffer = Vec::new();
    clap_complete::generate(shell, &mut command, "devtoys", &mut buffer);
    let script = String::from_utf8(buffer)
        .map_err(|_| ToolError::new("The generated completion script was not valid UTF-8."))?;
    emit(script.trim_end_matches('\n'))
}

fn input_or_stdin(value: Option<String>, example: &str) -> Result<String> {
    match value {
        Some(value) if value != "-" => Ok(value),
        Some(_) => read_stdin(),
        None => {
            if io::stdin().is_terminal() {
                Err(ToolError::new("No input provided.").with_hint(format!(
                    "Pass a value as an argument or pipe data via stdin. Example: {example}"
                )))
            } else {
                read_stdin()
            }
        }
    }
}

fn read_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|error| {
        ToolError::new(format!("Could not read stdin: {error}."))
            .with_hint("stdin must be valid UTF-8.")
    })?;
    Ok(input)
}

fn stdout_color_enabled(no_color: bool) -> bool {
    !no_color && environment_allows_color() && io::stdout().is_terminal()
}

fn error_color_enabled(no_color: bool) -> bool {
    !no_color && environment_allows_color() && io::stderr().is_terminal()
}

fn environment_allows_color() -> bool {
    env::var_os("NO_COLOR").is_none() && env::var("TERM").map_or(true, |term| term != "dumb")
}

fn emit(text: &str) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if let Err(error) = handle
        .write_all(text.as_bytes())
        .and_then(|()| handle.write_all(b"\n"))
        .and_then(|()| handle.flush())
    {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(ToolError::new(format!(
            "Could not write to stdout: {error}."
        )));
    }
    Ok(())
}

fn report_error(error: &ToolError, color: bool) {
    let mut output = format!("{} {}\n", style::red(color, "error:"), error.message());
    if let Some(hint) = error.hint() {
        output.push_str(&format!("  {} {hint}\n", style::yellow(color, "hint:")));
    }
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    let _ = handle.write_all(output.as_bytes());
    let _ = handle.flush();
}

fn visible_text(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    if escaped.chars().count() > 120 {
        let mut truncated: String = escaped.chars().take(117).collect();
        truncated.push_str("...");
        truncated
    } else {
        escaped
    }
}

#[cfg(feature = "tui")]
fn launch_tui() -> Result<()> {
    devtoys_cli::tui::run()
}

#[cfg(not(feature = "tui"))]
fn launch_tui() -> Result<()> {
    Err(
        ToolError::new("This build was compiled without the TUI feature.").with_hint(
            "Reinstall with default features or run a CLI command such as `devtoys --help`.",
        ),
    )
}

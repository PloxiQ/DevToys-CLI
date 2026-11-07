use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

use chrono::{SecondsFormat, Utc};
use clap::Parser;

use devtoys_cli::cli::{
    Base64Command, CharsetArg, Cli, Command, HashAlgorithm, JsonCommand, JwtCommand, JwtPart,
    RegexCommand, TimestampCommand,
};
use devtoys_cli::error::{Result, ToolError};
use devtoys_cli::style;
use devtoys_cli::tools::hash::Algorithm;
use devtoys_cli::tools::random::Charset;
use devtoys_cli::tools::{
    base64 as base64_tool, cron::CronSchedule, hash, json, jwt, lorem, random, regex as regex_tool,
    timestamp, url, uuid,
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let no_color = cli.no_color;
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report_error(&error, error_color_enabled(no_color));
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let color = stdout_color_enabled(cli.no_color);

    match cli.command {
        None => launch_tui(),
        Some(Command::Tui) => launch_tui(),
        Some(Command::Json { command }) => run_json(command, color),
        Some(Command::Base64 { command }) => run_base64(command),
        Some(Command::Url { command }) => run_url(command),
        Some(Command::Timestamp { command }) => run_timestamp(command),
        Some(Command::Cron(args)) => run_cron(args, color),
        Some(Command::Uuid(args)) => {
            let values = uuid::generate(usize::from(args.count), args.upper, !args.no_hyphens)?;
            emit(&values.join("\n"))
        }
        Some(Command::Hash(args)) => {
            let input = input_or_stdin(args.input, "devtoys hash sha256 'hello world'")?;
            let algorithm = match args.algorithm {
                HashAlgorithm::Md5 => Algorithm::Md5,
                HashAlgorithm::Sha256 => Algorithm::Sha256,
                HashAlgorithm::Sha512 => Algorithm::Sha512,
            };
            emit(&hash::hash(algorithm, &input))
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
    }
}

fn run_base64(command: Base64Command) -> Result<()> {
    match command {
        Base64Command::Encode(args) => {
            let input = input_or_stdin(args.input, "devtoys base64 encode 'hello'")?;
            if args.hex {
                emit(&base64_tool::encode_hex(&input)?)
            } else {
                emit(&base64_tool::encode_text(&input))
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

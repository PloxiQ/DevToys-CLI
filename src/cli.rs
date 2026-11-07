use clap::{Args, Parser, Subcommand, ValueEnum};

const AFTER_HELP: &str = "\
EXAMPLES:
  devtoys
      Launch the interactive TUI.
  devtoys json format '{\"a\":1}'
      Pretty-print JSON (or pipe it: echo '{\"a\":1}' | devtoys json format).
  devtoys base64 encode --hex 48656c6c6f
  devtoys jwt decode <token>
  devtoys cron '*/15 * * * *'
  devtoys regex test '\\d+' 'order 123'
  devtoys timestamp to-date 1700000000
  devtoys hash sha256 'hello world'

Use `devtoys <command> --help` for tool-specific help.";

/// A fast, offline developer toolbox for the terminal.
#[derive(Debug, Parser)]
#[command(
    name = "devtoys",
    version,
    about = "A fast, offline developer toolbox for the terminal - CLI and TUI.",
    long_about = "DevToys CLI is an offline, zero-telemetry developer toolbox: format JSON, decode JWTs, test regexes, convert timestamps, generate passwords and more. Run without arguments to open the interactive TUI.",
    after_help = AFTER_HELP,
    propagate_version = true
)]
pub struct Cli {
    /// Disable ANSI colors (NO_COLOR is also respected).
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Format, minify and inspect JSON.
    Json {
        #[command(subcommand)]
        command: JsonCommand,
    },

    /// Encode and decode Base64 (text or hex).
    #[command(alias = "b64")]
    Base64 {
        #[command(subcommand)]
        command: Base64Command,
    },

    /// Percent-encode and decode URLs.
    Url {
        #[command(subcommand)]
        command: UrlCommand,
    },

    /// Convert between Unix timestamps and human-readable dates.
    Timestamp {
        #[command(subcommand)]
        command: TimestampCommand,
    },

    /// Explain a cron expression and show upcoming runs.
    Cron(CronArgs),

    /// Generate UUID v4 values.
    Uuid(UuidArgs),

    /// Hash text with MD5, SHA-256 or SHA-512.
    Hash(HashArgs),

    /// Generate Lorem Ipsum filler text.
    Lorem(LoremArgs),

    /// Generate random strings.
    Random(RandomArgs),

    /// Generate secure random passwords.
    #[command(alias = "pw")]
    Password(PasswordArgs),

    /// Decode a JWT for inspection (signature is not verified).
    Jwt {
        #[command(subcommand)]
        command: JwtCommand,
    },

    /// Test regular expressions against text.
    #[command(alias = "re")]
    Regex {
        #[command(subcommand)]
        command: RegexCommand,
    },

    /// Launch the interactive TUI (default when no command is given).
    Tui,
}

#[derive(Debug, Subcommand)]
pub enum JsonCommand {
    /// Pretty-print JSON with optional terminal syntax highlighting.
    #[command(alias = "fmt", alias = "pretty")]
    Format(JsonFormatArgs),
    /// Remove insignificant whitespace from JSON.
    #[command(alias = "min")]
    Minify(JsonInputArgs),
}

#[derive(Debug, Args)]
pub struct JsonInputArgs {
    /// JSON input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Args)]
pub struct JsonFormatArgs {
    /// JSON input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Number of spaces per indentation level (0-16).
    #[arg(short, long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(0..=16))]
    pub indent: u8,
}

#[derive(Debug, Subcommand)]
pub enum Base64Command {
    /// Encode text (or hex with --hex) as Base64.
    Encode(Base64EncodeArgs),
    /// Decode Base64 as UTF-8 text (or hex with --hex).
    Decode(Base64DecodeArgs),
}

#[derive(Debug, Args)]
pub struct Base64EncodeArgs {
    /// Input text. With --hex, input is parsed as hex. Omit or use `-` for stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Treat the input as a hex string.
    #[arg(long)]
    pub hex: bool,
}

#[derive(Debug, Args)]
pub struct Base64DecodeArgs {
    /// Base64 input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Print the decoded bytes as lowercase hex instead of UTF-8 text.
    #[arg(long)]
    pub hex: bool,
}

#[derive(Debug, Subcommand)]
pub enum UrlCommand {
    /// Percent-encode a string for use in a URL.
    Encode(UrlArgs),
    /// Decode percent-encoded text (`+` becomes a space).
    Decode(UrlArgs),
}

#[derive(Debug, Args)]
pub struct UrlArgs {
    /// Input text. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum TimestampCommand {
    /// Print the current Unix timestamp.
    Now {
        /// Print milliseconds instead of seconds.
        #[arg(long)]
        ms: bool,
    },
    /// Convert a Unix timestamp to a date.
    #[command(alias = "date")]
    ToDate(TimestampToDateArgs),
    /// Convert a date to a Unix timestamp.
    #[command(alias = "epoch")]
    ToEpoch(TimestampToEpochArgs),
}

#[derive(Debug, Args)]
pub struct TimestampToDateArgs {
    /// Unix seconds or milliseconds. Omit or use `-` to read stdin.
    #[arg(value_name = "TIMESTAMP")]
    pub timestamp: Option<String>,

    /// Force interpretation as milliseconds (auto-detected for large values).
    #[arg(long)]
    pub ms: bool,

    /// Print local time instead of UTC.
    #[arg(long)]
    pub local: bool,

    /// Print a detailed breakdown.
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct TimestampToEpochArgs {
    /// Date string. Omit or use `-` to read stdin. Supports `now`.
    #[arg(value_name = "DATE")]
    pub date: Option<String>,

    /// Treat dates without an offset as UTC instead of local time.
    #[arg(long)]
    pub utc: bool,

    /// Output milliseconds instead of seconds.
    #[arg(long)]
    pub ms: bool,

    /// Print a detailed breakdown.
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct CronArgs {
    /// Five-field cron expression (quote it in your shell).
    #[arg(value_name = "EXPRESSION")]
    pub expression: Option<String>,

    /// Number of upcoming runs to show (0-50).
    #[arg(short = 'n', long, default_value_t = 5, value_parser = clap::value_parser!(u8).range(0..=50))]
    pub next: u8,
}

#[derive(Debug, Args)]
pub struct UuidArgs {
    /// Number of UUIDs to generate.
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=1000))]
    pub count: u16,

    /// Print uppercase UUIDs.
    #[arg(long)]
    pub upper: bool,

    /// Omit hyphens (32 characters).
    #[arg(long)]
    pub no_hyphens: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HashAlgorithm {
    Md5,
    Sha256,
    Sha512,
}

#[derive(Debug, Args)]
pub struct HashArgs {
    /// Hash algorithm.
    #[arg(value_enum)]
    pub algorithm: HashAlgorithm,

    /// Text to hash. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Args)]
pub struct LoremArgs {
    /// Number of paragraphs (1-20).
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=20))]
    pub paragraphs: u8,

    /// Sentences per paragraph (1-20).
    #[arg(short, long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..=20))]
    pub sentences: u8,

    /// Approximate words per sentence (3-50).
    #[arg(short, long, default_value_t = 12, value_parser = clap::value_parser!(u8).range(3..=50))]
    pub words: u8,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CharsetArg {
    Alnum,
    Alpha,
    Lower,
    Upper,
    Numeric,
    Hex,
    Base58,
    Symbols,
}

#[derive(Debug, Args)]
pub struct RandomArgs {
    /// Length of each string (1-4096).
    #[arg(value_name = "LENGTH", default_value_t = 32, value_parser = clap::value_parser!(u16).range(1..=4096))]
    pub length: u16,

    /// Character set to use.
    #[arg(long, value_enum, default_value_t = CharsetArg::Alnum)]
    pub charset: CharsetArg,

    /// Number of strings to generate.
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=1000))]
    pub count: u16,
}

#[derive(Debug, Args)]
pub struct PasswordArgs {
    /// Password length (4-256).
    #[arg(short, long, default_value_t = 20, value_parser = clap::value_parser!(u16).range(4..=256))]
    pub length: u16,

    /// Number of passwords to generate.
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=1000))]
    pub count: u16,

    /// Exclude lowercase letters.
    #[arg(long)]
    pub no_lower: bool,

    /// Exclude uppercase letters.
    #[arg(long)]
    pub no_upper: bool,

    /// Exclude digits.
    #[arg(long)]
    pub no_digits: bool,

    /// Exclude symbols.
    #[arg(long)]
    pub no_symbols: bool,
}

#[derive(Debug, Subcommand)]
pub enum JwtCommand {
    /// Decode JWT header and payload. The signature is never validated.
    Decode(JwtDecodeArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum JwtPart {
    Both,
    Header,
    Payload,
}

#[derive(Debug, Args)]
pub struct JwtDecodeArgs {
    /// Compact JWT. Omit or use `-` to read stdin.
    #[arg(value_name = "TOKEN")]
    pub token: Option<String>,

    /// Which decoded part to print.
    #[arg(long, value_enum, default_value_t = JwtPart::Both)]
    pub part: JwtPart,
}

#[derive(Debug, Subcommand)]
pub enum RegexCommand {
    /// Find matches and optionally replace them.
    Test(RegexTestArgs),
}

#[derive(Debug, Args)]
pub struct RegexTestArgs {
    /// Regular expression pattern.
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// Text to test. Omit or use `-` to read stdin.
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,

    /// Case-insensitive matching.
    #[arg(short = 'i', long)]
    pub insensitive: bool,

    /// Make `^` and `$` match line boundaries.
    #[arg(short = 'm', long)]
    pub multiline: bool,

    /// Make `.` match newlines.
    #[arg(short = 's', long)]
    pub dotall: bool,

    /// Show the result of replacing every match.
    #[arg(short = 'r', long, value_name = "REPLACEMENT")]
    pub replace: Option<String>,
}

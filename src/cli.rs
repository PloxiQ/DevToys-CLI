use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

const AFTER_HELP: &str = "\
EXAMPLES:
  devtoys
      Launch the interactive TUI.
  devtoys json format '{\"a\":1}'
      Pretty-print JSON (or pipe it: echo '{\"a\":1}' | devtoys json format).
  devtoys json get user.name '{\"user\":{\"name\":\"Ada\"}}'
  devtoys base64 encode --hex 48656c6c6f
  devtoys base32 encode 'hello' --no-padding
  devtoys headers 'Authorization: Bearer secret' --redact
  devtoys diff3 --base base.txt --ours ours.txt --theirs theirs.txt
  devtoys qr 'hello' --svg --output code.svg
  devtoys jsonschema --schema schema.json '{\"name\":\"Ada\"}'
  devtoys case 'Hello world' --style kebab
  devtoys number 255 --to 16
  devtoys diff old.txt new.txt
  devtoys csv to-json 'name,age\nAda,30'
  devtoys jwt decode <token>
  devtoys cron '*/15 * * * *'
  devtoys regex test '\\d+' 'order 123'
  devtoys timestamp to-date 1700000000
  devtoys hash sha256 'hello world'
  devtoys hash sha256 --file Cargo.toml
  devtoys hmac sha256 secret 'message'
  devtoys completions bash

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
    /// Encode or decode RFC 4648 Base32.
    Base32 {
        #[command(subcommand)]
        command: Base32Command,
    },
    /// Inspect an HTTP header block offline (maximum 1 MiB).
    Headers(HeadersArgs),
    /// Merge three UTF-8 files; conflicts exit 1 and print conflict markers.
    Diff3(Diff3Args),
    /// Generate a QR code as terminal half blocks or SVG.
    Qr(QrArgs),
    /// Validate JSON against a local schema; external references are disabled.
    #[command(name = "jsonschema")]
    JsonSchema(JsonSchemaArgs),
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

    /// Convert text between naming conventions (camelCase, snake_case, ...).
    Case(CaseArgs),

    /// Convert numbers between bases 2-36.
    Number(NumberArgs),

    /// Convert byte counts to human-readable sizes.
    #[command(alias = "size")]
    Bytes(BytesArgs),

    /// Parse a CSS color and show every common representation.
    Color(ColorArgs),

    /// Text statistics, sorting, deduplication and slugs.
    Text {
        #[command(subcommand)]
        command: TextCommand,
    },

    /// Compare two text blocks with a unified diff.
    Diff(DiffArgs),

    /// Encode and decode hexadecimal text.
    Hex {
        #[command(subcommand)]
        command: HexCommand,
    },

    /// Encode and decode binary text.
    Binary {
        #[command(subcommand)]
        command: BinaryCommand,
    },

    /// Encode and decode HTML entities.
    Html {
        #[command(subcommand)]
        command: HtmlCommand,
    },

    /// Convert between CSV and JSON.
    Csv {
        #[command(subcommand)]
        command: CsvCommand,
    },

    /// Look up HTTP status codes.
    Http(HttpArgs),

    /// Resolve file extensions and MIME types.
    Mime(MimeArgs),

    /// Parse an IP address or CIDR block.
    Ip(IpArgs),

    /// Compute an HMAC signature.
    Hmac(HmacArgs),

    /// Generate UUID v4 or time-ordered v7 values.
    Uuid(UuidArgs),

    /// Hash text or a file with MD5, SHA-256 or SHA-512.
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

    /// Generate shell completion scripts.
    Completions(CompletionsArgs),

    /// Launch the interactive TUI (default when no command is given).
    Tui,
}

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: CompletionShell,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

#[derive(Debug, Subcommand)]
pub enum Base32Command {
    /// Encode UTF-8 text as RFC 4648 Base32 (maximum 1 MiB).
    Encode(Base32EncodeArgs),
    /// Decode Base32 as UTF-8; accepts mixed case, whitespace and optional padding.
    Decode(EncodingInputArgs),
}

#[derive(Debug, Args)]
pub struct Base32EncodeArgs {
    /// Text input. Omit or use `-` to read stdin.
    pub input: Option<String>,
    /// Omit trailing padding.
    #[arg(long)]
    pub no_padding: bool,
    /// Emit lowercase letters.
    #[arg(long)]
    pub lower: bool,
}

#[derive(Debug, Args)]
pub struct HeadersArgs {
    /// Header block. Omit or use `-` to read stdin.
    pub input: Option<String>,
    /// Hide authorization, cookies and X-Api-Key values.
    #[arg(long)]
    pub redact: bool,
}

#[derive(Debug, Args)]
pub struct Diff3Args {
    /// Common ancestor file (maximum 256 KiB / 10,000 lines).
    #[arg(long, value_name = "PATH|-")]
    pub base: PathBuf,
    /// Our file; at most one input may be `-` for stdin.
    #[arg(long, value_name = "PATH|-")]
    pub ours: PathBuf,
    /// Their file.
    #[arg(long, value_name = "PATH|-")]
    pub theirs: PathBuf,
    /// Conflict label for the common ancestor.
    #[arg(long, default_value = "base")]
    pub label_base: String,
    /// Conflict label for our file.
    #[arg(long, default_value = "ours")]
    pub label_ours: String,
    /// Conflict label for their file.
    #[arg(long, default_value = "theirs")]
    pub label_theirs: String,
}

#[derive(Debug, Args)]
pub struct QrArgs {
    /// Text to encode. Omit or use `-` to read stdin.
    pub input: Option<String>,
    /// Error correction level (L, M, Q, H).
    #[arg(long, default_value = "M", value_parser = ["L", "M", "Q", "H"], ignore_case = true)]
    pub ecc: String,
    /// Invert dark and light modules.
    #[arg(long)]
    pub invert: bool,
    /// Emit SVG instead of terminal half blocks.
    #[arg(long)]
    pub svg: bool,
    /// Write the selected format to this path instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct JsonSchemaArgs {
    /// Local UTF-8 JSON Schema file (maximum 1 MiB).
    #[arg(long, value_name = "PATH")]
    pub schema: PathBuf,
    /// JSON instance (maximum 1 MiB). Omit or use `-` to read stdin.
    #[arg(value_name = "JSON")]
    pub input: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum JsonCommand {
    /// Pretty-print JSON with optional terminal syntax highlighting.
    #[command(alias = "fmt", alias = "pretty")]
    Format(JsonFormatArgs),
    /// Remove insignificant whitespace from JSON.
    #[command(alias = "min")]
    Minify(JsonInputArgs),
    /// Extract a value with a JSON Pointer or dot path.
    #[command(alias = "query")]
    Get(JsonGetArgs),
}

#[derive(Debug, Args)]
pub struct JsonGetArgs {
    /// JSON Pointer (`/user/name`) or dot path (`user.name`, `tools[0]`).
    #[arg(value_name = "PATH")]
    pub path: String,

    /// JSON input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Print string results without surrounding quotes.
    #[arg(long)]
    pub raw: bool,
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

    /// Emit URL-safe Base64 without padding.
    #[arg(long)]
    pub url_safe: bool,
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
    /// Break a URL into scheme, host, port, path, query and fragment.
    #[command(alias = "info")]
    Parse(UrlArgs),
}

#[derive(Debug, Args)]
pub struct UrlArgs {
    /// Input text. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Args)]
pub struct CaseArgs {
    /// Text to convert. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Target style: lower, upper, title, sentence, camel, pascal, snake,
    /// kebab, constant, dot or train.
    #[arg(short, long, default_value = "snake")]
    pub style: String,
}

#[derive(Debug, Args)]
pub struct NumberArgs {
    /// Number to convert. Prefixes such as 0x, 0b and 0o are auto-detected.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Source base (2-36). Defaults to auto-detect, then decimal.
    #[arg(long, value_parser = clap::value_parser!(u32).range(2..=36))]
    pub from: Option<u32>,

    /// Target base (2-36). Omit to show binary, octal, decimal, hex and base 36.
    #[arg(long, value_parser = clap::value_parser!(u32).range(2..=36))]
    pub to: Option<u32>,

    /// Print hexadecimal digits in uppercase.
    #[arg(long)]
    pub upper: bool,
}

#[derive(Debug, Args)]
pub struct BytesArgs {
    /// Byte count or size such as `1536`, `1.5 MB` or `2 GiB`.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Hide the SI (base 1000) representation.
    #[arg(long)]
    pub binary: bool,
}

#[derive(Debug, Args)]
pub struct ColorArgs {
    /// Hex, RGB, HSL, HSV or common CSS color name.
    #[arg(value_name = "COLOR")]
    pub input: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum TextCommand {
    /// Count characters, words, sentences, paragraphs and lines.
    Stats(TextInputArgs),
    /// Sort lines with optional de-duplication.
    Sort(TextSortArgs),
    /// Remove duplicate lines while keeping the first occurrence.
    #[command(alias = "deduplicate")]
    Dedupe(TextDedupeArgs),
    /// Convert text to a URL-friendly slug.
    #[command(alias = "slugify")]
    Slug(TextInputArgs),
}

#[derive(Debug, Args)]
pub struct TextInputArgs {
    /// Text input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Args)]
pub struct TextSortArgs {
    /// Text input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Sort in descending order.
    #[arg(short, long)]
    pub reverse: bool,

    /// Remove duplicate lines after sorting.
    #[arg(short, long)]
    pub unique: bool,

    /// Compare case-insensitively.
    #[arg(long)]
    pub ignore_case: bool,

    /// Trim leading and trailing whitespace from each line.
    #[arg(long)]
    pub trim: bool,
}

#[derive(Debug, Args)]
pub struct TextDedupeArgs {
    /// Text input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Treat lines that differ only by case as duplicates.
    #[arg(long)]
    pub ignore_case: bool,

    /// Trim lines before comparing them.
    #[arg(long)]
    pub trim: bool,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// Left text to compare, or `-` for stdin.
    #[arg(value_name = "LEFT")]
    pub left: Option<String>,

    /// Right text to compare, or `-` for stdin.
    #[arg(value_name = "RIGHT")]
    pub right: Option<String>,

    /// Number of context lines around each change (0-20).
    #[arg(short, long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(0..=20))]
    pub context: u8,

    /// Read the left side from a file.
    #[arg(long, value_name = "PATH", conflicts_with = "left")]
    pub left_file: Option<PathBuf>,

    /// Read the right side from a file.
    #[arg(long, value_name = "PATH", conflicts_with = "right")]
    pub right_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum HexCommand {
    /// Encode text as hexadecimal bytes.
    Encode(HexEncodeArgs),
    /// Decode hexadecimal bytes as UTF-8 text.
    Decode(EncodingInputArgs),
}

#[derive(Debug, Args)]
pub struct HexEncodeArgs {
    /// Text input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Print uppercase hex digits.
    #[arg(long)]
    pub upper: bool,
}

#[derive(Debug, Subcommand)]
pub enum BinaryCommand {
    /// Encode text as an 8-bit binary string.
    Encode(EncodingInputArgs),
    /// Decode an 8-bit binary string as UTF-8 text.
    Decode(EncodingInputArgs),
}

#[derive(Debug, Subcommand)]
pub enum HtmlCommand {
    /// Escape `&`, `<`, `>`, quotes and apostrophes as HTML entities.
    Encode(EncodingInputArgs),
    /// Decode named and numeric HTML entities.
    Decode(EncodingInputArgs),
}

#[derive(Debug, Args)]
pub struct EncodingInputArgs {
    /// Text input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum CsvCommand {
    /// Convert CSV rows into a JSON array.
    #[command(name = "to-json")]
    ToJson(CsvToJsonArgs),
    /// Convert a JSON array of objects or arrays into CSV.
    #[command(name = "from-json")]
    FromJson(CsvFromJsonArgs),
}

#[derive(Debug, Args)]
pub struct CsvToJsonArgs {
    /// CSV input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Field delimiter: one character, `tab` or `\t`.
    #[arg(short, long, default_value = ",")]
    pub delimiter: String,

    /// Treat the first row as data instead of column names.
    #[arg(long)]
    pub no_headers: bool,
}

#[derive(Debug, Args)]
pub struct CsvFromJsonArgs {
    /// JSON input. Omit or use `-` to read stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Field delimiter: one character, `tab` or `\t`.
    #[arg(short, long, default_value = ",")]
    pub delimiter: String,
}

#[derive(Debug, Args)]
pub struct HttpArgs {
    /// HTTP status code such as 200, 404 or 500.
    #[arg(value_name = "CODE")]
    pub code: Option<u16>,

    /// List every status code in the offline table.
    #[arg(long)]
    pub list: bool,

    /// List one status class (1-5).
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=5), conflicts_with = "list")]
    pub class: Option<u16>,
}

#[derive(Debug, Args)]
pub struct MimeArgs {
    /// File extension (`json`, `.png`) or MIME type (`application/json`).
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Args)]
pub struct IpArgs {
    /// IPv4 or IPv6 address, optionally with a CIDR prefix.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HmacAlgorithm {
    Sha256,
    Sha512,
}

#[derive(Debug, Args)]
pub struct HmacArgs {
    /// HMAC algorithm.
    #[arg(value_enum)]
    pub algorithm: HmacAlgorithm,

    /// Secret key.
    #[arg(value_name = "KEY")]
    pub key: String,

    /// Message to sign. Omit or use `-` to read stdin.
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

    /// Generate time-ordered version 7 UUIDs instead of version 4.
    #[arg(long)]
    pub v7: bool,

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
    #[arg(value_name = "INPUT", conflicts_with = "file")]
    pub input: Option<String>,

    /// Hash a file instead of text.
    #[arg(long, value_name = "PATH")]
    pub file: Option<PathBuf>,
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

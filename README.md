# DevToys CLI

[![CI](https://github.com/PloxiQ/DevToys-CLI/actions/workflows/ci.yml/badge.svg)](https://github.com/PloxiQ/DevToys-CLI/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/PloxiQ/DevToys-CLI?style=social)](https://github.com/PloxiQ/DevToys-CLI)

**A fast, offline developer toolbox for the terminal.** DevToys CLI packs 20 everyday utilities into one small Rust binary: a keyboard-first TUI when you want to explore, and a script-friendly CLI when you need to get things done.

No accounts. No telemetry. No network calls. Just tools.

```
  DEVTOYS CLI  offline developer toolbox                    20 tools · zero network calls · v0.1.0
  ───────────────────────────────────────────────────────────────────────────────────────────────
┌ Tools 20─────────────────────┐  JSON Format  FORMAT & CONVERT
│ FORMAT & CONVERT             │  Pretty-print JSON and preserve key order.
│ › JSON Format                │┌ JSON input ────────────────────────────────────────────────────┐
│   JSON Minify                ││{"name":"DevToys CLI","stars":1337,"offline":true,"tools":[...]}│
│   Base64 Encode              ││                                                                │
│   Base64 Decode              ││                                                                │
│   Base64 Encode HEX          ││                                                                │
│   Base64 Decode HEX          │└────────────────────────────────────────────────────────────────┘
│   URL Encode                 │┌ Output  OK   155 µs ──────────────────────────────────────────┐
│   URL Decode                 ││{                                                               │
│   Timestamp to Date          ││  "name": "DevToys CLI",                                        │
│   Date to Timestamp          ││  "stars": 1337,                                                │
│   Cron Explainer             ││  "offline": true,                                              │
│                              ││  "tools": [                                                    │
│ GENERATE                     ││    "json",                                                     │
│   UUID v4                    ││    "jwt",                                                      │
│   Hash MD5                   ││    "regex"                                                     │
│   Hash SHA-256               ││  ]                                                             │
│   Hash SHA-512               ││}                                                               │
│   Lorem Ipsum                ││                                                                │
│   Random String              │└────────────────────────────────────────────────────────────────┘
│   Password                   │
│                              │
│ INSPECT                      │
│   JWT Decoder                │
│   Regex Tester               │
└──────────────────────────────┘
  Up/Down navigate   Enter edit   ? help   q quit               20 tools · offline by design · MIT · v0.1.0
```

Run `devtoys` (or `devtoys tui`) with no arguments to open the TUI.

## Why DevToys CLI?

- **Two interfaces, one toolbox** — launch the TUI or use subcommands for pipelines.
- **Offline by design** — every conversion, hash and generator runs locally.
- **Pipe-friendly** — omit an input argument (or pass `-`) to read from stdin.
- **Friendly failures** — every error includes a hint that tells you how to fix it.
- **Safe defaults** — no `unsafe`, no `unwrap`/`expect` in project code, and OS entropy for UUIDs, random strings and passwords.
- **Small and fast** — size-optimized release profile with LTO, `opt-level = "z"` and symbol stripping.

## Installation

### Build from source

Requires [Rust](https://rustup.rs) 1.89 or newer.

```bash
git clone https://github.com/PloxiQ/DevToys-CLI.git
cd DevToys-CLI
cargo build --release
```

The binary is written to `target/release/devtoys` (`target\release\devtoys.exe` on Windows).

### Install with Cargo

```bash
# TUI + CLI (default)
cargo install --path .

# Or from GitHub
cargo install --git https://github.com/PloxiQ/DevToys-CLI

# CLI only, without ratatui/crossterm
cargo install --path . --no-default-features
```

### Uninstall

```bash
cargo uninstall devtoys-cli
```

## Quick start

```bash
# Open the interactive TUI
devtoys

# Format JSON with syntax highlighting when stdout is a terminal
devtoys json format '{"name":"DevToys CLI","stars":1337,"offline":true}'

# Read from stdin
echo '{"a":1,"b":[true,null]}' | devtoys json format

# Decode a JWT (signature is never verified)
devtoys jwt decode "$TOKEN" --part payload

# Explain a cron expression
devtoys cron '*/15 * * * *' --next 3

# Hash a value
devtoys hash sha256 'hello world'

# Generate 5 UUIDs
devtoys uuid --count 5
```

The CLI's JSON highlighter only emits ANSI colors when stdout is an interactive terminal. Piped output is always plain, and colors can be disabled explicitly with `--no-color` or `NO_COLOR=1`.

## TUI controls

| Key | Action |
| --- | --- |
| `Up` / `Down` or `k` / `j` | Move through the tool list |
| `Enter` | Start editing the input, or run the current tool |
| `Tab` | Switch between input fields |
| `Shift+Enter` / `Ctrl+J` | Insert a newline |
| `Ctrl+U` | Clear the active field |
| `Ctrl+W` | Delete the previous word |
| `PgUp` / `PgDn` | Scroll the output panel |
| `Esc` | Return to the tool list |
| `?` / `F1` | Open or close help |
| `Ctrl+C` | Quit |

Most tools run automatically as you type. Generators (UUID, Lorem Ipsum, Random String, Password) run when you press `Enter`.

## Tool reference

| Tool | TUI entry | CLI command |
| --- | --- | --- |
| JSON formatter | JSON Format | `devtoys json format [INPUT] [--indent N]` |
| JSON minifier | JSON Minify | `devtoys json minify [INPUT]` |
| Base64 encode | Base64 Encode | `devtoys base64 encode [INPUT]` |
| Base64 decode | Base64 Decode | `devtoys base64 decode [INPUT]` |
| Base64 encode (HEX) | Base64 Encode HEX | `devtoys base64 encode --hex [INPUT]` |
| Base64 decode (HEX) | Base64 Decode HEX | `devtoys base64 decode --hex [INPUT]` |
| URL encode | URL Encode | `devtoys url encode [INPUT]` |
| URL decode | URL Decode | `devtoys url decode [INPUT]` |
| Unix timestamp → date | Timestamp to Date | `devtoys timestamp to-date [TIMESTAMP] [--ms] [--local] [--verbose]` |
| Date → Unix timestamp | Date to Timestamp | `devtoys timestamp to-epoch [DATE] [--utc] [--ms] [--verbose]` |
| Current timestamp | — | `devtoys timestamp now [--ms]` |
| Cron explainer | Cron Explainer | `devtoys cron [EXPRESSION] [--next N]` |
| UUID v4 | UUID v4 | `devtoys uuid [--count N] [--upper] [--no-hyphens]` |
| MD5 hash | Hash MD5 | `devtoys hash md5 [INPUT]` |
| SHA-256 hash | Hash SHA-256 | `devtoys hash sha256 [INPUT]` |
| SHA-512 hash | Hash SHA-512 | `devtoys hash sha512 [INPUT]` |
| Lorem Ipsum | Lorem Ipsum | `devtoys lorem [-p N] [-s N] [-w N]` |
| Random string | Random String | `devtoys random [LENGTH] [--charset SET] [-c N]` |
| Password generator | Password | `devtoys password [-l N] [-c N] [--no-lower] [--no-upper] [--no-digits] [--no-symbols]` |
| JWT decoder | JWT Decoder | `devtoys jwt decode [TOKEN] [--part PART]` |
| Regex tester | Regex Tester | `devtoys regex test PATTERN [TEXT] [-i] [-m] [-s] [-r REPLACEMENT]` |

`--part` accepts `both` (default), `header` or `payload`.

Run `devtoys --help` or `devtoys <command> --help` for the complete flag list. Aliases such as `b64`, `pw`, `re`, `json fmt`, `timestamp date` and `timestamp epoch` are also available.

## Examples

### JSON

```bash
devtoys json format '{"z":1,"a":[1,2,3]}'
devtoys json format --indent 4 '{"nested":{"ok":true}}'
devtoys json minify '{
  "name": "DevToys CLI",
  "offline": true
}'
```

The formatter preserves key order, validates input before printing anything, and highlights strings, numbers, booleans and null values when writing to a terminal.

### Base64 and URL

```bash
devtoys base64 encode 'Hello, DevToys!'
devtoys base64 decode 'SGVsbG8sIERldlRveXMh'
devtoys base64 encode --hex 48656c6c6f
devtoys base64 decode --hex SGVsbG8=

devtoys url encode 'https://example.com/?q=rust cli'
devtoys url decode 'https%3A%2F%2Fexample.com%2F%3Fq%3Drust+cli'
```

URL encoding uses `%20` for spaces; decoding treats `+` as a space, matching form semantics.

### Timestamps and cron

```bash
devtoys timestamp now
devtoys timestamp now --ms
devtoys timestamp to-date 1700000000
devtoys timestamp to-date 1700000000000 --verbose
devtoys timestamp to-date 1700000000 --local
devtoys timestamp to-epoch '2024-01-01T00:00:00Z'
devtoys timestamp to-epoch '2024-01-01 12:00:00' --utc --verbose

# Macros, ranges, lists, steps, names and Vixie day-field OR semantics
devtoys cron '*/15 * * * *' --next 5
devtoys cron '0 9 * * MON-FRI'
devtoys cron '0 0 1 * *'
```

Date parsing accepts RFC 3339, common date/time layouts and `now`. Timestamps are auto-detected as seconds or milliseconds, with `--ms` available to force milliseconds.

### Generators

```bash
devtoys uuid
devtoys uuid --count 10 --upper
devtoys uuid --no-hyphens

devtoys lorem
devtoys lorem --paragraphs 3 --sentences 5 --words 12

devtoys random 32
devtoys random 16 --charset hex --count 4

devtoys password
devtoys password --length 32 --no-symbols
```

`--charset` accepts `alnum`, `alpha`, `lower`, `upper`, `numeric`, `hex`, `base58` and `symbols`. Passwords always include at least one character from every enabled class and are shuffled with OS entropy.

### JWT

```bash
devtoys jwt decode "$TOKEN"
devtoys jwt decode "$TOKEN" --part header
devtoys jwt decode "$TOKEN" --part payload

# From stdin
cat token.txt | devtoys jwt decode --part payload -
```

> [!WARNING]
> The JWT decoder only Base64URL-decodes the header and payload for inspection. It does **not** verify the signature, issuer, audience or expiry. Never use it to make trust decisions.

### Regex

```bash
devtoys regex test '\b\w+@\w+\.\w+\b' 'Contact dev@example.com or support@devtoys.dev'
devtoys regex test -i 'error' 'Error: something went wrong'
devtoys regex test -m '^>.*' $'> quote one\n> quote two'
devtoys regex test '\d+' 'order 123, total 456' --replace '<number>'
```

Uses Rust's linear-time [`regex`](https://docs.rs/regex) crate. Patterns are not vulnerable to catastrophic backtracking.

### Stdin recipes

Every input-taking command accepts data from stdin. Pass `-` explicitly when you want to make that intent clear.

```bash
cat Cargo.toml | devtoys hash sha256
curl -s https://example.com/data.json | devtoys json format
pbpaste | devtoys url encode
printf 'hello' | devtoys base64 encode
```

> On Windows PowerShell, quoting JSON as a native argument can drop embedded double quotes. Pipe it instead: `'{"a":1}' | devtoys json format` or use `Get-Content file.json | devtoys json format`.

## Security notes

- **MD5 is provided for legacy checksums only.** Use SHA-256 or SHA-512 for anything security-related.
- **Randomness uses the operating system CSPRNG** (`OsRng`) for UUIDs, random strings and passwords.
- **Nothing leaves your machine.** There is no networking code, analytics or update check.
- **JWT signatures are not verified.** See the warning above.

## Project structure

```
DevToys-CLI/
├── src/
│   ├── main.rs          # CLI dispatch, stdin handling, output and error reporting
│   ├── cli.rs           # clap command tree and argument validation
│   ├── lib.rs           # library entry point
│   ├── error.rs         # ToolError with optional hints
│   ├── style.rs         # ANSI helpers (respects NO_COLOR)
│   ├── tools/           # pure, reusable tool implementations
│   │   ├── json.rs      # validating formatter/minifier + syntax highlighter
│   │   ├── base64.rs    # text/HEX Base64
│   │   ├── url.rs       # percent encoding/decoding
│   │   ├── timestamp.rs # Unix time conversions
│   │   ├── cron.rs      # cron parser, describer and next-run calculator
│   │   ├── uuid.rs      # UUID v4 generation
│   │   ├── hash.rs      # MD5 / SHA-256 / SHA-512
│   │   ├── lorem.rs     # Lorem Ipsum generator
│   │   ├── random.rs    # random strings and passwords
│   │   ├── jwt.rs       # JWT decoder
│   │   ├── regex.rs     # regex tester
│   │   └── rng.rs       # fallible OS entropy wrapper
│   └── tui/             # ratatui + crossterm interface
│       ├── mod.rs       # terminal lifecycle and event loop
│       ├── app.rs       # state, key handling and tool execution
│       └── ui.rs        # layout, sidebar, editor and output rendering
├── tests/
│   └── cli.rs           # end-to-end CLI tests
├── Cargo.toml
└── README.md
```

The `tui` Cargo feature is enabled by default. Build with `--no-default-features` for a smaller CLI-only binary; running `devtoys` without a command then prints a helpful error instead of opening the TUI.

## How to contribute a new tool

Contributions are welcome! Adding a tool takes about six small steps.

### 1. Implement the tool

Create `src/tools/my_tool.rs` with a pure function that returns `crate::error::Result`. Use `ToolError::with_hint` for actionable failures, and never call `unwrap` or `expect`.

```rust
use crate::error::{Result, ToolError};

pub fn reverse(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err(ToolError::new("Input is empty.").with_hint("Pass at least one character."));
    }
    Ok(input.chars().rev().collect())
}
```

Register the module in `src/tools/mod.rs`:

```rust
pub mod my_tool;
```

### 2. Add CLI arguments

In `src/cli.rs`:

1. Add a variant to `Command` (and a subcommand enum if the tool has multiple actions).
2. Define an `Args` struct with clap attributes, value ranges and helpful doc comments.
3. Add an example to `AFTER_HELP`.

Then dispatch it in `src/main.rs` inside `run()`, and use `input_or_stdin` for text inputs so pipelines work.

### 3. Add the TUI entry

In `src/tui/app.rs`:

1. Add a `ToolId` variant.
2. Add a `ToolDef` to `TOOLS` with a name, group, description, sample input and `auto_run` setting.
3. Handle the new `ToolId` in `run_tool`.
4. If the tool needs two fields, give it a `secondary` field and handle `Focus::Secondary`.

The sidebar, keyboard navigation, help overlay and rendering are generated from `TOOLS`, so no UI changes are usually needed.

### 4. Test it

Add unit tests next to the implementation and an end-to-end case in `tests/cli.rs`:

```rust
let output = run(&["my-tool", "hello"])?;
assert_success(&output);
assert_eq!(text(&output.stdout)?.trim(), "olleh");
```

Cover the happy path, invalid input, ranges and stdin behavior.

### 5. Document it

Add the tool to the table and examples in this README, and mention any safety caveats.

### 6. Run the checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
```

Open a pull request with a short description of the tool, the flags you added and the tests you ran. Screenshots of new TUI screens are appreciated.

## License

MIT © [PloxiQ](https://github.com/PloxiQ). See [LICENSE](LICENSE) for details.

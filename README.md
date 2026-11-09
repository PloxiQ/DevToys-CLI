# DevToys CLI

<p align="center">
  <img src="docs/assets/banner.png" alt="DevToys CLI — 51 offline developer tools in one tiny Rust binary" width="100%">
</p>

<p align="center">
  <a href="https://github.com/PloxiQ/DevToys-CLI/actions/workflows/ci.yml"><img src="https://github.com/PloxiQ/DevToys-CLI/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/PloxiQ/DevToys-CLI/releases/latest"><img src="https://img.shields.io/github/v/release/PloxiQ/DevToys-CLI?sort=semver" alt="Latest release"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.89%2B-orange?logo=rust" alt="Rust 1.89+"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT license"></a>
  <a href="https://github.com/PloxiQ/DevToys-CLI/stargazers"><img src="https://img.shields.io/github/stars/PloxiQ/DevToys-CLI?style=social" alt="GitHub stars"></a>
</p>

<p align="center">
  <strong>51 everyday developer tools. One tiny Rust binary.</strong><br>
  A keyboard-first TUI for exploring and a pipe-friendly CLI for scripting.<br>
  <sub>No accounts. No telemetry. No network calls. Just tools.</sub>
</p>

<p align="center">
  <a href="#see-it-in-action">See it in action</a> ·
  <a href="#installation">Install</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#tool-reference">Tools</a> ·
  <a href="#how-to-contribute-a-new-tool">Contribute</a>
</p>

## See it in action

<p align="center">
  <img src="docs/assets/tui.png" alt="DevToys CLI TUI showing the tool sidebar, JSON input and formatted JSON output" width="100%">
</p>
<p align="center"><sub><strong>TUI</strong> — browse all 51 tools, edit input, and see results instantly. Press <code>?</code> for keyboard shortcuts.</sub></p>

<p align="center">
  <img src="docs/assets/cli.png" alt="DevToys CLI terminal session formatting JSON, explaining a cron schedule and testing a regex" width="860">
</p>
<p align="center"><sub><strong>CLI</strong> — pipe-friendly commands for scripts, CI and quick one-liners.</sub></p>

<details>
<summary>Plain-text TUI preview</summary>

```text
  DEVTOYS CLI  offline developer toolbox                    51 tools · zero network calls · v0.2.1
  ───────────────────────────────────────────────────────────────────────────────────────────────
┌ Tools 51─────────────────────┐  JSON Format  FORMAT & CONVERT
│ FORMAT & CONVERT             │  Pretty-print JSON and preserve key order.
│ › JSON Format                │┌ JSON input ────────────────────────────────────────────────────┐
│   JSON Minify                ││{"name":"DevToys CLI","stars":1337,"offline":true,"tools":[...]}│
│   JSON Query                 ││                                                                │
│   CSV to JSON                ││                                                                │
│   JSON to CSV                │└────────────────────────────────────────────────────────────────┘
│   Number Base                │┌ Output  OK   155 µs ──────────────────────────────────────────┐
│   Byte Size                  ││{                                                               │
│   Color Converter            ││  "name": "DevToys CLI",                                        │
│   Base64 Encode              ││  "stars": 1337,                                                │
│   Base64 Decode              ││  "offline": true,                                              │
│   ...                        ││  "tools": [                                                    │
│                              ││    "json",                                                     │
│ TEXT                         ││    "jwt",                                                      │
│   Case Converter             ││    "regex"                                                     │
│   Text Diff                  ││  ]                                                             │
│   Text Statistics            ││}                                                               │
│   Line Sorter                ││                                                                │
│   Line Deduplicator          │└────────────────────────────────────────────────────────────────┘
│   Slugify                    │
│                              │
│ DATE & TIME                  │
│   Timestamp to Date          │
│   Date to Timestamp          │
│   Cron Explainer             │
└──────────────────────────────┘
  Up/Down navigate   Enter edit   ? help   q quit               51 tools · offline by design · MIT · v0.2.1
```

</details>

Run `devtoys` (or `devtoys tui`) with no arguments to open the TUI.

## Why DevToys CLI?

| | |
| --- | --- |
| **Two interfaces, one toolbox**<br>Launch the TUI to explore, or use subcommands in scripts and pipelines. | **Offline by design**<br>Every conversion, hash and generator runs locally. No accounts, analytics or update checks. |
| **Pipe-friendly**<br>Omit an input argument or pass `-` to read from stdin. Perfect for `curl`, `cat` and CI. | **Friendly failures**<br>Every error includes a hint that tells you how to fix it. |
| **Safe defaults**<br>No `unsafe`, no `unwrap`/`expect` in project code, and OS entropy for every secret. | **Small and fast**<br>Size-optimized Rust release profile with LTO, `opt-level = "z"` and stripped symbols. |

> **Note:** DevToys CLI is an independent terminal project inspired by the offline-toolbox idea. It is not affiliated with the DevToys GUI project.

## Installation

### Prebuilt binaries

Download the latest archive from [GitHub Releases](https://github.com/PloxiQ/DevToys-CLI/releases/latest):

| Platform | Asset |
| --- | --- |
| Windows x86_64 | `devtoys-vX.Y.Z-x86_64-pc-windows-msvc.zip` |
| Linux x86_64 | `devtoys-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Apple Silicon | `devtoys-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `devtoys-vX.Y.Z-x86_64-apple-darwin.tar.gz` |

Each archive contains the `devtoys` binary, `README.md` and `LICENSE`. Verify downloads with the `SHA256SUMS` file attached to every release.

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

# Convert text, numbers and colors
devtoys case 'Hello world' --style kebab
devtoys number 255 --to 16
devtoys color '#58a6ff'

# Query JSON and convert CSV
devtoys json get user.name '{"user":{"name":"Ada"}}'
devtoys csv to-json 'name,age
Ada,30'
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

Most tools run automatically as you type. UUID v4, UUID v7, File Hash, Lorem Ipsum, Random String, Password, Three-Way Merge, QR Code and JSON Schema run when you press `Enter` after editing. In HTTP Headers, use `Tab` to select Redact, then `Space` or the arrow keys to toggle it. QR Code uses the same controls to select its ECC level. Three-Way Merge has three path fields; `Tab` cycles through Base, Ours and Theirs.

## Tool reference

The TUI and CLI share the same implementations. Every command that takes text accepts stdin (omit the argument or pass `-`). File-based commands use explicit path flags; `diff3` accepts `-` for at most one of its three paths. The 51 TUI entries include separate Base32 encode and decode tools.

### Format & Convert

| Tool | TUI entry | CLI command |
| --- | --- | --- |
| JSON formatter | JSON Format | `devtoys json format [INPUT] [--indent N]` |
| JSON minifier | JSON Minify | `devtoys json minify [INPUT]` |
| JSON query | JSON Query | `devtoys json get PATH [INPUT] [--raw]` |
| CSV → JSON | CSV to JSON | `devtoys csv to-json [INPUT] [-d DELIM] [--no-headers]` |
| JSON → CSV | JSON to CSV | `devtoys csv from-json [INPUT] [-d DELIM]` |
| Number base | Number Base | `devtoys number [INPUT] [--from 2-36] [--to 2-36] [--upper]` |
| Byte size | Byte Size | `devtoys bytes [INPUT] [--binary]` |
| Color converter | Color Converter | `devtoys color [COLOR]` |
| Base32 encode | Base32 Encode | `devtoys base32 encode [INPUT] [--no-padding] [--lower]` |
| Base32 decode | Base32 Decode | `devtoys base32 decode [INPUT]` |
| Base64 encode | Base64 Encode | `devtoys base64 encode [INPUT] [--url-safe]` |
| Base64 decode | Base64 Decode | `devtoys base64 decode [INPUT]` |
| Base64 encode (HEX) | Base64 Encode HEX | `devtoys base64 encode --hex [INPUT] [--url-safe]` |
| Base64 decode (HEX) | Base64 Decode HEX | `devtoys base64 decode --hex [INPUT]` |
| Hex encode | Hex Encode | `devtoys hex encode [INPUT] [--upper]` |
| Hex decode | Hex Decode | `devtoys hex decode [INPUT]` |
| Binary encode | Binary Encode | `devtoys binary encode [INPUT]` |
| Binary decode | Binary Decode | `devtoys binary decode [INPUT]` |
| HTML encode | HTML Encode | `devtoys html encode [INPUT]` |
| HTML decode | HTML Decode | `devtoys html decode [INPUT]` |
| URL encode | URL Encode | `devtoys url encode [INPUT]` |
| URL decode | URL Decode | `devtoys url decode [INPUT]` |

### Text

| Tool | TUI entry | CLI command |
| --- | --- | --- |
| Case converter | Case Converter | `devtoys case [INPUT] [-s STYLE]` |
| Three-way file merge | Three-Way Merge | `devtoys diff3 --base PATH --ours PATH --theirs PATH [--label-base LABEL] [--label-ours LABEL] [--label-theirs LABEL]` |
| Text diff | Text Diff | `devtoys diff [LEFT] [RIGHT] [-c N] [--left-file PATH] [--right-file PATH]` |
| Text statistics | Text Statistics | `devtoys text stats [INPUT]` |
| Line sorter | Line Sorter | `devtoys text sort [INPUT] [-r] [-u] [--ignore-case] [--trim]` |
| Line deduplicator | Line Deduplicator | `devtoys text dedupe [INPUT] [--ignore-case] [--trim]` |
| Slugify | Slugify | `devtoys text slug [INPUT]` |

### Date & Time

| Tool | TUI entry | CLI command |
| --- | --- | --- |
| Unix timestamp → date | Timestamp to Date | `devtoys timestamp to-date [TIMESTAMP] [--ms] [--local] [--verbose]` |
| Date → Unix timestamp | Date to Timestamp | `devtoys timestamp to-epoch [DATE] [--utc] [--ms] [--verbose]` |
| Cron explainer | Cron Explainer | `devtoys cron [EXPRESSION] [--next N]` |
| Current timestamp | — | `devtoys timestamp now [--ms]` |

### Generate

| Tool | TUI entry | CLI command |
| --- | --- | --- |
| UUID v4 | UUID v4 | `devtoys uuid [--count N] [--upper] [--no-hyphens]` |
| UUID v7 | UUID v7 | `devtoys uuid --v7 [--count N] [--upper] [--no-hyphens]` |
| MD5 hash | Hash MD5 | `devtoys hash md5 [INPUT]` or `devtoys hash md5 --file PATH` |
| SHA-256 hash | Hash SHA-256 | `devtoys hash sha256 [INPUT]` or `devtoys hash sha256 --file PATH` |
| SHA-512 hash | Hash SHA-512 | `devtoys hash sha512 [INPUT]` or `devtoys hash sha512 --file PATH` |
| HMAC | HMAC | `devtoys hmac sha256 KEY [INPUT]` or `devtoys hmac sha512 KEY [INPUT]` |
| File hash | File Hash | `devtoys hash sha256 --file PATH` |
| Lorem Ipsum | Lorem Ipsum | `devtoys lorem [-p N] [-s N] [-w N]` |
| Random string | Random String | `devtoys random [LENGTH] [--charset SET] [-c N]` |
| Password generator | Password | `devtoys password [-l N] [-c N] [--no-lower] [--no-upper] [--no-digits] [--no-symbols]` |

### Inspect

| Tool | TUI entry | CLI command |
| --- | --- | --- |
| JWT decoder | JWT Decoder | `devtoys jwt decode [TOKEN] [--part PART]` |
| HTTP header parser | HTTP Headers | `devtoys headers [INPUT] [--redact]` |
| JSON Schema validator | JSON Schema | `devtoys jsonschema --schema PATH [JSON]` |
| QR generator | QR Code | `devtoys qr [INPUT] [--ecc LEVEL] [--invert] [--svg] [--output PATH]` |
| Regex tester | Regex Tester | `devtoys regex test PATTERN [TEXT] [-i] [-m] [-s] [-r REPLACEMENT]` |
| URL parser | URL Parser | `devtoys url parse [URL]` |
| HTTP status | HTTP Status | `devtoys http CODE`, `devtoys http --class N` or `devtoys http --list` |
| MIME lookup | MIME Lookup | `devtoys mime EXTENSION_OR_MIME` |
| IP / CIDR | IP / CIDR | `devtoys ip ADDRESS[/PREFIX]` |

`--part` accepts `both` (default), `header` or `payload`. `--style` accepts `lower`, `upper`, `title`, `sentence`, `camel`, `pascal`, `snake`, `kebab`, `constant`, `dot` and `train`.

Run `devtoys --help` or `devtoys <command> --help` for the complete flag list. Aliases such as `b64`, `pw`, `re`, `size`, `json fmt`, `json query`, `timestamp date` and `timestamp epoch` are also available.

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

```bash
devtoys json get user.name '{"user":{"name":"Ada"}}' --raw
devtoys json get '/tools/1' '{"tools":["json","jwt"]}'
```

### Case, numbers and colors

```bash
devtoys case 'Hello, DevToys CLI!' --style kebab
devtoys case 'Hello, DevToys CLI!' --style constant

devtoys number 255
devtoys number 255 --to 2
devtoys number 0b1010 --to 10
devtoys number ff --from 16 --to 8 --upper

devtoys bytes 1536
devtoys bytes '1.5 MiB'
devtoys bytes '2 GB' --binary

devtoys color '#58a6ff'
devtoys color 'rgb(88, 166, 255)'
devtoys color 'hsl(212, 100%, 67%)'
```

The color converter reports hex, RGB/RGBA, HSL, HSV, relative luminance and WCAG contrast against black and white.

### Text utilities

```bash
cat README.md | devtoys text stats
sort names.txt | devtoys text sort --reverse
sort names.txt | devtoys text sort --unique --ignore-case
cat emails.txt | devtoys text dedupe --ignore-case --trim
devtoys text slug 'Crème brûlée à la DevToys'
devtoys diff --left-file old.txt --right-file new.txt --context 3
git show HEAD~1:README.md > old.md && devtoys diff --left-file old.md --right-file README.md
```

Text statistics count characters, words, sentences, paragraphs, lines, bytes, longest line and estimated reading time. The diff output is unified format and compares line by line.

### Three-way merge

```bash
printf 'one\ntwo\nthree\n' > base.txt
printf 'ONE\ntwo\nthree\n' > ours.txt
printf 'one\ntwo\nTHREE\n' > theirs.txt
devtoys diff3 --base base.txt --ours ours.txt --theirs theirs.txt

cat base.txt | devtoys diff3 --base - --ours ours.txt --theirs theirs.txt
devtoys diff3 --base base.txt --ours ours.txt --theirs theirs.txt \
  --label-base ancestor --label-ours local --label-theirs remote
```

`diff3` reads UTF-8 files and merges changes relative to the common ancestor. Non-overlapping changes merge cleanly with exit code 0. Overlapping changes print `<<<<<<<`, `|||||||`, `=======` and `>>>>>>>` conflict markers and exit 1. Stdout contains only the merged text, so it can be redirected to a file; clean merges preserve CRLF and missing final newlines without appending a status message or newline. Conflict markers start on separate lines even when an input lacks its final newline.

At most one of `--base`, `--ours` and `--theirs` may be `-` for stdin. Each input is limited to 256 KiB and 10,000 lines, and the merge result is limited to 8 MiB. Labels default to `base`, `ours` and `theirs`; custom labels accept up to 128 UTF-8 bytes without control characters. The TUI accepts three file paths and displays the merge or conflict in its output pane.

### Encoding

```bash
devtoys hex encode 'Hello, DevToys!'
devtoys hex decode 48656c6c6f2c20446576546f797321
devtoys hex encode --upper 'Hi'

devtoys binary encode 'Hi'
devtoys binary decode '01001000 01101001'

devtoys html encode '<a href="x">Tom & Jerry</a>'
devtoys html decode '&lt;b&gt;Hi&lt;/b&gt;'

devtoys base64 encode --url-safe 'Hello'
```

Hex input accepts whitespace, `:`, `-`, `_` and an optional `0x` prefix. HTML decoding supports common named entities plus decimal and hexadecimal numeric entities.

### Base32

```bash
devtoys base32 encode 'foobar'
devtoys base32 encode 'foobar' --no-padding --lower
devtoys base32 decode 'MZXW6YTBOI======'
printf 'mzxw6ytboi' | devtoys base32 decode -
```

Base32 uses the RFC 4648 alphabet (`A-Z`, `2-7`), with uppercase letters and padding by default. Decoding accepts mixed case, ASCII whitespace and either complete padding or none. Invalid characters, incomplete padding and non-zero trailing bits produce an error with a hint. Both directions operate on UTF-8 text and accept at most 1 MiB of input, including whitespace and padding when decoding.

### QR codes

```bash
devtoys qr 'Hello, DevToys!'
devtoys qr 'Привіт 🌍' --ecc H
devtoys qr 'hello' --invert
devtoys qr 'hello' --svg
devtoys qr 'hello' --ecc Q --svg --output code.svg
printf 'hello' | devtoys qr - --output code.txt
```

Terminal QR output uses Unicode half blocks with a four-module quiet zone on every side. ECC accepts `L`, `M` (default), `Q` or `H`; stronger error correction leaves less space for data. Empty input and data that exceeds the selected ECC capacity are rejected. Inputs are also capped at 7,089 UTF-8 bytes. `--invert` swaps dark and light modules. `--svg` emits SVG, and `--output` writes the selected format to a local file instead of stdout.

Terminal scanning depends on the font, foreground/background colors and available space. Choose inversion to suit the terminal theme, or use SVG for predictable colors and scaling. The TUI renders QR output with black modules on white and asks for a larger terminal if the full code will not fit. It supports text and ECC selection; SVG export, inversion and file output are CLI options. No PNG/image encoder is included.

### HTTP headers

```bash
devtoys headers 'GET / HTTP/1.1
host: example.com
accept: application/json'

printf 'HTTP/1.1 200 OK\r\nset-cookie: a=1\r\nset-cookie: b=2\r\n' | devtoys headers - --redact
devtoys headers 'Authorization: Bearer example' --redact
```

The parser accepts an optional request or status line followed by `Name: value` lines, using LF or CRLF. It preserves duplicate headers and their order, canonicalizes names to Title-Case and trims spaces/tabs around values. Output uses LF. `--redact` replaces Authorization, Proxy-Authorization, Cookie, Set-Cookie and X-Api-Key values with `[redacted]`, including duplicates.

Input is limited to 1 MiB. Empty input, malformed names/lines, obsolete folded continuation lines and body content after the header block produce errors with hints; malformed-line errors include the line number. This tool inspects pasted text locally and never sends a request.

### JSON Schema validation

```bash
printf '%s\n' '{"type":"object","required":["name"],"properties":{"name":{"type":"string"}}}' > schema.json
devtoys jsonschema --schema schema.json '{"name":"Ada"}'
printf '%s' '{"name":42}' | devtoys jsonschema --schema schema.json -
```

A valid instance prints `Valid.` and exits 0. Validation failures print a list of errors with JSON Pointer instance paths and exit 1. Paths are JSON-quoted, so `""` identifies the root and `"/items/0"` identifies the first array item. Schema and instance numbers retain arbitrary precision. Internal references such as `#/$defs/person` work; external resources, including HTTP and file references, are disabled. Invalid schemas and unreadable schema files produce friendly errors with hints.

Schema and instance input are each limited to 1 MiB and the JSON parser's nesting limit. Output lists at most 100 validation errors, followed by an omission notice when more remain. The TUI takes pasted schema JSON and an instance in separate fields; the CLI reads the schema from `--schema PATH`. Validation and reference resolution run entirely offline.

### Base64 and URL

```bash
devtoys base64 encode 'Hello, DevToys!'
devtoys base64 decode 'SGVsbG8sIERldlRveXMh'
devtoys base64 encode --hex 48656c6c6f
devtoys base64 decode --hex SGVsbG8=

devtoys url encode 'https://example.com/?q=rust cli'
devtoys url decode 'https%3A%2F%2Fexample.com%2F%3Fq%3Drust+cli'
devtoys url parse 'https://user:pass@example.com:8443/path?a=1&b=two#top'
```

URL encoding uses `%20` for spaces; decoding treats `+` as a space, matching form semantics. The parser prints scheme, host, port, credentials, path, query parameters, fragment and origin.

### CSV and JSON

```bash
devtoys csv to-json 'name,age
Ada,30
Grace,25'

devtoys csv to-json --delimiter ';' --no-headers 'Ada;30'
devtoys csv from-json '[{"name":"Ada","age":30}]'
devtoys csv from-json --delimiter tab '[{"name":"Ada","age":30}]'
```

CSV to JSON keeps every cell as a string so leading zeros and large identifiers are not silently converted. JSON to CSV accepts an array of objects (columns are the union of keys, in first-seen order) or an array of arrays. Nested objects and arrays are written as compact JSON cells.

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
devtoys uuid --v7 --count 5

devtoys lorem
devtoys lorem --paragraphs 3 --sentences 5 --words 12

devtoys random 32
devtoys random 16 --charset hex --count 4

devtoys password
devtoys password --length 32 --no-symbols
```

`--charset` accepts `alnum`, `alpha`, `lower`, `upper`, `numeric`, `hex`, `base58` and `symbols`. Passwords always include at least one character from every enabled class and are shuffled with OS entropy. UUID v7 values are time-ordered and use OS entropy for the random bits.

### Hashing and signatures

```bash
devtoys hash md5 --file legacy.iso
devtoys hash sha256 'hello world'
devtoys hash sha256 --file Cargo.toml
cat archive.tar.gz | devtoys hash sha512

devtoys hmac sha256 'secret' 'The quick brown fox jumps over the lazy dog'
printf 'message' | devtoys hmac sha512 'secret'
```

`--file` hashes raw file bytes instead of UTF-8 text, so it works for archives and other binary files. HMAC supports SHA-256 and SHA-512 and prints a lowercase hex tag.

### Web and network helpers

```bash
devtoys url parse 'https://github.com/PloxiQ/DevToys-CLI?tab=readme#install'
devtoys http 404
devtoys http --class 5
devtoys http --list
devtoys mime application/json
devtoys mime photo.png
devtoys ip 192.168.1.42/24
devtoys ip 2001:db8::1/64
```

The HTTP, MIME and IP tables are compiled into the binary. No DNS lookups or HTTP requests are made.

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

## Roadmap

- [x] 51 offline tools with both TUI and CLI
- [x] Converters, text utilities, encodings, CSV/JSON, web/network and security helpers
- [x] Automated Windows, Linux and macOS releases
- [ ] Publish `devtoys-cli` to crates.io
- [ ] Scoop, Winget and Homebrew packages
- [x] Base32, three-way merge, JSON Schema validation, HTTP header parsing and QR codes

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
│   │   ├── json.rs      # formatter, minifier, pointer/dot-path query
│   │   ├── csv.rs       # CSV ↔ JSON conversion
│   │   ├── case.rs      # naming-convention conversion
│   │   ├── text.rs      # statistics, sorting, dedupe, slugify
│   │   ├── diff.rs      # unified line diff
│   │   ├── diff3.rs     # bounded three-way file merge
│   │   ├── encoding.rs  # hex, binary and HTML entities
│   │   ├── base64.rs    # standard and URL-safe Base64
│   │   ├── base32.rs    # RFC 4648 Base32
│   │   ├── headers.rs   # HTTP header parsing and redaction
│   │   ├── jsonschema.rs # offline JSON Schema validation
│   │   ├── qr.rs        # terminal and SVG QR codes
│   │   ├── limits.rs    # bounded UTF-8 file and stream input
│   │   ├── url.rs       # percent coding and URL parsing
│   │   ├── number.rs    # base and byte-size conversion
│   │   ├── color.rs     # CSS color parsing and contrast
│   │   ├── ip.rs        # IPv4/IPv6 and CIDR inspection
│   │   ├── http.rs      # offline HTTP status table
│   │   ├── mime.rs      # MIME/extension lookup
│   │   ├── timestamp.rs # Unix time conversions
│   │   ├── cron.rs      # cron parser, describer and next-run calculator
│   │   ├── uuid.rs      # UUID v4/v7 generation
│   │   ├── hash.rs      # MD5 / SHA-256 / SHA-512
│   │   ├── hmac.rs      # HMAC-SHA-256 / HMAC-SHA-512
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
4. If the tool needs two fields, give it a `secondary` field and handle `Focus::Secondary`. A third field uses `tertiary` and `Focus::Tertiary`.

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

## Releasing

The release workflow in `.github/workflows/release.yml` builds archives for Windows, Linux, and macOS whenever a version tag is pushed.

1. Update `version` in `Cargo.toml` and commit the change.
2. Push a matching tag:

    ```bash
    git tag -a v0.2.1 -m "DevToys CLI v0.2.1"
    git push origin v0.2.1
    ```

3. GitHub Actions builds the four platform archives, generates `SHA256SUMS`, and publishes a GitHub Release with generated notes.
4. Publish the crate separately when ready:

    ```bash
    cargo publish
    ```

Tags that do not match the version in `Cargo.toml` are rejected by the workflow.

## License

MIT © [PloxiQ](https://github.com/PloxiQ). See [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with <a href="https://www.rust-lang.org">Rust</a>, <a href="https://ratatui.rs">ratatui</a> and <a href="https://github.com/clap-rs/clap">clap</a>.</sub><br>
  <sub>If DevToys CLI saves you a browser tab, <a href="https://github.com/PloxiQ/DevToys-CLI/stargazers">give it a star</a> — it helps other developers find it.</sub>
</p>

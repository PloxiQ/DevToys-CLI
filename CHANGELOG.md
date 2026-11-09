# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-11-10

### Added

- Six new TUI entries and five new tool families: RFC 4648 Base32 encode/decode,
  offline HTTP header inspection with redaction, bounded three-way file merging,
  terminal and SVG QR codes, and offline JSON Schema validation. The toolbox now
  has 51 TUI entries across both the TUI and CLI.
- HTTP header parsing preserves duplicate headers, accepts request and status
  lines, canonicalizes names, and can redact authorization and cookie values.
- Three-way merge prints standard conflict markers and exits with status 1 on
  conflicts; clean merges print only the merged text.
- JSON Schema validation uses internal references only and reports JSON Pointer
  paths with arbitrary-precision numbers.
- `devtoys completions <shell>` generates completion scripts for Bash, Zsh, Fish,
  PowerShell and Elvish.
- `cargo binstall devtoys-cli` can install prebuilt release archives.
- The release workflow now runs the full quality gate and verifies that the tag
  matches `Cargo.toml` before building archives.

### Changed

- CI now checks documentation with warnings denied, runs every cargo command
  with `--locked`, and the MSRV job also checks the CLI-only build.
- The release workflow grants `contents: write` only to the publishing job.

## [0.2.1] - 2026-11-08

### Fixed

- `json get` preserves object key order and arbitrary-precision numbers.
- IP scope classification now covers CGNAT, benchmarking, reserved, 6to4,
  NAT64, Teredo and other special-purpose ranges.
- CIDR parsing accepts spaces around `/`.
- CSV delimiters preserve their case.

### Changed

- JSON to CSV columns follow first-seen key order instead of alphabetical order.

## [0.2.0] - 2026-11-08

### Added

- Expanded the toolbox to 45 offline tools across the TUI and CLI.
- Added a detailed tool reference, examples, banner and screenshots.
- Added automated Windows, Linux and macOS releases with `SHA256SUMS`.

## [0.1.0] - 2026-11-07

### Added

- Initial release with JSON, Base64, URL, timestamp, cron, case, number, byte
  size, color, text, diff, hex, binary, HTML, CSV, HTTP, MIME, IP, HMAC, UUID,
  hash, Lorem Ipsum, random, password, JWT and regex tools.
- Interactive TUI and pipe-friendly CLI in a single offline binary.

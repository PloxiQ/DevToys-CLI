# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `devtoys completions <shell>` generates completion scripts for Bash, Zsh, Fish,
  PowerShell and Elvish.
- The release workflow now runs the full quality gate and verifies that the tag
  matches `Cargo.toml` before building archives.

## [0.2.1] - 2026-11-08

### Fixed

- `json get` preserves object key order and arbitrary-precision numbers.
- IP scope classification now covers CGNAT, benchmarking, reserved, 6to4,
  NAT64, Teredo and other special-purpose ranges.
- CIDR parsing accepts spaces around `/`.
- CSV delimiters preserve their case.

### Changed

- JSON to CSV columns follow first-seen key order instead of alphabetical order.
- CI now checks documentation with warnings denied.

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

# Changelog

All notable changes to claims-toolkit are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-18

### Added
- `era835` crate: X12 835 ERA parser with denial analysis, CARC code library
- `era835-synth` crate: Synthetic 835 generator with seeded RNG
- `phi-scan` crate: PHI scanner covering 18 HIPAA Safe Harbor categories
- `claims-toolkit` unified CLI with subcommands: parse, generate, scan, completions, info
- Shell completions for bash, zsh, fish, PowerShell, elvish
- CSV export for denial reports (`--csv` flag)
- JSON output for all commands (`--json` flag)
- Better error messages with troubleshooting guidance
- Sample ERA files and clinical notes in `samples/`
- CI pipeline (GitHub Actions: test, release)
- 69 automated tests (58 parser + 6 generator + 5 PHI scanner)
- Fuzz testing: 535K+ iterations, zero crashes

### Known Issues
- PHI scanner may have minor false positives on numeric patterns
- Synthetic data uses flat 12% denial rate (not payer-specific)
- No pre-built binaries yet (build from source)

# Claims Toolkit

Healthcare data tools for X12 835 ERA files and PHI scanning. Three tools, one CLI, zero cloud dependencies.

## Install

```bash
# From source (requires Rust 1.85+)
git clone https://github.com/KidIkaros/claims-toolkit.git
cd claims-toolkit
cargo install --path crates/claims-toolkit-cli

# Or build all binaries
cargo build --release
# Binaries: claims-toolkit, era835, synthetic-era835, phi-scan
```

## Quick Start

```bash
# Generate a test ERA file
claims-toolkit generate -n 5 --seed 42 -o test.835

# Parse it
claims-toolkit parse test.835 summary

# Get denial report
claims-toolkit parse test.835 denials

# Export denials to CSV (for spreadsheets)
claims-toolkit parse test.835 denials --csv

# Scan clinical text for PHI
echo 'Patient: John Smith, SSN 123-45-6789' | claims-toolkit scan

# Redact PHI
echo 'Patient: John Smith, SSN 123-45-6789' | claims-toolkit scan --redact
```

## Commands

### `claims-toolkit parse <file> [output]`

Parse an X12 835 ERA remittance file.

```
claims-toolkit parse remittance.835              # Full report
claims-toolkit parse remittance.835 summary      # Financial summary
claims-toolkit parse remittance.835 denials      # Denial report with appeals
claims-toolkit parse remittance.835 json         # Full JSON
claims-toolkit parse remittance.835 summary --json
claims-toolkit parse remittance.835 denials --json
claims-toolkit parse remittance.835 denials --csv
```

### `claims-toolkit generate [options]`

Generate synthetic X12 835 ERA files.

```
claims-toolkit generate -n 10                   # 10 claims to stdout
claims-toolkit generate -n 10 -o test.835       # Save to file
claims-toolkit generate -n 10 --seed 42         # Reproducible
claims-toolkit generate -n 5 --json             # JSON output
```

### `claims-toolkit scan [file] [options]`

Scan text for Protected Health Information.

```
claims-toolkit scan note.txt                    # Scan file
claims-toolkit scan --redact note.txt           # Redact PHI
claims-toolkit scan --json note.txt             # JSON output
echo 'text' | claims-toolkit scan               # From stdin
```

Detects: Names, Geographic, Dates, Phone, Fax, Email, SSN, MRN,
Health Plan IDs, Account Numbers, Certificates, Vehicle IDs,
Device IDs, URLs, IP Addresses, Biometric terms, CPT codes, ICD-10 codes

### `claims-toolkit completions <shell>`

Generate shell completions.

```bash
# Bash
claims-toolkit completions bash > ~/.local/share/bash-completion/completions/claims-toolkit

# Zsh
claims-toolkit completions zsh > ~/.zfunc/_claims-toolkit

# Fish
claims-toolkit completions fish > ~/.config/fish/completions/claims-toolkit.fish
```

## Libraries

Each tool is also a Rust library:

```toml
[dependencies]
era835 = "0.1"          # Parse X12 835 ERA files
era835-synth = "0.1"    # Generate synthetic 835 files
phi-scan = "0.1"        # Scan and redact PHI
```

## Examples

See `samples/` for example files:
- `realistic_5claim.835` — Multi-claim ERA with denials
- `minimal_1claim.835` — Single claim ERA
- `clinical_note_sample.txt` — Clinical note with PHI

## Testing

```bash
cargo test --release
# 69 tests: 58 parser + 6 generator + 5 PHI scanner
```

## License

Apache-2.0 OR MIT

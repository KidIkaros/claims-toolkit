# Claims Toolkit

Healthcare data tools for X12 835 ERA files and PHI scanning. Three tools, one CLI, zero cloud dependencies.

## Install

```bash
# From source (requires Rust 1.85+)
cargo install --path crates/claims-toolkit-cli

# Or build all binaries
cargo build --release
# Binaries in target/release/: claims-toolkit, era835, synthetic-era835, phi-scan
```

## Quick Start

```bash
# 1. Generate a test 835 file
claims-toolkit generate -n 5 --seed 42 -o test.835

# 2. Parse it
claims-toolkit parse test.835 summary

# 3. Get denial report
claims-toolkit parse test.835 denials

# 4. Scan clinical text for PHI
echo 'Patient: John Smith, SSN 123-45-6789' | claims-toolkit scan

# 5. Redact PHI
echo 'Patient: John Smith, SSN 123-45-6789' | claims-toolkit scan --redact
```

## Commands

### `claims-toolkit parse <file>`

Parse an X12 835 ERA remittance file.

```
claims-toolkit parse remittance.835            # Full claim-by-claim report
claims-toolkit parse remittance.835 summary    # Financial summary
claims-toolkit parse remittance.835 denials    # Denial report with appeal recommendations
claims-toolkit parse remittance.835 json       # Raw JSON output
claims-toolkit parse remittance.835 summary --json  # JSON summary
claims-toolkit parse remittance.835 denials --json  # JSON denials
```

### `claims-toolkit generate`

Generate synthetic X12 835 ERA files for testing.

```
claims-toolkit generate -n 10                  # 10 claims to stdout
claims-toolkit generate -n 10 -o test.835      # Save to file
claims-toolkit generate -n 10 --seed 42        # Reproducible (same seed = same output)
claims-toolkit generate -n 5 --json            # JSON instead of X12
```

### `claims-toolkit scan`

Scan text for Protected Health Information (PHI).

```
claims-toolkit scan note.txt                   # Scan file, show report
claims-toolkit scan --redact note.txt          # Redact PHI
claims-toolkit scan --json note.txt            # JSON output
echo 'text' | claims-toolkit scan              # Read from stdin
echo 'text' | claims-toolkit scan --redact     # Redact from stdin
```

Detects all 18 HIPAA Safe Harbor identifier categories:
Names, Geographic, Dates, Phone, Fax, Email, SSN, MRN,
Health Plan IDs, Account Numbers, Certificates, Vehicle IDs,
Device IDs, URLs, IP Addresses, Biometric terms, CPT codes, ICD-10 codes

## Libraries

Each tool is also available as a Rust library:

```toml
[dependencies]
era835 = "0.1"         # Parse X12 835 ERA files
era835-synth = "0.1"   # Generate synthetic 835 files
phi-scan = "0.1"       # Scan and redact PHI
```

## Examples

See `samples/` for example files:
- `realistic_5claim.835` — Multi-claim ERA with denials
- `minimal_1claim.835` — Single claim ERA
- `clinical_note_sample.txt` — Clinical note with PHI for scanning

## Testing

```
era835:         58 tests (16 unit + 42 integration)
era835-synth:    6 roundtrip tests
phi-scan:        5 tests
Total:          69 tests
```

```bash
cargo test --release
```

## License

Apache-2.0 OR MIT

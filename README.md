# Claims Toolkit

Healthcare data tools — parse, generate, and scan X12 835 ERA files and clinical text.

Three standalone tools in one repo. No cloud dependencies.

## Tools

| Tool | Library | CLI | Purpose |
|------|---------|-----|---------|
| **era835** | `era835` | `era835` | Parse X12 835 ERA remittance files |
| **era835-synth** | `era835-synth` | `synthetic-era835` | Generate realistic 835 test files |
| **phi-scan** | `phi-scan` | `phi-scan` | Detect and redact PHI (18 HIPAA categories) |

## Quick Start

```bash
# Build all tools
cargo build --release

# Generate a synthetic 835 file
synthetic-era835 --count 10 --seed 42 -o test.835

# Parse it
era835 test.835                    # full report
era835 denials test.835            # denial report with appeal recommendations
era835 summary --json test.835     # JSON summary

# Scan text for PHI
echo 'Patient: John Smith, SSN 123-45-6789' | phi-scan
echo 'Patient: John Smith, SSN 123-45-6789' | phi-scan redact
phi-scan scan --json < clinical_note.txt

# Pipeline: generate → parse → analyze
synthetic-era835 -n 50 | era835 denials --json
```

## era835 — ERA Parser

Parses X12 835 Electronic Remittance Advice files. Extracts:

- Payer/payee identification
- Claim payment details (status, amounts, patient info)
- Service line adjudication (CPT codes, modifiers, charges)
- CAS adjustments (CO/PR/OA/PI/CR groups with CARC codes)
- Provider-level adjustments (PLB)
- Denial analysis with appeal recommendations

40+ built-in CARC code descriptions with appeal strategies.

## era835-synth — Synthetic Generator

Generates realistic X12 835 ERA files for testing. Features:

- Realistic CPT codes with proper charge distributions
- 12% denial rate with CARC codes
- Service-line level adjudication
- All adjustment group codes (CO, PR, OA, PI, CR)
- Patient names, member IDs, service dates
- Seeded RNG for reproducible test data
- Roundtrip verified: generate → parse → structure matches

## phi-scan — PHI Scanner

Detects Protected Health Information across 18 HIPAA Safe Harbor categories:

Names, Geographic, Dates, Phone, Fax, Email, SSN, MRN,
Health Plan IDs, Account Numbers, Certificates, Vehicle IDs,
Device IDs, URLs, IP Addresses, Biometric terms, CPT codes, ICD-10 codes

## Tests

```
era835:         58 tests (16 unit + 42 integration)
era835-synth:    6 roundtrip tests
phi-scan:        5 tests
Total:          69 tests
```

## License

Apache-2.0 OR MIT

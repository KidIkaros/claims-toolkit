# Changelog

## v0.2.0 — Tier 2: Claims Scrubber (2026-04-20)

### New: `claims-scrub` crate
- Claims scrubbing engine for real-time validation
- NCCI edit checking (mutually exclusive, comprehensive/component, modifier-allowed)
- 200+ recognized CPT/HCPCS modifier codes
- Modifier conflict detection (25+50, 51+59, etc.)
- Diagnosis-procedure linkage validation
- Place of service, ICD-10, CPT format validation
- Denial risk estimation (0-100%) with payer-specific multipliers
- `claim_from_era835()` converts parsed 835 data into scrubable claims
- 9 unit tests

### New: `codes` crate
- CPT code database (53 codes)
- ICD-10-CM database (64 codes)
- CARC (Claim Adjustment Reason Code) database (40 codes)
- Modifier database (61 codes)

### Updated: `claims-toolkit-cli`
- New `scrub` command — validate a single claim (JSON)
- New `check` command — parse 835 + scrub all claims in one pass
- 8 tier integration tests (generate → parse → scrub pipeline)
- Fixed tier integration test for claim ID mapping

## v0.1.0 — Tier 1: Parser, Synth, PHI Scanner (2026-04-20)

- `era835` — X12 835 ERA parser (58 tests)
- `era835-synth` — Synthetic 835 generator (6 tests)
- `phi-scan` — PHI scanner (18 HIPAA categories, 5 tests)
- `claims-toolkit-cli` — Unified binary with parse/generate/scan/info/completions (17 CLI tests)
- GitHub Actions CI pipeline
- Shell completions (bash/zsh/fish/powershell/elvish)

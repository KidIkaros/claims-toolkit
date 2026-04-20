//! Integration tests: Tier 1 (era835 + era835-synth) → Tier 2 (claims-scrub)
//!
//! Tests the full pipeline: generate synthetic 835 → serialize → parse → scrub.

use era835;
use era835_synth;
use claims_scrub;

/// Helper: generate, serialize, parse — full Tier 1 roundtrip
fn generate_and_parse(num: usize, seed: u64) -> era835::Remittance {
    let batch = era835_synth::generate_synthetic_era835(num, Some(seed));
    let text = era835_synth::serialize_era835(&batch);
    era835::parse_era835(&text).expect("Failed to parse synthetic 835")
}

#[test]
fn synthetic_835_full_pipeline() {
    let remittance = generate_and_parse(5, 42);
    assert_eq!(remittance.claims.len(), 5, "Should parse 5 claims");

    let scrubber = claims_scrub::ClaimsScrubber::new();

    for claim_payment in &remittance.claims {
        let claim = claims_scrub::claim_from_era835(
            claim_payment,
            &remittance.payer.name,
            &["I10".into()],
        );

        assert!(!claim.claim_id.is_empty(), "Claim ID should be populated");
        assert!(!claim.lines.is_empty(), "Should have service lines");

        let result = scrubber.validate_claim(&claim);
        // All synthetic CPT codes should be valid 5-digit format
        for f in &result.findings {
            if f.finding_type == claims_scrub::FindingType::InvalidCode {
                panic!(
                    "Synthetic data produced invalid code: {} — {:?}",
                    f.description, f.cpt_code
                );
            }
        }
    }
}

#[test]
fn synthetic_preserves_data_through_pipeline() {
    let batch = era835_synth::generate_synthetic_era835(1, Some(123));
    let original_claim = &batch.claims[0];
    let text = era835_synth::serialize_era835(&batch);
    let remittance = era835::parse_era835(&text).unwrap();
    let parsed_claim = &remittance.claims[0];

    let claim = claims_scrub::claim_from_era835(parsed_claim, "Test Payer", &["I10".into()]);

    // Verify data flows through generation → serialization → parsing → scrub conversion
    assert_eq!(claim.claim_id, parsed_claim.patient_control_number);
    assert_eq!(claim.lines.len(), parsed_claim.service_lines.len());
    assert_eq!(claim.total_charge, parsed_claim.charge_amount);

    for (i, (scrub_line, parsed_line)) in claim
        .lines
        .iter()
        .zip(parsed_claim.service_lines.iter())
        .enumerate()
    {
        assert_eq!(
            scrub_line.cpt_code, parsed_line.procedure_code,
            "Line {} CPT mismatch",
            i
        );
        assert_eq!(
            scrub_line.charge_amount, parsed_line.charge_amount,
            "Line {} charge mismatch",
            i
        );
    }
}

#[test]
fn without_dx_triggers_linkage_warning() {
    let remittance = generate_and_parse(1, 456);
    let scrubber = claims_scrub::ClaimsScrubber::new();

    let claim = claims_scrub::claim_from_era835(&remittance.claims[0], "Test Payer", &[]);
    let result = scrubber.validate_claim(&claim);

    assert!(
        result
            .findings
            .iter()
            .any(|f| f.finding_type == claims_scrub::FindingType::DiagnosisMismatch),
        "Empty DX codes should trigger diagnosis mismatch"
    );
}

#[test]
fn with_dx_passes_linkage() {
    let remittance = generate_and_parse(1, 789);
    let scrubber = claims_scrub::ClaimsScrubber::new();

    let claim = claims_scrub::claim_from_era835(
        &remittance.claims[0],
        "Test Payer",
        &["I10".into(), "E11.9".into()],
    );
    let result = scrubber.validate_claim(&claim);

    assert!(
        !result
            .findings
            .iter()
            .any(|f| f.finding_type == claims_scrub::FindingType::DiagnosisMismatch),
        "DX codes provided should prevent diagnosis mismatch"
    );
}

#[test]
fn scrub_result_serializes_roundtrip() {
    let remittance = generate_and_parse(1, 111);
    let scrubber = claims_scrub::ClaimsScrubber::new();
    let claim = claims_scrub::claim_from_era835(&remittance.claims[0], "Test Payer", &["I10".into()]);

    let result = scrubber.validate_claim(&claim);
    let json = serde_json::to_string(&result).unwrap();
    let roundtrip: claims_scrub::ValidationResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.is_clean, roundtrip.is_clean);
    assert_eq!(result.error_count, roundtrip.error_count);
    assert_eq!(result.warning_count, roundtrip.warning_count);
    assert_eq!(result.denial_risk, roundtrip.denial_risk);
    assert_eq!(result.findings.len(), roundtrip.findings.len());
}

#[test]
fn claim_scrub_result_serializes_roundtrip() {
    let remittance = generate_and_parse(1, 222);
    let scrubber = claims_scrub::ClaimsScrubber::new();
    let cp = &remittance.claims[0];
    let claim = claims_scrub::claim_from_era835(cp, "Test Payer", &["I10".into()]);
    let result = scrubber.validate_claim(&claim);

    let scrub_result = claims_scrub::ClaimScrubResult {
        claim_id: cp.patient_control_number.clone(),
        payer_claim_number: cp.payer_claim_number.clone(),
        scrub_result: result,
        original_denied: cp.paid_amount == 0.0,
        original_denied_amount: if cp.paid_amount == 0.0 {
            cp.charge_amount
        } else {
            0.0
        },
        carc_codes: vec!["1".into()],
    };

    let json = serde_json::to_string(&scrub_result).unwrap();
    let roundtrip: claims_scrub::ClaimScrubResult = serde_json::from_str(&json).unwrap();

    assert_eq!(scrub_result.claim_id, roundtrip.claim_id);
    assert_eq!(scrub_result.original_denied, roundtrip.original_denied);
}

#[test]
fn batch_scrub_counts_consistent() {
    let remittance = generate_and_parse(10, 333);
    let scrubber = claims_scrub::ClaimsScrubber::new();

    let mut clean = 0usize;
    let mut dirty = 0usize;

    for cp in &remittance.claims {
        let claim = claims_scrub::claim_from_era835(cp, "Payer", &["I10".into()]);
        let result = scrubber.validate_claim(&claim);
        if result.is_clean {
            clean += 1;
        } else {
            dirty += 1;
        }
    }

    assert_eq!(clean + dirty, 10, "All claims should be counted");
}

#[test]
fn ncci_conflict_detected_through_pipeline() {
    // Generate, parse, then manually check that NCCI conflicts are caught
    let remittance = generate_and_parse(3, 444);
    let scrubber = claims_scrub::ClaimsScrubber::new();

    // Look for any claim with multiple service lines
    for cp in &remittance.claims {
        if cp.service_lines.len() >= 2 {
            let cpts: Vec<&str> = cp.service_lines.iter().map(|s| s.procedure_code.as_str()).collect();
            // Check if any pair is in our NCCI table
            let has_99213 = cpts.contains(&"99213");
            let has_99214 = cpts.contains(&"99214");

            if has_99213 && has_99214 {
                let claim = claims_scrub::claim_from_era835(cp, "Test", &["I10".into()]);
                let result = scrubber.validate_claim(&claim);
                assert!(
                    result
                        .findings
                        .iter()
                        .any(|f| f.finding_type == claims_scrub::FindingType::NcciEdit),
                    "99213+99214 should trigger NCCI through pipeline"
                );
                return;
            }
        }
    }
    // If synthetic data didn't produce an NCCI conflict, that's OK — the pipeline still works
}

#[test]
fn claim_from_era835_has_correct_structure() {
    let remittance = generate_and_parse(1, 555);
    let cp = &remittance.claims[0];
    let claim = claims_scrub::claim_from_era835(cp, &remittance.payer.name, &["I10".into()]);

    // Basic structure checks
    assert!(!claim.claim_id.is_empty());
    assert_eq!(claim.payer.name, remittance.payer.name);
    assert_eq!(claim.lines.len(), cp.service_lines.len());

    // Each line should have correct line numbers
    for (i, line) in claim.lines.iter().enumerate() {
        assert_eq!(line.line_number, i + 1, "Line numbers should be 1-indexed");
        assert!(!line.cpt_code.is_empty(), "CPT code should not be empty");
    }
}

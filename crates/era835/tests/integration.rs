//! Integration tests for the medassist-era835 parser.
//!
//! Tests against realistic 835 fixtures covering:
//! - Multi-claim ERAs with various status codes
//! - All adjustment group codes (CO, PR, OA, PI, CR)
//! - Service-line and claim-level adjustments
//! - Remark codes and allowed amounts
//! - Provider-level adjustments (PLB)
//! - Modifier codes in service lines
//! - Edge cases (minimal files, zero amounts, reversals)

use era835::*;
use std::fs;

fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}.835", name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", path, e))
}

// ── Complex Multi-Claim Fixture ──

#[test]
fn complex_multi_claim_parses_all_claims() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    assert_eq!(era.claims.len(), 5, "Should have 5 claims");
}

#[test]
fn complex_multi_claim_financial_totals() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    assert_eq!(era.total_charged(), 4050.0);
    assert_eq!(era.total_paid(), 2000.0);
    assert_eq!(era.total_denied(), 2050.0);
}

#[test]
fn complex_multi_claim_payer_info() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    assert_eq!(era.payer.name, "UNITED HEALTHCARE");
    assert_eq!(era.payer.id, "98765");
}

#[test]
fn complex_multi_claim_payee_info() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    assert_eq!(era.payee.name, "MULTISPECIALTY CLINIC LLC");
    assert_eq!(era.payee.npi, "1234567890");
    assert_eq!(era.payee.tax_id, Some("987654321".to_string()));
}

#[test]
fn complex_multi_claim_payment_info() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    assert_eq!(era.payment.total_amount, 4875.50);
    assert_eq!(era.payment.method, "ACH");
}

#[test]
fn complex_multi_claim_status_codes() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let statuses: Vec<&str> = era.claims.iter().map(|c| c.claim_status_code.as_str()).collect();
    assert!(statuses.contains(&"1"), "Should have primary processed claim");
    assert!(statuses.contains(&"2"), "Should have secondary processed claim");
    assert!(statuses.contains(&"4"), "Should have denied claim");
    assert!(statuses.contains(&"22"), "Should have reversal claim");
}

#[test]
fn complex_multi_claim_denied_claim() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let denied = &era.claims[2]; // PCN-2024-003
    assert_eq!(denied.claim_status_code, "4");
    assert_eq!(denied.paid_amount, 0.0);
    assert_eq!(denied.charge_amount, 500.0);
}

#[test]
fn complex_multi_claim_reversal() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let reversal = &era.claims[4]; // PCN-2024-005
    assert_eq!(reversal.claim_status_code, "22");
    assert_eq!(reversal.paid_amount, 0.0);
}

#[test]
fn complex_multi_claim_all_adjustment_groups() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let all_adjs: Vec<&Adjustment> = era.claims.iter()
        .flat_map(|c| c.adjustments.iter())
        .chain(era.claims.iter().flat_map(|c| c.service_lines.iter().flat_map(|s| s.adjustments.iter())))
        .collect();

    let groups: std::collections::HashSet<String> = all_adjs.iter()
        .map(|a| format!("{:?}", a.group_code))
        .collect();

    assert!(groups.contains("CO"), "Should have CO adjustments");
    assert!(groups.contains("PR"), "Should have PR adjustments");
    assert!(groups.contains("CR"), "Should have CR adjustments");
}

#[test]
fn complex_multi_claim_service_lines() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let total_services: usize = era.claims.iter().map(|c| c.service_lines.len()).sum();
    assert_eq!(total_services, 10, "Should have 10 service lines total");
}

#[test]
fn complex_multi_claim_remark_codes() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let claim1 = &era.claims[0];
    let svc1 = &claim1.service_lines[0];
    assert!(svc1.remark_codes.contains(&"MED01".to_string()), "Should have remark code");
}

#[test]
fn complex_multi_claim_allowed_amount() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let svc = &era.claims[0].service_lines[0];
    assert_eq!(svc.allowed_amount, Some(135.0));
}

#[test]
fn complex_multi_claim_provider_adjustments() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    assert_eq!(era.provider_adjustments.len(), 2);
    assert_eq!(era.provider_adjustments[0].amount, -150.0);
    assert_eq!(era.provider_adjustments[1].amount, -25.50);
}

#[test]
fn complex_multi_claim_patient_info() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    assert_eq!(era.claims[0].patient_name, Some("MARY ANDERSON".to_string()));
    assert_eq!(era.claims[0].patient_member_id, Some("UHC-MBR-001".to_string()));
    assert_eq!(era.claims[2].patient_name, Some("ROBERT JOHNSON".to_string()));
}

#[test]
fn complex_multi_claim_service_dates() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let claim = &era.claims[0];
    assert!(claim.service_date_from.is_some(), "Should have service date from");
    assert!(claim.service_date_to.is_some(), "Should have service date to");
}

#[test]
fn complex_multi_claim_denial_summaries() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let summaries = era.denial_summaries();
    assert!(summaries.len() >= 3, "Should have at least 3 denial summaries");

    let full_denials: Vec<_> = summaries.iter().filter(|s| s.denial_type == DenialType::FullDenial).collect();
    assert!(!full_denials.is_empty(), "Should have at least one full denial");
}

#[test]
fn complex_multi_claim_carc_codes_present() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let summaries = era.denial_summaries();

    for summary in &summaries {
        assert!(!summary.carc_codes.is_empty(), "Each denial should have CARC codes");
        assert!(!summary.denial_reasons.is_empty(), "Each denial should have reasons");
    }
}

// ── Minimal Fixture ──

#[test]
fn minimal_no_claims_parses() {
    let era = parse_era835(&load_fixture("minimal_no_claims")).unwrap();
    assert_eq!(era.claims.len(), 0);
    assert_eq!(era.total_charged(), 0.0);
    assert_eq!(era.denial_rate(), 0.0);
}

#[test]
fn minimal_no_claims_denial_summaries_empty() {
    let era = parse_era835(&load_fixture("minimal_no_claims")).unwrap();
    assert!(era.denial_summaries().is_empty());
}

#[test]
fn minimal_no_claims_payer() {
    let era = parse_era835(&load_fixture("minimal_no_claims")).unwrap();
    assert_eq!(era.payer.name, "TEST PAYER");
}

// ── Edge Cases Fixture ──

#[test]
fn edge_cases_parses_all_claims() {
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    assert_eq!(era.claims.len(), 3);
}

#[test]
fn edge_cases_check_payment() {
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    assert_eq!(era.payment.method, "CHK");
    assert_eq!(era.payment.check_number, Some("123456".to_string()));
}

#[test]
fn edge_cases_multiple_cas_per_segment() {
    // CAS segments can have up to 6 triplets
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    let claim1 = &era.claims[0];
    // First claim has CAS*CO*45*500.00*0*2*250.00*97*250.00 — two triplets
    let co_adjs: Vec<_> = claim1.adjustments.iter()
        .filter(|a| a.group_code == AdjustmentGroup::CO)
        .collect();
    assert!(co_adjs.len() >= 2, "Should parse multiple CAS triplets");
}

#[test]
fn edge_cases_oa_adjustment_group() {
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    let claim2 = &era.claims[1];
    let oa_adjs: Vec<_> = claim2.adjustments.iter()
        .filter(|a| a.group_code == AdjustmentGroup::OA)
        .collect();
    assert!(!oa_adjs.is_empty(), "Should have OA (Other Adjustments)");
}

#[test]
fn edge_cases_pi_adjustment_group() {
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    let claim3 = &era.claims[2];
    let pi_adjs: Vec<_> = claim3.adjustments.iter()
        .filter(|a| a.group_code == AdjustmentGroup::PI)
        .collect();
    assert!(!pi_adjs.is_empty(), "Should have PI (Payer Initiated Reductions)");
}

#[test]
fn edge_cases_modifier_parsing() {
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    let svc = &era.claims[0].service_lines[0];
    assert_eq!(svc.procedure_code, "99215");
    assert!(svc.modifiers.contains(&"25".to_string()), "Should parse modifier 25");
}

#[test]
fn edge_cases_long_payee_name() {
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    assert_eq!(era.payee.name, "DR. JANE SMITH FAMILY PRACTICE");
}

#[test]
fn edge_cases_service_date_range() {
    let era = parse_era835(&load_fixture("edge_cases")).unwrap();
    let claim = &era.claims[0];
    assert!(claim.service_date_from.is_some());
    assert!(claim.service_date_to.is_some());
}

// ── CARC Code Tests ──

#[test]
fn carc_all_common_codes_have_descriptions() {
    let common_codes = ["1", "2", "3", "4", "5", "6", "9", "11", "16", "18",
        "22", "23", "24", "26", "27", "29", "31", "32", "33", "35",
        "39", "40", "45", "49", "50", "55", "58", "59", "96", "97",
        "109", "119", "125", "140", "167", "170", "171", "181", "182",
        "197", "198", "199", "204", "226", "227", "234", "236", "242", "243", "252", "256"];

    for code in &common_codes {
        assert!(carc_description(code).is_some(),
            "CARC {} should have a description", code);
    }
}

#[test]
fn carc_unknown_code_returns_none() {
    assert!(carc_description("999999").is_none());
    assert!(carc_description("").is_none());
    assert!(carc_description("ABC").is_none());
}

#[test]
fn appeal_recommendations_coverage() {
    let appealable_codes = ["4", "16", "18", "29", "31", "39", "45", "50", "59", "89", "125", "197", "226"];
    for code in &appealable_codes {
        assert!(appeal_recommendation(code).is_some(),
            "CARC {} should have an appeal recommendation", code);
    }
}

#[test]
fn appeal_recommendations_non_appealable() {
    // Patient responsibility codes generally aren't appealable
    assert!(appeal_recommendation("1").is_none(), "Deductible is patient responsibility");
    assert!(appeal_recommendation("2").is_none(), "Coinsurance is patient responsibility");
    assert!(appeal_recommendation("3").is_none(), "Copay is patient responsibility");
}

// ── Error Handling ──

#[test]
fn empty_input_returns_error() {
    let result = parse_era835("");
    assert!(result.is_err());
}

#[test]
fn whitespace_only_returns_error() {
    let result = parse_era835("   \n\t  ");
    assert!(result.is_err());
}

#[test]
fn no_segment_terminator_returns_error() {
    let result = parse_era835("ISA*00*stuff without tildes");
    assert!(result.is_err());
}

#[test]
fn garbage_data_returns_error_or_empty() {
    // Random garbage should either error or return empty claims
    let result = parse_era835("~~~~~*~*~*~");
    match result {
        Ok(era) => assert!(era.claims.is_empty()),
        Err(_) => {} // Error is also acceptable
    }
}

// ── AdjustmentGroup ──

#[test]
fn adjustment_group_denial_indicators() {
    assert!(AdjustmentGroup::CO.is_denial_indicator());
    assert!(AdjustmentGroup::OA.is_denial_indicator());
    assert!(AdjustmentGroup::PI.is_denial_indicator());
    assert!(!AdjustmentGroup::PR.is_denial_indicator());
    assert!(!AdjustmentGroup::CR.is_denial_indicator());
}

#[test]
fn adjustment_group_code_labels() {
    use era835::AdjustmentGroup;
    let adj = Adjustment { group_code: AdjustmentGroup::CO, reason_code: "45".into(), amount: 10.0, quantity: None };
    assert_eq!(adj.group_code_label(), "CO");

    let adj = Adjustment { group_code: AdjustmentGroup::PR, reason_code: "1".into(), amount: 10.0, quantity: None };
    assert_eq!(adj.group_code_label(), "PR");
}

// ── Financial Calculations ──

#[test]
fn denial_rate_zero_charged() {
    let era = parse_era835(&load_fixture("minimal_no_claims")).unwrap();
    assert_eq!(era.denial_rate(), 0.0);
}

#[test]
fn denial_rate_calculation() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let expected = (era.total_denied() / era.total_charged()) * 100.0;
    assert!((era.denial_rate() - expected).abs() < 0.01);
}

// ── JSON Serialization ──

#[test]
fn remittance_serializes_to_json() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let json = serde_json::to_string(&era).unwrap();
    assert!(json.contains("\"payer\""));
    assert!(json.contains("\"claims\""));
    assert!(json.contains("\"BLUE CROSS BLUE SHIELD\"") || json.contains("UNITED HEALTHCARE"));
}

#[test]
fn denial_summary_serializes_to_json() {
    let era = parse_era835(&load_fixture("complex_multi_claim")).unwrap();
    let summaries = era.denial_summaries();
    let json = serde_json::to_string(&summaries).unwrap();
    assert!(json.contains("\"denial_type\""));
    assert!(json.contains("\"carc_codes\""));
}

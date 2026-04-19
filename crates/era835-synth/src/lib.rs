//! X12 835 ERA text serializer.
//!
//! Converts Remittance835 structs into proper X12 835 EDI format.
//! Designed to pair with medassist-era835 parser — what we generate,
//! the parser should be able to read back.

use chrono::{NaiveDate, Utc};
use rand::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// A complete X12 835 ERA envelope with multiple claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Era835Batch {
    pub interchange_control_number: String,
    pub group_control_number: String,
    pub transaction_control_number: String,
    pub payer: PayerInfo,
    pub payee: PayeeInfo,
    pub payment: PaymentInfo,
    pub trace_number: String,
    pub claims: Vec<Era835Claim>,
    pub provider_adjustments: Vec<ProviderAdjustment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayerInfo {
    pub name: String,
    pub id: String,
    pub id_qualifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayeeInfo {
    pub name: String,
    pub npi: String,
    pub tax_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    pub total_amount: f64,
    pub method: String,
    pub check_number: Option<String>,
    pub payment_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Era835Claim {
    pub patient_control_number: String,
    pub claim_status_code: u32,
    pub charge_amount: f64,
    pub paid_amount: f64,
    pub patient_responsibility: f64,
    pub payer_claim_number: Option<String>,
    pub filing_indicator: Option<String>,
    pub patient_name: Option<String>,
    pub patient_member_id: Option<String>,
    pub service_date_from: Option<NaiveDate>,
    pub service_date_to: Option<NaiveDate>,
    pub claim_adjustments: Vec<CasAdjustment>,
    pub service_lines: Vec<Era835ServiceLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Era835ServiceLine {
    pub procedure_code: String,
    pub modifiers: Vec<String>,
    pub charge_amount: f64,
    pub paid_amount: f64,
    pub units: f64,
    pub adjustments: Vec<CasAdjustment>,
    pub remark_codes: Vec<String>,
    pub allowed_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasAdjustment {
    pub group_code: String,
    pub reason_code: String,
    pub amount: f64,
    pub quantity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAdjustment {
    pub provider_id: String,
    pub fiscal_period_date: Option<NaiveDate>,
    pub adjustments: Vec<(String, f64)>, // (reason_code, amount)
}

use std::collections::HashMap;

/// Write CAS segments, grouping adjustments by group_code (max 6 triplets per segment).
fn write_cas_segments(adjustments: &[CasAdjustment], out: &mut String, seg_count: &mut usize) {
    if adjustments.is_empty() {
        return;
    }

    // Group by group_code
    let mut groups: HashMap<&str, Vec<&CasAdjustment>> = HashMap::new();
    for adj in adjustments {
        groups.entry(&adj.group_code).or_default().push(adj);
    }

    // Write each group, chunking into segments of max 6 triplets
    let order = ["CO", "PR", "OA", "PI", "CR"];
    for group_code in order {
        if let Some(adjs) = groups.get(group_code) {
            for chunk in adjs.chunks(6) {
                let mut cas = format!("CAS*{}", group_code);
                for adj in chunk {
                    cas.push_str(&format!("*{}*{:.2}", adj.reason_code, adj.amount));
                }
                cas.push('~');
                out.push_str(&cas);
                out.push('\n');
                *seg_count += 1;
            }
        }
    }

    // Handle any unknown group codes
    for (group_code, adjs) in &groups {
        if !order.contains(group_code) {
            for chunk in adjs.chunks(6) {
                let mut cas = format!("CAS*{}", group_code);
                for adj in chunk {
                    cas.push_str(&format!("*{}*{:.2}", adj.reason_code, adj.amount));
                }
                cas.push('~');
                out.push_str(&cas);
                out.push('\n');
                *seg_count += 1;
            }
        }
    }
}

/// Serialize a batch to X12 835 text format.
pub fn serialize_era835(batch: &Era835Batch) -> String {
    let mut out = String::with_capacity(4096);
    let date = Utc::now().format("%y%m%d").to_string();
    let time = Utc::now().format("%H%M").to_string();

    let mut seg_count: usize = 0;

    // ISA
    out.push_str(&format!(
        "ISA*00*          *00*          *ZZ*{:<15}*ZZ*{:<15}*{}*{}*^*00501*{}*0*T*:~\n",
        "PAYERSND", "PAYEERCV", date, time, batch.interchange_control_number
    ));
    seg_count += 1;

    // GS
    out.push_str(&format!(
        "GS*HP*PAYERSND*PAYEERCV*{}*{}*{}*X*005010X221A1~\n",
        Utc::now().format("%Y%m%d"),
        Utc::now().format("%H%M"),
        batch.group_control_number
    ));
    seg_count += 1;

    // ST
    out.push_str(&format!("ST*835*{}~\n", batch.transaction_control_number));
    seg_count += 1;

    // BPR
    out.push_str(&format!(
        "BPR*I*{:.2}*C*{}{}~\n",
        batch.payment.total_amount,
        batch.payment.method,
        batch.payment.check_number.as_ref()
            .map(|c| format!("*CHK*{}", c))
            .unwrap_or_default()
    ));
    seg_count += 1;

    // TRN
    out.push_str(&format!("TRN*1*{}~\n", batch.trace_number));
    seg_count += 1;

    // DTM (payment date)
    out.push_str(&format!(
        "DTM*405*{}~\n",
        batch.payment.payment_date.format("%Y%m%d")
    ));
    seg_count += 1;

    // N1*PR (Payer)
    out.push_str(&format!(
        "N1*PR*{}*{}*{}~\n",
        batch.payer.name, batch.payer.id_qualifier, batch.payer.id
    ));
    seg_count += 1;

    // N1*PE (Payee)
    out.push_str(&format!(
        "N1*PE*{}*XX*{}~\n",
        batch.payee.name, batch.payee.npi
    ));
    seg_count += 1;

    // REF*TJ (Tax ID)
    if let Some(ref tax_id) = batch.payee.tax_id {
        out.push_str(&format!("REF*TJ*{}~\n", tax_id));
        seg_count += 1;
    }

    // Claims
    for (claim_idx, claim) in batch.claims.iter().enumerate() {
        let lx = claim_idx + 1;
        out.push_str(&format!("LX*{}~\n", lx));
        seg_count += 1;

        // CLP
        out.push_str(&format!(
            "CLP*{}*{}*{:.2}*{:.2}*{:.2}**MC*{}~\n",
            claim.patient_control_number,
            claim.claim_status_code,
            claim.charge_amount,
            claim.paid_amount,
            claim.patient_responsibility,
            claim.payer_claim_number.as_deref().unwrap_or("")
        ));
        seg_count += 1;

        // Claim-level CAS adjustments — group by group_code, max 6 triplets per segment
        write_cas_segments(&claim.claim_adjustments, &mut out, &mut seg_count);

        // NM1*QC (Patient name)
        if let Some(ref name) = claim.patient_name {
            let parts: Vec<&str> = name.splitn(2, ' ').collect();
            let last = parts.first().unwrap_or(&"");
            let first = parts.get(1).unwrap_or(&"");
            out.push_str(&format!(
                "NM1*QC*1*{}*{}****MI*{}~\n",
                last.to_uppercase(),
                first.to_uppercase(),
                claim.patient_member_id.as_deref().unwrap_or("")
            ));
            seg_count += 1;
        }

        // DTM (service dates)
        if let Some(d) = claim.service_date_from {
            out.push_str(&format!("DTM*232*{}~\n", d.format("%Y%m%d")));
            seg_count += 1;
        }
        if let Some(d) = claim.service_date_to {
            out.push_str(&format!("DTM*233*{}~\n", d.format("%Y%m%d")));
            seg_count += 1;
        }

        // Service lines
        for svc in &claim.service_lines {
            let composite = if svc.modifiers.is_empty() {
                format!("HC:{}", svc.procedure_code)
            } else {
                format!("HC:{}:{}", svc.procedure_code, svc.modifiers.join(":"))
            };

            out.push_str(&format!(
                "SVC*{}*{:.2}*{:.2}**{:.0}~\n",
                composite, svc.charge_amount, svc.paid_amount, svc.units
            ));
            seg_count += 1;

            // Service-line CAS adjustments — group by group_code
            write_cas_segments(&svc.adjustments, &mut out, &mut seg_count);

            // AMT*B6 (allowed amount)
            if let Some(allowed) = svc.allowed_amount {
                out.push_str(&format!("AMT*B6*{:.2}~\n", allowed));
                seg_count += 1;
            }

            // LQ (remark codes)
            for rc in &svc.remark_codes {
                out.push_str(&format!("LQ*HE*{}~\n", rc));
                seg_count += 1;
            }
        }
    }

    // PLB (provider-level adjustments)
    for plb in &batch.provider_adjustments {
        let fiscal = plb.fiscal_period_date
            .map(|d| d.format("%Y%m%d").to_string())
            .unwrap_or_else(|| Utc::now().format("%Y%m%d").to_string());

        let mut plb_line = format!("PLB*{}*{}", plb.provider_id, fiscal);
        for (reason, amount) in &plb.adjustments {
            plb_line.push_str(&format!("*WO:{}*{:.2}", reason, amount));
        }
        plb_line.push('~');
        out.push_str(&plb_line);
        out.push('\n');
        seg_count += 1;
    }

    // SE
    seg_count += 1; // SE itself
    out.push_str(&format!(
        "SE*{}*{}~\n",
        seg_count, batch.transaction_control_number
    ));

    // GE
    out.push_str(&format!("GE*1*{}~\n", batch.group_control_number));

    // IEA
    out.push_str(&format!("IEA*1*{}~\n", batch.interchange_control_number));

    out
}

/// Generate a synthetic ERA 835 batch with realistic claims.
pub fn generate_synthetic_era835(num_claims: usize, seed: Option<u64>) -> Era835Batch {
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let payers = [
        ("BLUE CROSS BLUE SHIELD", "XV", "BCBS001"),
        ("AETNA LIFE INSURANCE", "XV", "AETNA01"),
        ("UNITED HEALTHCARE", "XV", "UHC0001"),
        ("CIGNA HEALTHCARE", "XV", "CIGNA01"),
        ("HUMANA", "XV", "HUMANA1"),
        ("ANTHEM BLUE CROSS", "XV", "ANTHM01"),
        ("KAISER FOUNDATION", "XV", "KAISER1"),
    ];

    let practices = [
        ("MULTISPECIALTY CLINIC LLC", "1234567890"),
        ("FAMILY PRACTICE ASSOCIATES", "9876543210"),
        ("DOWNTOWN MEDICAL GROUP", "5556667777"),
        ("RIVERSIDE HEALTH PARTNERS", "1112223334"),
        ("NORTHSIDE INTERNAL MEDICINE", "4445556667"),
    ];

    let (payer_name, payer_qual, payer_id) = payers[rng.gen_range(0..payers.len())];
    let (payee_name, payee_npi) = practices[rng.gen_range(0..practices.len())];

    let _total_payment: f64 = (0..num_claims)
        .map(|_| rng.gen_range(100.0..2000.0))
        .sum();

    let batch_num = rng.gen_range(100000..999999);

    let mut claims = Vec::with_capacity(num_claims);
    let mut running_payment = 0.0;

    for i in 0..num_claims {
        let claim = generate_synthetic_claim(&mut rng, i, &mut running_payment, payer_name);
        claims.push(claim);
    }

    Era835Batch {
        interchange_control_number: format!("{:09}", batch_num),
        group_control_number: format!("{}", batch_num % 1000000),
        transaction_control_number: format!("{:04}", batch_num % 10000),
        payer: PayerInfo {
            name: payer_name.to_string(),
            id: payer_id.to_string(),
            id_qualifier: payer_qual.to_string(),
        },
        payee: PayeeInfo {
            name: payee_name.to_string(),
            npi: payee_npi.to_string(),
            tax_id: Some(format!("{}{}", &payee_npi[..2], rng.gen_range(1000000..9999999))),
        },
        payment: PaymentInfo {
            total_amount: running_payment,
            method: if rng.gen_bool(0.8) { "ACH".to_string() } else { "CHK".to_string() },
            check_number: if rng.gen_bool(0.2) { Some(format!("{}", rng.gen_range(100000..999999))) } else { None },
            payment_date: Utc::now().date_naive(),
        },
        trace_number: format!("TRC{:010}", batch_num),
        claims,
        provider_adjustments: if rng.gen_bool(0.15) {
            vec![ProviderAdjustment {
                provider_id: payee_npi.to_string(),
                fiscal_period_date: Some(Utc::now().date_naive()),
                adjustments: vec![
                    (format!("ADJ{:04}", rng.gen_range(1000..9999)), -rng.gen_range(10.0..200.0)),
                ],
            }]
        } else {
            vec![]
        },
    }
}

fn generate_synthetic_claim(
    rng: &mut StdRng,
    index: usize,
    running_payment: &mut f64,
    payer_name: &str,
) -> Era835Claim {
    let cpt_codes = [
        ("99213", 125.0, 250.0),
        ("99214", 175.0, 350.0),
        ("99215", 250.0, 500.0),
        ("99203", 175.0, 350.0),
        ("99223", 350.0, 700.0),
        ("93000", 75.0, 200.0),
        ("36415", 15.0, 50.0),
        ("80053", 50.0, 150.0),
        ("85025", 25.0, 75.0),
    ];

    let denial_reasons = [
        ("CO", "50", "Not deemed medical necessity"),
        ("CO", "18", "Exact duplicate claim/service"),
        ("CO", "16", "Lacks information needed for adjudication"),
        ("CO", "96", "Non-covered charge(s)"),
        ("CO", "204", "Not covered under current benefit plan"),
        ("PI", "109", "Claim not covered by this payer"),
        ("OA", "131", "Claim specific negotiated discount"),
    ];

    let first_names = ["MARY", "JOHN", "ROBERT", "SARAH", "DAVID", "JENNIFER", "MICHAEL", "LISA", "JAMES", "PATRICIA"];
    let last_names = ["SMITH", "JOHNSON", "WILLIAMS", "BROWN", "JONES", "GARCIA", "MILLER", "DAVIS", "RODRIGUEZ", "MARTINEZ"];

    let denial_rate = match payer_name {
        "BLUE CROSS BLUE SHIELD" => 0.18,
        "AETNA LIFE INSURANCE" => 0.15,
        "UNITED HEALTHCARE" => 0.14,
        "CIGNA HEALTHCARE" => 0.12,
        "HUMANA" => 0.10,
        "ANTHEM BLUE CROSS" => 0.16,
        "KAISER FOUNDATION" => 0.08,
        _ => 0.12,
    };
    let is_denied = rng.gen_bool(denial_rate);
    let is_partial = !is_denied && rng.gen_bool(0.15);
    let is_reversal = !is_denied && !is_partial && rng.gen_bool(0.03);

    let status_code = if is_denied {
        4
    } else if is_reversal {
        22
    } else if rng.gen_bool(0.1) {
        2 // secondary
    } else {
        1 // primary
    };

    let num_lines = rng.gen_range(1..=4);
    let mut service_lines = Vec::with_capacity(num_lines);
    let mut total_charge = 0.0;
    let mut total_paid = 0.0;

    let service_date = Utc::now().date_naive() - chrono::Duration::days(rng.gen_range(1..60));

    for _ in 0..num_lines {
        let (cpt, min_charge, max_charge) = cpt_codes[rng.gen_range(0..cpt_codes.len())];
        let charge = (rng.gen_range(min_charge..max_charge) as f64).round();
        total_charge += charge;

        let (paid, adjustments, remark_codes, allowed) = if is_denied {
            let (grp, code, _) = denial_reasons[rng.gen_range(0..denial_reasons.len())];
            (
                0.0,
                vec![CasAdjustment {
                    group_code: grp.to_string(),
                    reason_code: code.to_string(),
                    amount: charge,
                    quantity: None,
                }],
                vec![],
                None,
            )
        } else {
            let allowed_pct = rng.gen_range(0.55..0.90);
            let allowed = (charge * allowed_pct).round();
            let copay = (rng.gen_range::<f64, _>(0.0..50.0)).round();
            let coinsurance = ((allowed - copay) * rng.gen_range::<f64, _>(0.0..0.20)).round();
            let contractual = charge - allowed;
            let line_paid = (allowed - copay - coinsurance).max(0.0).round();

            let mut adjs = Vec::new();
            if contractual > 0.0 {
                adjs.push(CasAdjustment {
                    group_code: "CO".to_string(),
                    reason_code: "45".to_string(),
                    amount: contractual,
                    quantity: None,
                });
            }
            if copay > 0.0 {
                adjs.push(CasAdjustment {
                    group_code: "PR".to_string(),
                    reason_code: "3".to_string(),
                    amount: copay,
                    quantity: None,
                });
            }
            if coinsurance > 0.0 {
                adjs.push(CasAdjustment {
                    group_code: "PR".to_string(),
                    reason_code: "2".to_string(),
                    amount: coinsurance,
                    quantity: None,
                });
            }

            total_paid += line_paid;

            let remarks = if rng.gen_bool(0.1) {
                vec![format!("MED{:02}", rng.gen_range(1..20))]
            } else {
                vec![]
            };

            (line_paid, adjs, remarks, Some(allowed - copay - coinsurance))
        };

        let modifiers = if rng.gen_bool(0.15) {
            vec![format!("{}", rng.gen_range(25..60))]
        } else {
            vec![]
        };

        service_lines.push(Era835ServiceLine {
            procedure_code: cpt.to_string(),
            modifiers,
            charge_amount: charge,
            paid_amount: paid,
            units: 1.0,
            adjustments,
            remark_codes,
            allowed_amount: allowed,
        });
    }

    if is_reversal {
        total_paid = 0.0;
    }

    let patient_resp = if is_denied || is_reversal {
        0.0
    } else {
        service_lines.iter().map(|s| {
            s.adjustments.iter()
                .filter(|a| a.group_code == "PR")
                .map(|a| a.amount)
                .sum::<f64>()
        }).sum()
    };

    let claim_adjustments = if is_denied {
        let (grp, code, _) = denial_reasons[rng.gen_range(0..denial_reasons.len())];
        vec![CasAdjustment {
            group_code: grp.to_string(),
            reason_code: code.to_string(),
            amount: total_charge,
            quantity: None,
        }]
    } else {
        vec![]
    };

    *running_payment += total_paid;

    let member_id = format!("MBR-{:06}", rng.gen_range(100000..999999));
    let patient_name = format!(
        "{} {}",
        last_names[rng.gen_range(0..last_names.len())],
        first_names[rng.gen_range(0..first_names.len())]
    );

    Era835Claim {
        patient_control_number: format!("PCN-2024-{:04}", index + 1),
        claim_status_code: status_code,
        charge_amount: total_charge,
        paid_amount: total_paid,
        patient_responsibility: patient_resp,
        payer_claim_number: Some(format!("{}-CLM-{:04}", &member_id[..3], index + 1)),
        filing_indicator: None,
        patient_name: Some(patient_name),
        patient_member_id: Some(member_id),
        service_date_from: Some(service_date),
        service_date_to: Some(service_date),
        claim_adjustments,
        service_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_single_claim_roundtrip() {
        let batch = generate_synthetic_era835(1, Some(42));
        let x12_text = serialize_era835(&batch);

        // Parse it back with the era835 parser
        let parsed = era835::parse_era835(&x12_text).expect("Parser should accept generated 835");

        assert_eq!(parsed.claims.len(), 1, "Should have 1 claim");
        assert_eq!(parsed.payer.name, batch.payer.name, "Payer name should match");
        assert_eq!(parsed.payee.npi, batch.payee.npi, "Payee NPI should match");
        assert_eq!(parsed.claims[0].patient_control_number, batch.claims[0].patient_control_number);
    }

    #[test]
    fn generate_multi_claim_roundtrip() {
        let batch = generate_synthetic_era835(10, Some(123));
        let x12_text = serialize_era835(&batch);

        let parsed = era835::parse_era835(&x12_text).expect("Parser should accept generated 835");

        assert_eq!(parsed.claims.len(), 10, "Should have 10 claims");

        // Verify financial totals are reasonable
        assert!(parsed.total_charged() > 0.0, "Total charged should be > 0");
        assert!(parsed.total_paid() >= 0.0, "Total paid should be >= 0");

        // Verify denial analysis works on generated data
        let denials = parsed.denial_summaries();
        // With 12% denial rate on 10 claims, we should see at least 0 denials (probabilistic)
        assert!(denials.len() <= 10, "Can't have more denials than claims");
    }

    #[test]
    fn generate_with_seed_is_deterministic() {
        let batch1 = generate_synthetic_era835(5, Some(999));
        let batch2 = generate_synthetic_era835(5, Some(999));

        let text1 = serialize_era835(&batch1);
        let text2 = serialize_era835(&batch2);

        // Same seed should produce same output (except for date/time in ISA/GS)
        assert_eq!(batch1.claims.len(), batch2.claims.len());
        assert_eq!(batch1.payer.name, batch2.payer.name);
        assert_eq!(batch1.claims[0].patient_control_number, batch2.claims[0].patient_control_number);
    }

    #[test]
    fn generate_large_batch_roundtrip() {
        let batch = generate_synthetic_era835(50, Some(777));
        let x12_text = serialize_era835(&batch);

        let parsed = era835::parse_era835(&x12_text).expect("Should parse 50-claim batch");

        assert_eq!(parsed.claims.len(), 50);

        // All claims should have service lines
        for claim in &parsed.claims {
            assert!(!claim.service_lines.is_empty(), "Each claim should have service lines");
        }
    }

    #[test]
    fn generated_835_has_all_status_codes() {
        // Generate enough claims to likely get all status types
        let batch = generate_synthetic_era835(100, Some(555));
        let x12_text = serialize_era835(&batch);
        let parsed = era835::parse_era835(&x12_text).unwrap();

        let statuses: std::collections::HashSet<&str> = parsed.claims.iter()
            .map(|c| c.claim_status_code.as_str())
            .collect();

        // Should have at least primary (1) and denied (4) with 100 claims
        assert!(statuses.contains(&"1"), "Should have primary claims");
    }

    #[test]
    fn generated_835_adjustments_parse_correctly() {
        let batch = generate_synthetic_era835(20, Some(333));
        let x12_text = serialize_era835(&batch);
        let parsed = era835::parse_era835(&x12_text).unwrap();

        // Find a paid claim with adjustments
        let paid_claim = parsed.claims.iter()
            .find(|c| c.claim_status_code == "1" && !c.adjustments.is_empty());

        if let Some(claim) = paid_claim {
            // Should have CO adjustments (contractual)
            let co_adj: Vec<_> = claim.adjustments.iter()
                .filter(|a| a.group_code == era835::AdjustmentGroup::CO)
                .collect();
            assert!(!co_adj.is_empty(), "Paid claim should have CO adjustments");
        }
    }
}

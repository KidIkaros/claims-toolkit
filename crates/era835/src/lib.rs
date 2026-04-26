//! X12 835 Electronic Remittance Advice (ERA) parser.
//!
//! Parses ERA/835 files from payers to extract claim payment details,
//! denials, adjustments, and remark codes. This powers the Claims Denial
//! Management product (P3) by ingesting real payer remittance data.
//!
//! The X12 835 format uses segments separated by `~` and elements by `*`.
//! Key segments parsed:
//! - ISA/IEA: Interchange envelope
//! - GS/GE: Functional group
//! - ST/SE: Transaction set
//! - BPR: Financial information (payment amount, method)
//! - TRN: Reassociation trace number
//! - N1/REF: Payer/payee identification
//! - CLP: Claim payment information
//! - SVC: Service line detail
//! - CAS: Claim adjustment (CARC codes)
//! - PLB: Provider-level adjustments

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// A fully parsed ERA/835 remittance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remittance {
    /// Payer who sent this ERA.
    pub payer: PayerInfo,
    /// Payee (provider) who received payment.
    pub payee: PayeeInfo,
    /// Payment/check information.
    pub payment: PaymentInfo,
    /// Trace number for reassociation with EFT.
    pub trace_number: Option<String>,
    /// Individual claim payment details.
    pub claims: Vec<ClaimPayment>,
    /// Provider-level adjustments (PLB).
    pub provider_adjustments: Vec<ProviderAdjustment>,
    /// Raw segment count for validation.
    pub segment_count: usize,
}

/// Payer identification from N1*PR loop.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PayerInfo {
    pub name: String,
    pub id: String,
    pub id_qualifier: String,
}

/// Payee identification from N1*PE loop.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PayeeInfo {
    pub name: String,
    pub npi: String,
    pub tax_id: Option<String>,
}

/// Payment information from BPR segment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaymentInfo {
    /// Total payment amount (can be negative for recoupments).
    pub total_amount: f64,
    /// Payment method: CHK, ACH, FWT, etc.
    pub method: String,
    /// Check or EFT number.
    pub check_number: Option<String>,
    /// Payment date.
    pub payment_date: Option<NaiveDate>,
}

/// Individual claim payment detail from CLP loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimPayment {
    /// Patient control number (from original claim).
    pub patient_control_number: String,
    /// Claim status: 1=Processed as Primary, 2=Processed as Secondary,
    /// 3=Processed as Tertiary, 4=Denied, 19=Processed as Primary, Forwarded, 22=Reversal.
    pub claim_status_code: String,
    /// Human-readable claim status.
    pub claim_status_desc: String,
    /// Total charged amount.
    pub charge_amount: f64,
    /// Amount paid by payer.
    pub paid_amount: f64,
    /// Patient responsibility amount.
    pub patient_responsibility: f64,
    /// Payer claim control number (payer's internal ID).
    pub payer_claim_number: Option<String>,
    /// Filing indicator code.
    pub filing_indicator: Option<String>,
    /// Service date range.
    pub service_date_from: Option<NaiveDate>,
    pub service_date_to: Option<NaiveDate>,
    /// Patient name.
    pub patient_name: Option<String>,
    /// Patient member ID.
    pub patient_member_id: Option<String>,
    /// Service line details.
    pub service_lines: Vec<ServiceLine>,
    /// Claim-level adjustments.
    pub adjustments: Vec<Adjustment>,
}

/// Service line detail from SVC segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLine {
    /// Procedure code (CPT/HCPCS).
    pub procedure_code: String,
    /// Modifier codes.
    pub modifiers: Vec<String>,
    /// Charged amount.
    pub charge_amount: f64,
    /// Paid amount.
    pub paid_amount: f64,
    /// Units of service.
    pub units: f64,
    /// Adjustments on this service line.
    pub adjustments: Vec<Adjustment>,
    /// Remark codes.
    pub remark_codes: Vec<String>,
    /// Allowed amount.
    pub allowed_amount: Option<f64>,
}

/// Claim or service-line adjustment from CAS segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjustment {
    /// Adjustment group code: CO, PR, OA, PI, CR.
    pub group_code: AdjustmentGroup,
    /// CARC (Claim Adjustment Reason Code).
    pub reason_code: String,
    /// Adjustment amount.
    pub amount: f64,
    /// Number of units adjusted.
    pub quantity: Option<f64>,
}

/// Adjustment group codes per X12 835 spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdjustmentGroup {
    /// Contractual Obligations — provider write-off.
    CO,
    /// Patient Responsibility — deductible, copay, coinsurance.
    PR,
    /// Other Adjustments.
    OA,
    /// Payor Initiated Reductions.
    PI,
    /// Corrections and Reversals.
    CR,
    /// Unknown group code.
    Unknown(String),
}

impl AdjustmentGroup {
    fn from_code(code: &str) -> Self {
        match code {
            "CO" => Self::CO,
            "PR" => Self::PR,
            "OA" => Self::OA,
            "PI" => Self::PI,
            "CR" => Self::CR,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Whether this adjustment represents a denial (not just a write-off).
    pub fn is_denial_indicator(&self) -> bool {
        matches!(self, Self::CO | Self::OA | Self::PI)
    }
}

/// Provider-level adjustment from PLB segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAdjustment {
    pub provider_id: String,
    pub fiscal_period_date: Option<NaiveDate>,
    pub reason_code: String,
    pub amount: f64,
}

/// Errors from ERA parsing.
#[derive(Debug, thiserror::Error)]
pub enum Era835Error {
    #[error("Invalid ERA format: {0}")]
    InvalidFormat(String),
    #[error("Missing required segment: {0}")]
    MissingSegment(String),
    #[error("Parse error in segment {segment}: {detail}")]
    SegmentError { segment: String, detail: String },
}

impl Adjustment {
    /// Human-readable group code label for display.
    pub fn group_code_label(&self) -> &'static str {
        match &self.group_code {
            AdjustmentGroup::CO => "CO",
            AdjustmentGroup::PR => "PR",
            AdjustmentGroup::OA => "OA",
            AdjustmentGroup::PI => "PI",
            AdjustmentGroup::CR => "CR",
            AdjustmentGroup::Unknown(_) => "??",
        }
    }

    /// Whether this adjustment represents a denial (not just a write-off).
    pub fn is_denial_indicator(&self) -> bool {
        self.group_code.is_denial_indicator()
    }
}

/// Denial summary extracted from a claim payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenialSummary {
    /// Patient control number.
    pub claim_id: String,
    /// Whether the claim was fully or partially denied.
    pub denial_type: DenialType,
    /// Total denied amount.
    pub denied_amount: f64,
    /// All CARC codes contributing to the denial.
    pub carc_codes: Vec<String>,
    /// Human-readable denial reasons.
    pub denial_reasons: Vec<String>,
    /// Recommended appeal actions based on CARC codes.
    pub appeal_recommendations: Vec<String>,
}

/// Type of denial.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DenialType {
    /// Entire claim denied (paid $0).
    FullDenial,
    /// Some service lines paid, some denied.
    PartialDenial,
    /// Claim paid less than charged (adjustments only).
    Underpayment,
}

// ── CARC Code Reference ──

/// Common CARC (Claim Adjustment Reason Code) descriptions.
pub fn carc_description(code: &str) -> Option<&'static str> {
    match code {
        "1" => Some("Deductible amount"),
        "2" => Some("Coinsurance amount"),
        "3" => Some("Co-payment amount"),
        "4" => Some("The procedure code is inconsistent with the modifier used"),
        "5" => Some("The procedure code/bill type is inconsistent with the place of service"),
        "6" => Some("The procedure/revenue code is inconsistent with the patient's age"),
        "9" => Some("The diagnosis is inconsistent with the patient's age"),
        "11" => Some("The diagnosis is inconsistent with the procedure"),
        "13" => Some("The date of death precedes the date of service"),
        "16" => Some("Claim/service lacks information needed for adjudication"),
        "18" => Some("Exact duplicate claim/service"),
        "22" => Some("This care may be covered by another payer per coordination of benefits"),
        "23" => Some("The impact of prior payer(s) adjudication including payments and/or adjustments"),
        "24" => Some("Charges are covered under a capitation agreement/managed care plan"),
        "26" => Some("Expenses incurred prior to coverage"),
        "27" => Some("Expenses incurred after coverage terminated"),
        "29" => Some("The time limit for filing has expired"),
        "31" => Some("Patient cannot be identified as our insured"),
        "32" => Some("Our records indicate the patient is not eligible for benefits"),
        "33" => Some("The insured has no dependent coverage"),
        "35" => Some("Lifetime benefit maximum has been reached"),
        "39" => Some("Services denied at the time authorization/pre-certification was requested"),
        "40" => Some("Charges do not meet qualifications for emergent/urgent care"),
        "45" => Some("Charge exceeds fee schedule/maximum allowable"),
        "49" => Some("These are non-covered services because this is a routine/preventive exam"),
        "50" => Some("These are non-covered services because this is not deemed a medical necessity"),
        "55" => Some("Procedure/treatment/drug is deemed experimental/investigational"),
        "58" => Some("Treatment was deemed by the payer to have been rendered in an inappropriate setting"),
        "59" => Some("Processed based on multiple or concurrent procedure rules"),
        "89" => Some("Services not provided or authorized by designated provider"),
        "96" => Some("Non-covered charge(s)"),
        "97" => Some("The benefit for this service is included in the payment for another service"),
        "109" => Some("Claim/service not covered by this payer/contractor"),
        "119" => Some("Benefit maximum for this time period or occurrence has been reached"),
        "125" => Some("Submission/billing error(s)"),
        "140" => Some("Patient/insured health identification number and name do not match"),
        "167" => Some("This (these) diagnosis(es) is (are) not covered"),
        "170" => Some("Payment is denied when performed/billed by this type of provider"),
        "171" => Some("Payment is denied when performed/billed by this type of provider in this type of facility"),
        "181" => Some("Procedure code was invalid on the date of service"),
        "182" => Some("Procedure modifier was invalid on the date of service"),
        "183" => Some("The referring provider is not eligible to refer the service billed"),
        "186" => Some("Level of care change reason"),
        "187" => Some("Consumer Directed/Consumer Driven Health Plan"),
        "197" => Some("Precertification/authorization/notification/pre-treatment absent"),
        "198" => Some("Precertification/authorization/notification/pre-treatment exceeded"),
        "199" => Some("Revenue code and Procedure code do not match"),
        "204" => Some("This service/equipment/drug is not covered under the patient's current benefit plan"),
        "226" => Some("Information requested from the Billing/Rendering Provider was not provided"),
        "227" => Some("Information requested from the patient/insured/responsible party was not provided"),
        "233" => Some("Services/charges related to the treatment of a hospital-acquired condition"),
        "234" => Some("This procedure is not paid separately"),
        "235" => Some("Sales Tax"),
        "236" => Some("This procedure or procedure/modifier combination is not compatible with another procedure"),
        "242" => Some("Services not provided by network/primary care providers"),
        "243" => Some("Services not authorized by network/primary care providers"),
        "252" => Some("An attachment/other documentation is required to adjudicate this claim/service"),
        "256" => Some("Service not payable per managed care contract"),
        _ => None,
    }
}

/// Recommend appeal actions based on CARC codes.
pub fn appeal_recommendation(carc_code: &str) -> Option<&'static str> {
    match carc_code {
        "4" | "5" | "6" | "11" => Some("Review coding: verify procedure-diagnosis match and modifier usage. Recode and resubmit if error found."),
        "16" | "252" => Some("Missing information: gather required documentation and resubmit with attachments."),
        "18" => Some("Duplicate claim: verify original claim status. If not a duplicate, submit with documentation proving separate service."),
        "29" => Some("Timely filing: check payer's filing deadline. If within limit, appeal with proof of timely submission."),
        "31" | "32" | "33" | "140" => Some("Eligibility issue: verify patient demographics and coverage dates. Resubmit with corrected information."),
        "39" | "197" | "198" | "243" => Some("Authorization required: obtain retroactive authorization if possible, or appeal with clinical documentation."),
        "45" => Some("Fee schedule: review contracted rate. If underpaid, appeal with fee schedule documentation."),
        "49" | "50" | "55" | "96" | "109" | "167" | "204" => Some("Non-covered/medical necessity: submit appeal with clinical notes, peer-reviewed literature, and letter of medical necessity."),
        "59" | "97" | "234" => Some("Bundling/multiple procedure: review CCI edits. If procedures are distinct, appeal with modifier 59 documentation."),
        "89" | "170" | "171" | "242" => Some("Provider eligibility: verify network status and provider credentials. Re-refer if needed."),
        "125" | "181" | "182" | "199" => Some("Billing error: correct the coding/billing error and resubmit as a corrected claim."),
        "226" | "227" => Some("Information requested: respond to payer's request with required documentation within deadline."),
        _ => None,
    }
}

// ── Parser ──

/// Parse an X12 835 ERA string into structured data.
pub fn parse_era835(raw: &str) -> Result<Remittance, Era835Error> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Era835Error::InvalidFormat("Empty input".to_string()));
    }

    // Detect segment terminator (usually ~, sometimes ~\n)
    let segment_terminator = if raw.contains('~') { '~' } else {
        return Err(Era835Error::InvalidFormat("No segment terminator (~) found".to_string()));
    };

    let segments: Vec<&str> = raw
        .split(segment_terminator)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        return Err(Era835Error::InvalidFormat("No segments found".to_string()));
    }

    // Detect element separator from ISA segment (position 3)
    let element_sep = if segments[0].starts_with("ISA") && segments[0].len() > 3 {
        segments[0].chars().nth(3).unwrap_or('*')
    } else {
        '*'
    };

    let mut remittance = Remittance {
        payer: PayerInfo::default(),
        payee: PayeeInfo::default(),
        payment: PaymentInfo::default(),
        trace_number: None,
        claims: Vec::new(),
        provider_adjustments: Vec::new(),
        segment_count: segments.len(),
    };

    let mut current_claim: Option<ClaimPayment> = None;
    let mut current_service: Option<ServiceLine> = None;
    let mut _in_payer_loop = false;
    let mut in_payee_loop = false;

    for seg_str in &segments {
        let elements: Vec<&str> = seg_str.split(element_sep).collect();
        if elements.is_empty() {
            continue;
        }

        let seg_id = elements[0];

        match seg_id {
            "BPR" => {
                remittance.payment = parse_bpr(&elements);
            }
            "TRN" => {
                if elements.len() > 2 {
                    remittance.trace_number = Some(elements[2].to_string());
                }
            }
            "N1" => {
                if elements.len() > 1 {
                    match elements[1] {
                        "PR" => {
                            _in_payer_loop = true;
                            in_payee_loop = false;
                            if elements.len() > 2 {
                                remittance.payer.name = elements[2].to_string();
                            }
                            if elements.len() > 4 {
                                remittance.payer.id_qualifier = elements[3].to_string();
                                remittance.payer.id = elements[4].to_string();
                            }
                        }
                        "PE" => {
                            in_payee_loop = true;
                            _in_payer_loop = false;
                            if elements.len() > 2 {
                                remittance.payee.name = elements[2].to_string();
                            }
                            if elements.len() > 4 {
                                remittance.payee.npi = elements[4].to_string();
                            }
                        }
                        _ => {
                            _in_payer_loop = false;
                            in_payee_loop = false;
                        }
                    }
                }
            }
            "REF" => {
                if elements.len() > 2 && in_payee_loop && elements[1] == "TJ" {
                    remittance.payee.tax_id = Some(elements[2].to_string());
                }
            }
            "DTM" => {
                if elements.len() > 2 && elements[1] == "405" {
                    remittance.payment.payment_date = parse_date(elements[2]);
                }
                // Also check for claim-level date if we have a current claim
                if elements.len() > 2 && current_claim.is_some() {
                    if let Some(ref mut claim) = current_claim {
                        match elements[1] {
                            "232" => claim.service_date_from = parse_date(elements[2]),
                            "233" => claim.service_date_to = parse_date(elements[2]),
                            _ => {}
                        }
                    }
                }
            }
            "CLP" => {
                // Flush previous claim
                if let Some(mut claim) = current_claim.take() {
                    if let Some(svc) = current_service.take() {
                        claim.service_lines.push(svc);
                    }
                    remittance.claims.push(claim);
                }
                current_service = None;
                current_claim = Some(parse_clp(&elements));
            }
            "CAS" => {
                let adjustments = parse_cas(&elements);
                if let Some(ref mut svc) = current_service {
                    svc.adjustments.extend(adjustments);
                } else if let Some(ref mut claim) = current_claim {
                    claim.adjustments.extend(adjustments);
                }
            }
            "SVC" => {
                // Flush previous service line
                if let Some(svc) = current_service.take() {
                    if let Some(ref mut claim) = current_claim {
                        claim.service_lines.push(svc);
                    }
                }
                current_service = Some(parse_svc(&elements));
            }
            "AMT" => {
                if elements.len() > 2 && elements[1] == "B6" {
                    if let Some(ref mut svc) = current_service {
                        svc.allowed_amount = elements[2].parse().ok();
                    }
                }
            }
            "LQ" => {
                if elements.len() > 2 {
                    if let Some(ref mut svc) = current_service {
                        svc.remark_codes.push(elements[2].to_string());
                    }
                }
            }
            "NM1" => {
                if let Some(ref mut claim) = current_claim {
                    if elements.len() > 1 && elements[1] == "QC" {
                        // Patient name
                        let last = elements.get(3).unwrap_or(&"");
                        let first = elements.get(4).unwrap_or(&"");
                        claim.patient_name = Some(format!("{} {}", first, last).trim().to_string());
                        if elements.len() > 9 {
                            claim.patient_member_id = Some(elements[9].to_string());
                        }
                    }
                }
            }
            "PLB" => {
                let adj = parse_plb(&elements);
                remittance.provider_adjustments.extend(adj);
            }
            _ => {
                debug!(segment = seg_id, "Skipping unhandled ERA segment");
            }
        }
    }

    // Flush last claim and service line
    if let Some(mut claim) = current_claim {
        if let Some(svc) = current_service {
            claim.service_lines.push(svc);
        }
        remittance.claims.push(claim);
    }

    Ok(remittance)
}

fn parse_bpr(elements: &[&str]) -> PaymentInfo {
    PaymentInfo {
        total_amount: elements.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        method: elements.get(4).unwrap_or(&"").to_string(),
        check_number: elements.get(6).map(|s| s.to_string()).filter(|s| !s.is_empty()),
        payment_date: elements.get(16).and_then(|s| parse_date(s)),
    }
}

fn parse_clp(elements: &[&str]) -> ClaimPayment {
    let status_code = elements.get(2).unwrap_or(&"0").to_string();
    let status_desc = match status_code.as_str() {
        "1" => "Processed as Primary",
        "2" => "Processed as Secondary",
        "3" => "Processed as Tertiary",
        "4" => "Denied",
        "19" => "Processed as Primary, Forwarded to Additional Payer(s)",
        "20" => "Processed as Secondary, Forwarded to Additional Payer(s)",
        "21" => "Processed as Tertiary, Forwarded to Additional Payer(s)",
        "22" => "Reversal of Previous Payment",
        "23" => "Not Our Claim, Forwarded to Additional Payer(s)",
        "25" => "Predetermination Pricing Only, No Payment",
        _ => "Unknown",
    };

    ClaimPayment {
        patient_control_number: elements.get(1).unwrap_or(&"").to_string(),
        claim_status_code: status_code,
        claim_status_desc: status_desc.to_string(),
        charge_amount: elements.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        paid_amount: elements.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        patient_responsibility: elements.get(5).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        payer_claim_number: elements.get(7).map(|s| s.to_string()).filter(|s| !s.is_empty()),
        filing_indicator: elements.get(6).map(|s| s.to_string()).filter(|s| !s.is_empty()),
        service_date_from: None,
        service_date_to: None,
        patient_name: None,
        patient_member_id: None,
        service_lines: Vec::new(),
        adjustments: Vec::new(),
    }
}

fn parse_cas(elements: &[&str]) -> Vec<Adjustment> {
    if elements.len() < 4 {
        return Vec::new();
    }
    // CAS segments can have up to 6 reason/amount/quantity triplets = max 6 adjustments
    let max_adjustments = (elements.len() - 2) / 3;
    let mut adjustments = Vec::with_capacity(max_adjustments);
    let group = AdjustmentGroup::from_code(elements[1]);

    // CAS segments can have up to 6 reason/amount/quantity triplets
    let mut i = 2;
    while i + 1 < elements.len() {
        let reason = elements[i].to_string();
        let amount = elements.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let quantity = elements.get(i + 2).and_then(|s| s.parse().ok());

        if !reason.is_empty() {
            adjustments.push(Adjustment {
                group_code: group.clone(),
                reason_code: reason,
                amount,
                quantity,
            });
        }
        i += 3;
    }
    // Shrink to actual size if significantly over-allocated
    adjustments.shrink_to_fit();
    adjustments
}

fn parse_svc(elements: &[&str]) -> ServiceLine {
    // SVC*HC:procedure_code:modifier1:modifier2*charge*paid*...*units
    let (proc_code, modifiers) = if let Some(composite) = elements.get(1) {
        let parts: Vec<&str> = composite.split(':').collect();
        let code = parts.get(1).or(parts.first()).unwrap_or(&"").to_string();
        let mods: Vec<String> = parts.iter().skip(2).map(|m| m.to_string()).filter(|m| !m.is_empty()).collect();
        (code, mods)
    } else {
        (String::new(), Vec::new())
    };

    ServiceLine {
        procedure_code: proc_code,
        modifiers,
        charge_amount: elements.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        paid_amount: elements.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        units: elements.get(5).and_then(|s| s.parse().ok()).unwrap_or(1.0),
        adjustments: Vec::new(),
        remark_codes: Vec::new(),
        allowed_amount: None,
    }
}

fn parse_plb(elements: &[&str]) -> Vec<ProviderAdjustment> {
    if elements.len() < 4 {
        return Vec::new();
    }
    // Estimate capacity: each adjustment uses 2 elements after header
    let max_adjs = (elements.len() - 3) / 2;
    let mut adjs = Vec::with_capacity(max_adjs);
    let provider_id = elements[1].to_string();
    let fiscal_date = elements.get(2).and_then(|s| parse_date(s));

    let mut i = 3;
    while i + 1 < elements.len() {
        let reason_composite = elements[i];
        let reason = reason_composite.split(':').next().unwrap_or("").to_string();
        let amount = elements.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0.0);

        if !reason.is_empty() {
            adjs.push(ProviderAdjustment {
                provider_id: provider_id.clone(),
                fiscal_period_date: fiscal_date,
                reason_code: reason,
                amount,
            });
        }
        i += 2;
    }
    adjs.shrink_to_fit();
    adjs
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    if s.len() == 8 {
        NaiveDate::parse_from_str(s, "%Y%m%d").ok()
    } else {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    }
}

// ── Denial Analysis ──

impl Remittance {
    /// Extract all denied or underpaid claims (excluding reversals).
    pub fn denied_claims(&self) -> Vec<&ClaimPayment> {
        self.claims
            .iter()
            .filter(|c| c.claim_status_code != "22") // Reversals aren't denials
            .filter(|c| c.claim_status_code == "4" || c.paid_amount < c.charge_amount)
            .collect()
    }

    /// Generate denial summaries for all denied/underpaid claims.
    pub fn denial_summaries(&self) -> Vec<DenialSummary> {
        self.denied_claims()
            .into_iter()
            .map(|claim| {
                let denied_amount = claim.charge_amount - claim.paid_amount;
                let is_full_denial = claim.claim_status_code == "4" || claim.paid_amount == 0.0;

                let mut carc_codes: Vec<String> = Vec::new();
                let mut reasons: Vec<String> = Vec::new();
                let mut recommendations: Vec<String> = Vec::new();

                // Collect from claim-level adjustments
                for adj in &claim.adjustments {
                    if adj.group_code.is_denial_indicator() || adj.group_code == AdjustmentGroup::PR {
                        carc_codes.push(adj.reason_code.clone());
                        if let Some(desc) = carc_description(&adj.reason_code) {
                            reasons.push(format!("CARC {}: {}", adj.reason_code, desc));
                        }
                        if let Some(rec) = appeal_recommendation(&adj.reason_code) {
                            recommendations.push(rec.to_string());
                        }
                    }
                }

                // Collect from service-line adjustments
                for svc in &claim.service_lines {
                    for adj in &svc.adjustments {
                        if adj.group_code.is_denial_indicator() && !carc_codes.contains(&adj.reason_code) {
                            carc_codes.push(adj.reason_code.clone());
                            if let Some(desc) = carc_description(&adj.reason_code) {
                                reasons.push(format!("CARC {}: {}", adj.reason_code, desc));
                            }
                            if let Some(rec) = appeal_recommendation(&adj.reason_code) {
                                if !recommendations.contains(&rec.to_string()) {
                                    recommendations.push(rec.to_string());
                                }
                            }
                        }
                    }
                }

                let denial_type = if is_full_denial {
                    DenialType::FullDenial
                } else if claim.service_lines.iter().any(|s| s.paid_amount == 0.0) {
                    DenialType::PartialDenial
                } else {
                    DenialType::Underpayment
                };

                DenialSummary {
                    claim_id: claim.patient_control_number.clone(),
                    denial_type,
                    denied_amount,
                    carc_codes,
                    denial_reasons: reasons,
                    appeal_recommendations: recommendations,
                }
            })
            .collect()
    }

    /// Total amount paid across all claims.
    pub fn total_paid(&self) -> f64 {
        self.claims.iter().map(|c| c.paid_amount).sum()
    }

    /// Total amount charged across all claims.
    pub fn total_charged(&self) -> f64 {
        self.claims.iter().map(|c| c.charge_amount).sum()
    }

    /// Total denied amount across all claims.
    pub fn total_denied(&self) -> f64 {
        self.total_charged() - self.total_paid()
    }

    /// Denial rate as a percentage.
    pub fn denial_rate(&self) -> f64 {
        let charged = self.total_charged();
        if charged == 0.0 {
            0.0
        } else {
            (self.total_denied() / charged) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_era835() -> &'static str {
        "ISA*00*          *00*          *ZZ*SENDER         *ZZ*RECEIVER       *230615*1200*^*00501*000000001*0*T*:~\
         GS*HP*SENDER*RECEIVER*20230615*1200*1*X*005010X221A1~\
         ST*835*0001~\
         BPR*I*1250.00*C*ACH*CCP*01*111222333*DA*999888777**01*222333444*DA*888999111*20230615~\
         TRN*1*TRACE123456*1111111111~\
         DTM*405*20230615~\
         N1*PR*BLUE CROSS BLUE SHIELD*XV*12345~\
         N1*PE*ACME MEDICAL GROUP*XX*1234567890~\
         REF*TJ*123456789~\
         CLP*CLAIM001*1*500.00*400.00*50.00**MC*PAYERCLM001~\
         CAS*CO*45*50.00~\
         CAS*PR*1*40.00*0*2*10.00~\
         NM1*QC*1*DOE*JOHN****MI*MBR001~\
         SVC*HC:99213*250.00*200.00**1~\
         CAS*CO*45*25.00~\
         CAS*PR*1*20.00*0*2*5.00~\
         AMT*B6*225.00~\
         SVC*HC:85025*250.00*200.00**1~\
         CAS*CO*45*25.00~\
         CAS*PR*1*20.00*0*2*5.00~\
         CLP*CLAIM002*4*300.00*0.00*0.00**MC*PAYERCLM002~\
         CAS*CO*50*300.00~\
         NM1*QC*1*SMITH*JANE****MI*MBR002~\
         SVC*HC:99214*300.00*0.00**1~\
         CAS*CO*50*300.00~\
         PLB*1234567890*20230615*WO:ADJ001*-25.00~\
         SE*24*0001~\
         GE*1*1~\
         IEA*1*000000001~"
    }

    #[test]
    fn test_parse_basic_era835() {
        let era = parse_era835(sample_era835()).unwrap();
        assert_eq!(era.payer.name, "BLUE CROSS BLUE SHIELD");
        assert_eq!(era.payee.name, "ACME MEDICAL GROUP");
        assert_eq!(era.payee.npi, "1234567890");
        assert_eq!(era.trace_number, Some("TRACE123456".to_string()));
    }

    #[test]
    fn test_payment_info() {
        let era = parse_era835(sample_era835()).unwrap();
        assert_eq!(era.payment.total_amount, 1250.0);
        assert_eq!(era.payment.method, "ACH");
    }

    #[test]
    fn test_claim_count() {
        let era = parse_era835(sample_era835()).unwrap();
        assert_eq!(era.claims.len(), 2);
    }

    #[test]
    fn test_paid_claim() {
        let era = parse_era835(sample_era835()).unwrap();
        let claim1 = &era.claims[0];
        assert_eq!(claim1.patient_control_number, "CLAIM001");
        assert_eq!(claim1.claim_status_code, "1");
        assert_eq!(claim1.charge_amount, 500.0);
        assert_eq!(claim1.paid_amount, 400.0);
        assert_eq!(claim1.patient_responsibility, 50.0);
    }

    #[test]
    fn test_denied_claim() {
        let era = parse_era835(sample_era835()).unwrap();
        let claim2 = &era.claims[1];
        assert_eq!(claim2.patient_control_number, "CLAIM002");
        assert_eq!(claim2.claim_status_code, "4");
        assert_eq!(claim2.charge_amount, 300.0);
        assert_eq!(claim2.paid_amount, 0.0);
    }

    #[test]
    fn test_service_lines() {
        let era = parse_era835(sample_era835()).unwrap();
        let claim1 = &era.claims[0];
        assert_eq!(claim1.service_lines.len(), 2);
        assert_eq!(claim1.service_lines[0].procedure_code, "99213");
        assert_eq!(claim1.service_lines[0].charge_amount, 250.0);
        assert_eq!(claim1.service_lines[0].paid_amount, 200.0);
    }

    #[test]
    fn test_adjustments() {
        let era = parse_era835(sample_era835()).unwrap();
        let claim1 = &era.claims[0];
        // Claim-level adjustments
        assert!(!claim1.adjustments.is_empty());
        // Should have CO*45 and PR*1, PR*2
        let co_adj: Vec<_> = claim1.adjustments.iter().filter(|a| a.group_code == AdjustmentGroup::CO).collect();
        assert!(!co_adj.is_empty());
    }

    #[test]
    fn test_denial_summaries() {
        let era = parse_era835(sample_era835()).unwrap();
        let summaries = era.denial_summaries();
        assert!(!summaries.is_empty(), "Should have at least 1 denial");

        let full_denial = summaries.iter().find(|s| s.denial_type == DenialType::FullDenial);
        assert!(full_denial.is_some(), "Should have a full denial");
        let denial = full_denial.unwrap();
        assert_eq!(denial.claim_id, "CLAIM002");
        assert_eq!(denial.denied_amount, 300.0);
        assert!(denial.carc_codes.contains(&"50".to_string()));
    }

    #[test]
    fn test_carc_descriptions() {
        assert!(carc_description("1").unwrap().contains("Deductible"));
        assert!(carc_description("45").unwrap().contains("fee schedule"));
        assert!(carc_description("50").unwrap().contains("medical necessity"));
        assert!(carc_description("999999").is_none());
    }

    #[test]
    fn test_appeal_recommendations() {
        let rec = appeal_recommendation("50").unwrap();
        assert!(rec.contains("medical necessity"));
        let rec = appeal_recommendation("16").unwrap();
        assert!(rec.contains("Missing information"));
    }

    #[test]
    fn test_financial_totals() {
        let era = parse_era835(sample_era835()).unwrap();
        assert_eq!(era.total_charged(), 800.0);
        assert_eq!(era.total_paid(), 400.0);
        assert_eq!(era.total_denied(), 400.0);
        assert!((era.denial_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_provider_adjustments() {
        let era = parse_era835(sample_era835()).unwrap();
        assert!(!era.provider_adjustments.is_empty());
        assert_eq!(era.provider_adjustments[0].provider_id, "1234567890");
        assert_eq!(era.provider_adjustments[0].amount, -25.0);
    }

    #[test]
    fn test_patient_info() {
        let era = parse_era835(sample_era835()).unwrap();
        let claim2 = &era.claims[1];
        assert_eq!(claim2.patient_name, Some("JANE SMITH".to_string()));
        assert_eq!(claim2.patient_member_id, Some("MBR002".to_string()));
    }

    #[test]
    fn test_empty_input() {
        let result = parse_era835("");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_terminator() {
        let result = parse_era835("ISA*00*stuff");
        assert!(result.is_err());
    }

    #[test]
    fn test_allowed_amount() {
        let era = parse_era835(sample_era835()).unwrap();
        let svc = &era.claims[0].service_lines[0];
        assert_eq!(svc.allowed_amount, Some(225.0));
    }
}

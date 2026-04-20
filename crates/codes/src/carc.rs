//! CARC/RARC (Claim Adjustment Reason Codes and Remittance Advice Remark Codes)
//!
//! These codes explain why a claim was adjusted, denied, or paid differently than billed.

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// CARC code categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CarcCategory {
    /// Deductible, coinsurance, co-payment
    BeneficiaryLiability,
    /// Coordination of benefits, multiple payers
    CoordinationOfBenefits,
    /// Procedure/treatment not covered
    NotCovered,
    /// Medical necessity
    MedicalNecessity,
    /// Prior authorization, precertification
    PriorAuthorization,
    /// Billing errors, duplicates
    BillingErrors,
    /// Processing errors by payer
    ProcessingErrors,
    /// Documentation issues
    Documentation,
    /// Other reasons
    Other,
}

/// CARC (Claim Adjustment Reason Code) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarcCode {
    /// The CARC code number
    pub code: &'static str,

    /// Full description
    pub description: &'static str,

    /// Category of this CARC
    pub category: CarcCategory,

    /// Typical appeal success rate (0.0 to 1.0)
    pub appeal_success_rate: f32,

    /// Whether this is typically a "fixable" denial
    pub fixable: bool,

    /// Suggested action for this CARC
    pub suggested_action: &'static str,
}

/// RARC (Remittance Advice Remark Code) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarcCode {
    /// The RARC code
    pub code: &'static str,

    /// Full description
    pub description: &'static str,

    /// Associated CARC codes that commonly use this RARC
    pub associated_carc: Vec<String>,
}

/// CARC code database
pub static CARC_DATABASE: Lazy<FxHashMap<&'static str, CarcCode>> = Lazy::new(|| {
    let mut m = FxHashMap::default();

    // Group 1: Deductible, Coinsurance, Co-payment
    m.insert(
        "1",
        CarcCode {
            code: "1",
            description: "Deductible amount",
            category: CarcCategory::BeneficiaryLiability,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Verify patient's deductible status and collect appropriately",
        },
    );
    m.insert(
        "2",
        CarcCode {
            code: "2",
            description: "Coinsurance amount",
            category: CarcCategory::BeneficiaryLiability,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Verify patient's coinsurance percentage and collect appropriately",
        },
    );
    m.insert(
        "3",
        CarcCode {
            code: "3",
            description: "Co-payment amount",
            category: CarcCategory::BeneficiaryLiability,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Verify patient's co-payment amount and collect appropriately",
        },
    );

    // Group 16-22: Claim/Payment Adjustments
    m.insert("16", CarcCode {
        code: "16",
        description: "Claim/service lacks information or has submission/billing error(s) which is needed for adjudication",
        category: CarcCategory::BillingErrors,
        appeal_success_rate: 0.85,
        fixable: true,
        suggested_action: "Correct the missing or invalid information and resubmit",
    });
    m.insert(
        "19",
        CarcCode {
            code: "19",
            description: "Claim/service does not meet coverage criteria for diagnosis/procedure",
            category: CarcCategory::MedicalNecessity,
            appeal_success_rate: 0.80,
            fixable: true,
            suggested_action:
                "Review medical necessity documentation, consider adding modifier 25 or 32",
        },
    );
    m.insert(
        "20",
        CarcCode {
            code: "20",
            description: "Claim/service does not meet coverage criteria for diagnosis/procedure",
            category: CarcCategory::MedicalNecessity,
            appeal_success_rate: 0.75,
            fixable: true,
            suggested_action: "Provide additional documentation supporting medical necessity",
        },
    );
    m.insert(
        "21",
        CarcCode {
            code: "21",
            description:
                "The claim does not meet coverage criteria for the diagnostic test ordered",
            category: CarcCategory::MedicalNecessity,
            appeal_success_rate: 0.70,
            fixable: true,
            suggested_action: "Ensure test results are documented and support medical necessity",
        },
    );
    m.insert(
        "22",
        CarcCode {
            code: "22",
            description: "This care may be covered by another payer per coordination of benefits",
            category: CarcCategory::CoordinationOfBenefits,
            appeal_success_rate: 0.70,
            fixable: true,
            suggested_action:
                "Verify coordination of benefits and update primary payer information",
        },
    );

    // Group 45-54: Procedure/Billing Issues
    m.insert("45", CarcCode {
        code: "45",
        description: "Charge exceeds fee schedule/maximum allowable or contracted/legislated fee arrangement",
        category: CarcCategory::BillingErrors,
        appeal_success_rate: 0.05,
        fixable: false,
        suggested_action: "Adjust charge to contracted rate; cannot be billed to patient",
    });
    m.insert("50", CarcCode {
        code: "50",
        description: "These are non-covered services because this is not deemed a medical necessity by the payer",
        category: CarcCategory::MedicalNecessity,
        appeal_success_rate: 0.50,
        fixable: true,
        suggested_action: "Provide comprehensive documentation supporting medical necessity; consider peer-to-peer review",
    });
    m.insert(
        "54",
        CarcCode {
            code: "54",
            description: "Multiple physicians/assistants are not covered in this case",
            category: CarcCategory::BillingErrors,
            appeal_success_rate: 0.30,
            fixable: false,
            suggested_action:
                "Review modifier usage; ensure documentation supports multiple providers",
        },
    );

    // Group 96-200: Coverage and Authorization
    m.insert(
        "96",
        CarcCode {
            code: "96",
            description: "Non-covered procedure or service (not covered by any payer)",
            category: CarcCategory::NotCovered,
            appeal_success_rate: 0.02,
            fixable: false,
            suggested_action: "Patient may be billed; verify ABN was obtained if applicable",
        },
    );
    m.insert("151", CarcCode {
        code: "151",
        description: "Payment adjusted because the payer deems the information submitted does not support this many/frequent services",
        category: CarcCategory::MedicalNecessity,
        appeal_success_rate: 0.40,
        fixable: true,
        suggested_action: "Provide documentation supporting frequency and medical necessity",
    });
    m.insert("152", CarcCode {
        code: "152",
        description: "Payment adjusted because the service/procedure was not performed in the place of service indicated on the claim",
        category: CarcCategory::BillingErrors,
        appeal_success_rate: 0.60,
        fixable: true,
        suggested_action: "Correct place of service code and resubmit",
    });
    m.insert(
        "197",
        CarcCode {
            code: "197",
            description: "Precertification/authorization/notification absent",
            category: CarcCategory::PriorAuthorization,
            appeal_success_rate: 0.10,
            fixable: false,
            suggested_action:
                "Obtain retroactive authorization if possible; otherwise patient responsibility",
        },
    );
    m.insert(
        "200",
        CarcCode {
            code: "200",
            description: "Expense not covered by the payer",
            category: CarcCategory::NotCovered,
            appeal_success_rate: 0.02,
            fixable: false,
            suggested_action: "Review plan benefits; patient may be responsible",
        },
    );
    m.insert("204", CarcCode {
        code: "204",
        description: "This service/equipment/drug is not covered under the patient's current benefit plan",
        category: CarcCategory::NotCovered,
        appeal_success_rate: 0.02,
        fixable: false,
        suggested_action: "Patient may be billed; verify benefit plan coverage",
    });
    m.insert(
        "216",
        CarcCode {
            code: "216",
            description: "Based on the findings of a review organization",
            category: CarcCategory::Other,
            appeal_success_rate: 0.30,
            fixable: true,
            suggested_action:
                "Request detailed review findings; consider appeal with additional documentation",
        },
    );

    // Group 218-227: Other
    m.insert(
        "218",
        CarcCode {
            code: "218",
            description:
                "Payment made for bundled/linked procedure/services (only applicable toProviders)",
            category: CarcCategory::Other,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Review bundling rules; ensure proper modifier usage",
        },
    );
    m.insert(
        "219",
        CarcCode {
            code: "219",
            description: "Payment made for automated/automated tests",
            category: CarcCategory::Other,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Verify correct test code was used",
        },
    );
    m.insert(
        "220",
        CarcCode {
            code: "220",
            description: "Payment made for performance/quality measures",
            category: CarcCategory::Other,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Review value-based program participation",
        },
    );
    m.insert("223", CarcCode {
        code: "223",
        description: "Adjustment code for mandated federal, state or local law/regulation that is not already covered by another code and is mandated before a new code can be created",
        category: CarcCategory::Other,
        appeal_success_rate: 0.01,
        fixable: false,
        suggested_action: "Review regulatory requirements",
    });

    // Group 252-253: Anesthesia
    m.insert(
        "252",
        CarcCode {
            code: "252",
            description: "Anesthesia not covered as the procedure is not covered",
            category: CarcCategory::NotCovered,
            appeal_success_rate: 0.02,
            fixable: false,
            suggested_action: "Anesthesia is not covered when the primary procedure is not covered",
        },
    );
    m.insert(
        "253",
        CarcCode {
            code: "253",
            description: "Anesthesia for this procedure is not covered",
            category: CarcCategory::NotCovered,
            appeal_success_rate: 0.02,
            fixable: false,
            suggested_action: "Review plan anesthesia benefits",
        },
    );

    // Group W1, M51-M64: Additional codes
    m.insert(
        "W1",
        CarcCode {
            code: "W1",
            description: "Provider selected - Medicaid specific",
            category: CarcCategory::Other,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Medicaid managed care selection",
        },
    );
    m.insert("M51", CarcCode {
        code: "M51",
        description: "The attachment or other documentation identified in the notice is not on file or was not submitted for the claim identified on the notice",
        category: CarcCategory::Documentation,
        appeal_success_rate: 0.80,
        fixable: true,
        suggested_action: "Submit the required documentation with a tracking number",
    });
    m.insert(
        "M54",
        CarcCode {
            code: "M54",
            description: "Providers who are not enrolled may not render services to beneficiaries",
            category: CarcCategory::BillingErrors,
            appeal_success_rate: 0.01,
            fixable: false,
            suggested_action: "Provider must be enrolled with the payer",
        },
    );

    m
});

/// RARC code database (select codes)
pub static RARC_DATABASE: Lazy<FxHashMap<&'static str, RarcCode>> = Lazy::new(|| {
    let mut m = FxHashMap::default();

    m.insert(
        "1",
        RarcCode {
            code: "1",
            description: "Federal or State regulations/requirements prevent payment.",
            associated_carc: vec!["16".to_string(), "96".to_string()],
        },
    );
    m.insert(
        "2",
        RarcCode {
            code: "2",
            description: "Service not payable per Managed Care contract.",
            associated_carc: vec!["45".to_string(), "96".to_string()],
        },
    );
    m.insert(
        "3",
        RarcCode {
            code: "3",
            description: "Enrolled in managed care, program not payable.",
            associated_carc: vec!["45".to_string()],
        },
    );
    m.insert(
        "N1",
        RarcCode {
            code: "N1",
            description: "Referral absent or not valid, renders service not payable.",
            associated_carc: vec!["16".to_string(), "197".to_string()],
        },
    );
    m.insert(
        "N5",
        RarcCode {
            code: "N5",
            description: "Timely filing limits have expired.",
            associated_carc: vec!["16".to_string()],
        },
    );
    m.insert(
        "N10",
        RarcCode {
            code: "N10",
            description: "Provider is not authorized to refer or prescribe.",
            associated_carc: vec!["16".to_string()],
        },
    );
    m.insert(
        "N115",
        RarcCode {
            code: "N115",
            description: "This is an initial denial for additional information.",
            associated_carc: vec!["16".to_string(), "22".to_string()],
        },
    );
    m.insert(
        "N290",
        RarcCode {
            code: "N290",
            description: "Referral not required for this service/procedure.",
            associated_carc: vec!["16".to_string()],
        },
    );
    m.insert(
        "M1",
        RarcCode {
            code: "M1",
            description: "The claim was not signed by the provider.",
            associated_carc: vec!["16".to_string()],
        },
    );
    m.insert("M2", RarcCode {
        code: "M2",
        description: "The provider was not enrolled or certified in the state where the service was rendered.",
        associated_carc: vec!["16".to_string(), "54".to_string()],
    });
    m.insert(
        "M62",
        RarcCode {
            code: "M62",
            description: "Missing, incomplete or invalid diagnosis.",
            associated_carc: vec!["16".to_string(), "19".to_string()],
        },
    );
    m.insert(
        "M63",
        RarcCode {
            code: "M63",
            description: "Missing, incomplete or invalid procedure code(s).",
            associated_carc: vec!["16".to_string()],
        },
    );
    m.insert(
        "M64",
        RarcCode {
            code: "M64",
            description: "Missing, incomplete or invalid rendering provider ID.",
            associated_carc: vec!["16".to_string()],
        },
    );

    m
});

/// Find a CARC code by its code string
pub fn find_carc(code: &str) -> Option<&'static CarcCode> {
    CARC_DATABASE.get(code)
}

/// Find a RARC code by its code string
pub fn find_rarc(code: &str) -> Option<&'static RarcCode> {
    RARC_DATABASE.get(code)
}

/// Check if a CARC code is typically appealable
pub fn carc_is_appealable(code: &str) -> bool {
    find_carc(code)
        .map(|c| c.appeal_success_rate > 0.3)
        .unwrap_or(false)
}

/// Get suggested action for a CARC code
pub fn carc_suggested_action(code: &str) -> Option<&'static str> {
    find_carc(code).map(|c| c.suggested_action)
}

/// Check if a CARC indicates a fixable denial
pub fn carc_is_fixable(code: &str) -> bool {
    find_carc(code).map(|c| c.fixable).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_carc() {
        let carc = find_carc("50");
        assert!(carc.is_some());
        assert_eq!(carc.unwrap().code, "50");
    }

    #[test]
    fn test_medical_necessity_carc() {
        let carc = find_carc("50");
        assert_eq!(carc.unwrap().category, CarcCategory::MedicalNecessity);
    }

    #[test]
    fn test_appeal_success_rate() {
        let carc = find_carc("50");
        assert!(carc.unwrap().appeal_success_rate < 0.6);
        assert!(carc.unwrap().appeal_success_rate > 0.3);

        let carc = find_carc("19"); // Processing error
        assert!(carc.unwrap().appeal_success_rate > 0.7);
    }

    #[test]
    fn test_fixable_denials() {
        assert!(carc_is_fixable("19")); // Processing error
        assert!(carc_is_fixable("16")); // Missing info
        assert!(!carc_is_fixable("96")); // Non-covered
    }

    #[test]
    fn test_rarc_lookup() {
        let rarc = find_rarc("N5");
        assert!(rarc.is_some());
        assert!(rarc.unwrap().description.contains("Timely"));
    }
}

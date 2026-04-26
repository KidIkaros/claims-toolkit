//! Claims Scrubber - Real-time claim validation and denial prevention.
//!
//! Validates healthcare claims against NCCI edits, modifier rules, diagnosis-procedure
//! linkage, and demographic constraints. Helps reduce claim denials by identifying
//! errors before submission.
//!
//! # Features
//!
//! - **NCCI Edit Checking**: Mutually exclusive, comprehensive/component, modifier-allowed
//! - **Modifier Validation**: Detects conflicting modifiers (e.g., 25+50, 51+59)
//! - **Diagnosis-Procedure Linkage**: Validates E/M codes, lab codes, surgical codes
//! - **Denial Risk Scoring**: Estimates denial probability (0-100%) with payer-specific multipliers
//!
//! # Quick Start
//!
//! ```no_run
//! use claims_scrub::{ClaimsScrubber, Claim, ClaimLine, PatientInfo, ProviderInfo, PayerInfo};
//!
//! let scrubber = ClaimsScrubber::new();
//!
//! let claim = Claim {
//!     claim_id: "CLM001".to_string(),
//!     patient: PatientInfo {
//!         patient_id: "P001".to_string(),
//!         date_of_birth: "1980-01-01".to_string(),
//!         gender: "M".to_string(),
//!         insurance_id: "INS001".to_string(),
//!     },
//!     provider: ProviderInfo {
//!         npi: "1234567890".to_string(),
//!         taxonomy_code: "207Q00000X".to_string(),
//!         name: "Dr. Smith".to_string(),
//!     },
//!     payer: PayerInfo {
//!         payer_id: "PAYER001".to_string(),
//!         name: "Insurance Co".to_string(),
//!         payer_type: "commercial".to_string(),
//!     },
//!     date_of_service: "2024-01-15".to_string(),
//!     lines: vec![ClaimLine {
//!         line_number: 1,
//!         cpt_code: "99213".to_string(),
//!         modifiers: vec![],
//!         units: 1,
//!         charge_amount: 150.0,
//!         diagnosis_codes: vec!["I10".to_string()],
//!         date_of_service: "2024-01-15".to_string(),
//!         place_of_service: "11".to_string(),
//!     }],
//!     total_charge: 150.0,
//! };
//!
//! let result = scrubber.validate_claim(&claim);
//! println!("Denial risk: {}%", result.denial_risk);
//! ```

pub mod scrubber;

pub use scrubber::{
    Claim, ClaimLine, PatientInfo, ProviderInfo, PayerInfo,
    ClaimsScrubber, ScrubberConfig,
    ValidationResult, ValidationFinding,
    FindingSeverity, FindingType,
    ClaimScrubResult, claim_from_era835,
};

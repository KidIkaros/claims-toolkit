//! X12 837 Professional and Institutional Claims Parser
//!
//! This crate parses X12 837 transaction sets for healthcare claims:
//! - 837P: Professional claims (doctors, physicians)
//! - 837I: Institutional claims (hospitals, facilities)
//!
//! # Example
//!
//! ```
//! use claims_837::parse_837;
//!
//! let x12_content = r#"ISA*00*...~GS*HC*...~ST*837*..."#;
//! match parse_837(x12_content) {
//!     Ok(claim) => println!("Claim ID: {}", claim.claim_id),
//!     Err(e) => eprintln!("Parse error: {}", e),
//! }
//! ```

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error type for 837 parsing failures
#[derive(Debug, thiserror::Error)]
pub enum Claims837Error {
    #[error("Invalid X12 format: {0}")]
    InvalidFormat(String),
    #[error("Missing required segment: {0}")]
    MissingSegment(String),
    #[error("Invalid date format: {0}")]
    InvalidDate(String),
    #[error("Empty input")]
    EmptyInput,
}

/// Parsed 837 claim (supports both Professional and Institutional)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    /// Claim identification
    pub claim_id: String,
    /// Claim type: Professional or Institutional
    pub claim_type: ClaimType,
    /// Patient information
    pub patient: PatientInfo,
    /// Subscriber (insured) information
    pub subscriber: SubscriberInfo,
    /// Provider information
    pub provider: ProviderInfo,
    /// Payer information
    pub payer: PayerInfo,
    /// Claim dates
    pub dates: ClaimDates,
    /// Service lines
    pub service_lines: Vec<ServiceLine>,
    /// Diagnosis codes (ICD-10)
    pub diagnosis_codes: Vec<String>,
    /// Total claim charge amount
    pub total_charge: f64,
    /// Claim status
    pub status: ClaimStatus,
}

/// Type of claim
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimType {
    /// Professional claim (837P)
    Professional,
    /// Institutional claim (837I)
    Institutional,
    /// Dental claim (837D)
    Dental,
}

impl fmt::Display for ClaimType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaimType::Professional => write!(f, "Professional"),
            ClaimType::Institutional => write!(f, "Institutional"),
            ClaimType::Dental => write!(f, "Dental"),
        }
    }
}

/// Patient demographic information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatientInfo {
    /// Patient last name
    pub last_name: String,
    /// Patient first name
    pub first_name: String,
    /// Patient middle name (optional)
    pub middle_name: Option<String>,
    /// Patient date of birth
    pub birth_date: NaiveDate,
    /// Patient gender
    pub gender: Gender,
    /// Patient member ID
    pub member_id: String,
    /// Patient address
    pub address: Address,
}

/// Gender codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Unknown,
    Other,
}

/// Subscriber (insured party) information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriberInfo {
    /// Subscriber last name
    pub last_name: String,
    /// Subscriber first name
    pub first_name: String,
    /// Subscriber member ID
    pub member_id: String,
    /// Group/Policy number
    pub group_number: Option<String>,
    /// Payer identification
    pub payer_id: String,
    /// Relationship to patient (if not self)
    pub relationship: Relationship,
}

/// Relationship to insured
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Relationship {
    Self_Patient,
    Spouse,
    Child,
    Other,
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderInfo {
    /// Provider NPI
    pub npi: String,
    /// Provider taxonomy code
    pub taxonomy: Option<String>,
    /// Provider name
    pub name: String,
    /// Provider address
    pub address: Address,
    /// Tax ID (EIN)
    pub tax_id: Option<String>,
}

/// Payer information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayerInfo {
    /// Payer name
    pub name: String,
    /// Payer ID
    pub payer_id: String,
    /// Address
    pub address: Option<Address>,
}

/// Address structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Address {
    /// Street address line 1
    pub line1: String,
    /// Street address line 2
    pub line2: Option<String>,
    /// City
    pub city: String,
    /// State code
    pub state: String,
    /// ZIP code
    pub zip: String,
}

/// Important claim dates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaimDates {
    /// Date service was rendered (or start date for range)
    pub service_date: NaiveDate,
    /// End date for date range (if applicable)
    pub service_end_date: Option<NaiveDate>,
    /// Claim submission date
    pub submission_date: Option<NaiveDate>,
}

/// Individual service line item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceLine {
    /// Line number
    pub line_number: u32,
    /// Service date
    pub service_date: NaiveDate,
    /// Procedure code (CPT, HCPCS, or revenue code)
    pub procedure_code: String,
    /// Procedure modifiers
    pub modifiers: Vec<String>,
    /// Diagnosis code pointers
    pub diagnosis_pointers: Vec<u8>,
    /// Units of service
    pub units: f64,
    /// Unit type (e.g., "UN", "MJ")
    pub unit_type: String,
    /// Charge amount
    pub charge: f64,
    /// Place of service code
    pub place_of_service: String,
}

/// Claim status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimStatus {
    /// Original claim
    Original,
    /// Replacement claim
    Replacement,
    /// Void/cancel claim
    Void,
}

/// Parse X12 837 content into a Claim structure
///
/// # Arguments
///
/// * `content` - The X12 837 EDI content as a string
///
/// # Returns
///
/// * `Ok(Claim)` - Successfully parsed claim
/// * `Err(Claims837Error)` - Parse error with details
///
/// # Example
///
/// ```
/// use claims_837::parse_837;
///
/// let minimal = r#"ISA*00*          *00*          *ZZ*SUBMITTER     *ZZ*RECEIVER       *240101*1200*^*00501*000000001*0*P*:~
/// GS*HC*SUBMITTER*RECEIVER*20240101*1200*1*X*005010X222A1~
/// ST*837*0001*005010X222A1~
/// BHT*0019*00*1*20240101*1200*CH~
/// NM1*41*2*PROVIDER*****XX*1234567890~
/// PER*IC*CONTACT*TE*5551234567~
/// NM1*40*2*PAYER*****46*PAYERID~
/// HL*1**20*1~
/// PRV*BI*PXC*207Q00000X~
/// NM1*85*2*PROVIDER*JOHN****XX*1234567890~
/// N3*123 MAIN ST~
/// N4*ANYTOWN*ST*12345~
/// REF*EI*123456789~
/// HL*2*1*22*0~
/// SBR*P*18*GROUP123**CI***MB~
/// NM1*IL*1*DOE*JANE****MI*MEMBER123~
/// DMG*D8*19800101*F~
/// NM1*PR*2*PAYER*****PI*PAYERID~
/// CLM*CLAIM123*100***11:B:1*Y*A*Y*Y~
/// DTP*472*D8*20240101~
/// REF*D9*CLAIMREF123~
/// HI*BK:J44.1~
/// LX*1~
/// SV1*HC:99213*100***1***1~
/// DTP*472*D8*20240101~
/// SE*25*0001~
/// GE*1*1~
/// IEA*1*000000001~"#;
///
/// // Note: This is a simplified example; real parsing would handle the full X12 structure
/// ```
pub fn parse_837(content: &str) -> Result<Claim, Claims837Error> {
    if content.trim().is_empty() {
        return Err(Claims837Error::EmptyInput);
    }

    // Validate X12 envelope
    if !content.starts_with("ISA") {
        return Err(Claims837Error::InvalidFormat(
            "Missing ISA segment - not valid X12".to_string(),
        ));
    }

    // TODO: Full X12 837 parsing implementation
    // For now, return a placeholder error indicating the parser is being developed
    Err(Claims837Error::InvalidFormat(
        "Full 837 parser implementation in progress. See GitHub issues for status.".to_string(),
    ))
}

/// Detect claim type from X12 content
pub fn detect_claim_type(content: &str) -> Option<ClaimType> {
    if content.contains("005010X222") {
        Some(ClaimType::Professional)
    } else if content.contains("005010X223") {
        Some(ClaimType::Institutional)
    } else if content.contains("005010X224") {
        Some(ClaimType::Dental)
    } else {
        None
    }
}

/// Validate a claim structure
pub fn validate_claim(claim: &Claim) -> Vec<String> {
    let mut errors = Vec::new();

    if claim.claim_id.is_empty() {
        errors.push("Claim ID is required".to_string());
    }

    if claim.service_lines.is_empty() {
        errors.push("At least one service line is required".to_string());
    }

    if claim.total_charge <= 0.0 {
        errors.push("Total charge must be greater than zero".to_string());
    }

    if claim.diagnosis_codes.is_empty() {
        errors.push("At least one diagnosis code is required".to_string());
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = parse_837("");
        assert!(matches!(result, Err(Claims837Error::EmptyInput)));
    }

    #[test]
    fn test_missing_isa() {
        let result = parse_837("GS*HC*...");
        assert!(matches!(
            result,
            Err(Claims837Error::InvalidFormat(_))
        ));
    }

    #[test]
    fn test_detect_professional() {
        let content = r#"ISA*00*...~GS*HC*...~ST*837*...~...~005010X222A1~..."#;
        assert_eq!(detect_claim_type(content), Some(ClaimType::Professional));
    }

    #[test]
    fn test_detect_institutional() {
        let content = r#"ISA*00*...~GS*HC*...~ST*837*...~...~005010X223A2~..."#;
        assert_eq!(detect_claim_type(content), Some(ClaimType::Institutional));
    }

    #[test]
    fn test_validate_claim_ok() {
        let claim = Claim {
            claim_id: "TEST001".to_string(),
            claim_type: ClaimType::Professional,
            patient: PatientInfo {
                last_name: "Doe".to_string(),
                first_name: "John".to_string(),
                middle_name: None,
                birth_date: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
                gender: Gender::Male,
                member_id: "MEM123".to_string(),
                address: Address::default(),
            },
            subscriber: SubscriberInfo {
                last_name: "Doe".to_string(),
                first_name: "John".to_string(),
                member_id: "MEM123".to_string(),
                group_number: None,
                payer_id: "PAYER1".to_string(),
                relationship: Relationship::Self_Patient,
            },
            provider: ProviderInfo {
                npi: "1234567890".to_string(),
                taxonomy: None,
                name: "Provider".to_string(),
                address: Address::default(),
                tax_id: None,
            },
            payer: PayerInfo {
                name: "Insurance".to_string(),
                payer_id: "PAYER1".to_string(),
                address: None,
            },
            dates: ClaimDates {
                service_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                service_end_date: None,
                submission_date: None,
            },
            service_lines: vec![ServiceLine {
                line_number: 1,
                service_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                procedure_code: "99213".to_string(),
                modifiers: vec![],
                diagnosis_pointers: vec![1],
                units: 1.0,
                unit_type: "UN".to_string(),
                charge: 100.0,
                place_of_service: "11".to_string(),
            }],
            diagnosis_codes: vec!["J44.1".to_string()],
            total_charge: 100.0,
            status: ClaimStatus::Original,
        };

        let errors = validate_claim(&claim);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_claim_errors() {
        let claim = Claim {
            claim_id: "".to_string(),
            claim_type: ClaimType::Professional,
            patient: PatientInfo {
                last_name: "Doe".to_string(),
                first_name: "John".to_string(),
                middle_name: None,
                birth_date: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
                gender: Gender::Male,
                member_id: "MEM123".to_string(),
                address: Address::default(),
            },
            subscriber: SubscriberInfo {
                last_name: "Doe".to_string(),
                first_name: "John".to_string(),
                member_id: "MEM123".to_string(),
                group_number: None,
                payer_id: "PAYER1".to_string(),
                relationship: Relationship::Self_Patient,
            },
            provider: ProviderInfo {
                npi: "1234567890".to_string(),
                taxonomy: None,
                name: "Provider".to_string(),
                address: Address::default(),
                tax_id: None,
            },
            payer: PayerInfo {
                name: "Insurance".to_string(),
                payer_id: "PAYER1".to_string(),
                address: None,
            },
            dates: ClaimDates {
                service_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                service_end_date: None,
                submission_date: None,
            },
            service_lines: vec![],
            diagnosis_codes: vec![],
            total_charge: 0.0,
            status: ClaimStatus::Original,
        };

        let errors = validate_claim(&claim);
        assert_eq!(errors.len(), 4);
        assert!(errors.iter().any(|e| e.contains("Claim ID")));
        assert!(errors.iter().any(|e| e.contains("service line")));
        assert!(errors.iter().any(|e| e.contains("Total charge")));
        assert!(errors.iter().any(|e| e.contains("diagnosis")));
    }
}

//! US medical code databases for healthcare claims processing.
//!
//! This crate provides lookup databases for:
//! - **CPT** (Current Procedural Terminology) codes
//! - **ICD-10-CM** (International Classification of Diseases) diagnosis codes
//! - **CARC/RARC** (Claim Adjustment Reason Codes / Remittance Advice Remark Codes)
//! - **CPT/HCPCS Modifiers**

pub mod carc;
pub mod cpt;
pub mod icd10_cm;
pub mod modifiers;

// Re-export key types for convenience
pub use carc::{CarcCategory, CarcCode, RarcCode, CARC_DATABASE, RARC_DATABASE};
pub use cpt::{CptCategory, CptCode, CPT_DATABASE};
pub use icd10_cm::{Icd10Category, Icd10Code, ICD10_DATABASE};
pub use modifiers::{CptModifier, MODIFIER_DATABASE};

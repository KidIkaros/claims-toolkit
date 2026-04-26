//! PHI Scanner - Detects and redacts Protected Health Information.
//!
//! This crate provides HIPAA Safe Harbor compliant PHI detection for 18 identifier
//! categories including SSN, names, dates, phone numbers, medical record numbers,
//! and healthcare codes (CPT, ICD-10).
//!
//! # Quick Start
//!
//! ```
//! use phi_scan::{scan_phi, redact_phi};
//!
//! // Scan text for PHI
//! let result = scan_phi("Patient: John Smith, SSN: 123-45-6789");
//! assert!(result.contains_phi);
//!
//! // Redact PHI
//! let redacted = redact_phi("Patient: John Smith, SSN: 123-45-6789");
//! // Result: "Patient: [REDACTED:Name], SSN: [REDACTED:SSN]"
//! ```

pub mod taint_router;

pub use taint_router::{scan_phi, redact_phi, PhiCategory, PhiScanResult, PhiDetection};

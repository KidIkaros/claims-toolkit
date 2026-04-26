//! PHI detection using regex patterns for all 18 HIPAA Safe Harbor identifiers.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiScanResult {
    pub contains_phi: bool,
    pub detections: Vec<PhiDetection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiDetection {
    pub category: PhiCategory,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhiCategory {
    Name, Geographic, Date, Phone, Fax, Email, Ssn, Mrn,
    HealthPlanId, AccountNumber, CertificateNumber, VehicleId,
    DeviceId, WebUrl, IpAddress, Biometric, FacialImage,
    UniqueId, CptCode, Icd10Code,
}

impl std::fmt::Display for PhiCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Name => "Name", Self::Geographic => "Geographic",
            Self::Date => "Date", Self::Phone => "Phone", Self::Fax => "Fax",
            Self::Email => "Email", Self::Ssn => "SSN", Self::Mrn => "MRN",
            Self::HealthPlanId => "HealthPlanID", Self::AccountNumber => "AccountNumber",
            Self::CertificateNumber => "CertificateNumber", Self::VehicleId => "VehicleID",
            Self::DeviceId => "DeviceID", Self::WebUrl => "WebURL",
            Self::IpAddress => "IPAddress", Self::Biometric => "Biometric",
            Self::FacialImage => "FacialImage", Self::UniqueId => "UniqueID",
            Self::CptCode => "CPT", Self::Icd10Code => "ICD-10",
        };
        write!(f, "{}", s)
    }
}

struct Patterns {
    name: Regex, geographic: Regex, date: Regex, phone: Regex, fax: Regex,
    email: Regex, ssn: Regex, mrn: Regex, health_plan: Regex,
    account_number: Regex, certificate: Regex, vehicle_id: Regex,
    device_id: Regex, web_url: Regex, ip_address: Regex,
    biometric: Regex, facial_image: Regex, unique_id: Regex,
    cpt: Regex, icd10: Regex,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        // All patterns are static string literals that are validated at compile time.
        // These expect calls document the invariant that makes them infallible.
        Patterns {
            name: Regex::new(r"(?i)\b(?:patient|name|pt\.?|dr\.?|doctor|nurse|provider|physician|mr\.?|mrs\.?|ms\.?|miss)\s*[:=]\s*[A-Z][a-z]+(?:\s+[A-Z][a-z]+){1,3}\b").expect("static name pattern is valid regex"),
            geographic: Regex::new(r"(?i)\b\d{1,5}\s+[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*(?:\s+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Drive|Dr|Lane|Ln|Court|Ct|Place|Pl|Way|Circle|Cir))\b|\b(?:ZIP|zip\s*code|postal|address)\s*[:=]?\s*\d{5}(?:-\d{4})?\b").expect("static geographic pattern is valid regex"),
            date: Regex::new(r"(?i)\b(?:DOB|date\s+of\sbirth|birth\s+date|admission|discharge|admitted|born)\s*[:=]?\s*\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4}\b|\b\d{1,2}[/\-]\d{1,2}[/\-]\d{4}\b").expect("static date pattern is valid regex"),
            phone: Regex::new(r"\(\d{3}\)\s*\d{3}[-.]\d{4}|\b\d{3}[-.]\d{3}[-.]\d{4}\b").expect("static phone pattern is valid regex"),
            fax: Regex::new(r"(?i)\bfax\s*[:=]?\s*(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b").expect("static fax pattern is valid regex"),
            email: Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b").expect("static email pattern is valid regex"),
            ssn: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").expect("static ssn pattern is valid regex"),
            mrn: Regex::new(r"(?i)\b(?:MRN|medical\s+record|patient\s+id)\s*[:=]?\s*[A-Z0-9]{4,12}\b").expect("static mrn pattern is valid regex"),
            health_plan: Regex::new(r"(?i)\b(?:insurance|plan|member|policy)\s*(?:id|number|#)?\s*[:=]?\s*[A-Z0-9\-]{5,20}\b").expect("static health_plan pattern is valid regex"),
            account_number: Regex::new(r"(?i)\b(?:account|acct|billing)\s*(?:number|#|no\.?)?\s*[:=]?\s*[A-Z0-9\-]{5,20}\b").expect("static account_number pattern is valid regex"),
            certificate: Regex::new(r"(?i)\b(?:DEA|certificate|license|permit)\s*(?:number|#|no\.?)?\s*[:=]?\s*[A-Z0-9]{5,12}\b").expect("static certificate pattern is valid regex"),
            vehicle_id: Regex::new(r"(?i)\b(?:VIN|vehicle\s+id)\s*[:=]?\s*[A-HJ-NPR-Z0-9]{17}\b").expect("static vehicle_id pattern is valid regex"),
            device_id: Regex::new(r"(?i)\b(?:UDI|device\s+id|serial)\s*[:=]?\s*[A-Z0-9\-]{8,20}\b").expect("static device_id pattern is valid regex"),
            web_url: Regex::new(r"https?://[^\s<>)\x22']+").expect("static web_url pattern is valid regex"),
            ip_address: Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").expect("static ip_address pattern is valid regex"),
            biometric: Regex::new(r"(?i)\b(?:fingerprint|retinal|iris\s+scan|voiceprint|biometric|face\s*(?:id|recognition)|dna\s+profile|palm\s+print)\b").expect("static biometric pattern is valid regex"),
            facial_image: Regex::new(r"(?i)\b(?:photo(?:graph)?|image|picture|portrait|selfie)\s*(?:of|attached|included|showing)\s*(?:patient|resident|individual|person|pt\.?)\b").expect("static facial_image pattern is valid regex"),
            unique_id: Regex::new(r"\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b").expect("static unique_id pattern is valid regex"),
            cpt: Regex::new(r"\b(?:CPT|cpt)\s*[-:]?\s*\d{5}\b").expect("static cpt pattern is valid regex"),
            icd10: Regex::new(r"\b[A-TV-Z]\d{2,3}(?:\.\d{1,4})?\b").expect("static icd10 pattern is valid regex"),
        }
    })
}

/// Case-insensitive check if `text` ends with `suffix`.
/// Avoids allocating a lowercase string for the entire prefix.
fn ends_with_ignore_case(text: &str, suffix: &str) -> bool {
    if text.len() < suffix.len() {
        return false;
    }
    let start = text.len() - suffix.len();
    text[start..].eq_ignore_ascii_case(suffix)
}

/// Check if any of the suffixes match (case-insensitive) after trimming whitespace.
fn ends_with_any_trimmed(text: &str, suffixes: &[&str]) -> bool {
    let trimmed = text.trim_end();
    suffixes.iter().any(|&s| ends_with_ignore_case(trimmed, s))
}

pub fn scan_phi(text: &str) -> PhiScanResult {
    let p = patterns();
    let mut d = Vec::new();
    // Pre-allocate with estimated capacity based on typical PHI density
    d.reserve(text.len() / 100);

    for m in p.name.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Name, span: (m.start(), m.end()) }); }
    for m in p.geographic.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Geographic, span: (m.start(), m.end()) }); }
    for m in p.date.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Date, span: (m.start(), m.end()) }); }
    for m in p.phone.find_iter(text) {
        let before = &text[..m.start()];

        // Skip if this is an NPI number (not PHI) — using case-insensitive check without allocation
        const NPI_PREFIXES: &[&str] = &["npi", "provider", "national provider", "provider npi"];
        if ends_with_any_trimmed(before, NPI_PREFIXES) {
            continue;
        }

        // Skip bare 10-digit numbers without phone formatting
        let matched = &text[m.start()..m.end()];
        let has_formatting = matched.contains('(')
            || matched.contains(')')
            || matched.contains('-')
            || matched.contains('.')
            || matched.starts_with('+');
        if !has_formatting && matched.len() == 10 && matched.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let cat = if ends_with_ignore_case(before.trim_end(), "fax") {
            PhiCategory::Fax
        } else {
            PhiCategory::Phone
        };
        d.push(PhiDetection {
            category: cat,
            span: (m.start(), m.end()),
        });
    }
    for m in p.fax.find_iter(text) {
        let span = (m.start(), m.end());
        if !d.iter().any(|x| x.span == span) { d.push(PhiDetection { category: PhiCategory::Fax, span }); }
    }
    for m in p.email.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Email, span: (m.start(), m.end()) }); }
    for m in p.ssn.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Ssn, span: (m.start(), m.end()) }); }
    for m in p.mrn.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Mrn, span: (m.start(), m.end()) }); }
    for m in p.health_plan.find_iter(text) { d.push(PhiDetection { category: PhiCategory::HealthPlanId, span: (m.start(), m.end()) }); }
    for m in p.account_number.find_iter(text) { d.push(PhiDetection { category: PhiCategory::AccountNumber, span: (m.start(), m.end()) }); }
    for m in p.certificate.find_iter(text) { d.push(PhiDetection { category: PhiCategory::CertificateNumber, span: (m.start(), m.end()) }); }
    for m in p.vehicle_id.find_iter(text) { d.push(PhiDetection { category: PhiCategory::VehicleId, span: (m.start(), m.end()) }); }
    for m in p.device_id.find_iter(text) { d.push(PhiDetection { category: PhiCategory::DeviceId, span: (m.start(), m.end()) }); }
    for m in p.web_url.find_iter(text) { d.push(PhiDetection { category: PhiCategory::WebUrl, span: (m.start(), m.end()) }); }
    for m in p.ip_address.find_iter(text) { d.push(PhiDetection { category: PhiCategory::IpAddress, span: (m.start(), m.end()) }); }
    for m in p.biometric.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Biometric, span: (m.start(), m.end()) }); }
    for m in p.facial_image.find_iter(text) { d.push(PhiDetection { category: PhiCategory::FacialImage, span: (m.start(), m.end()) }); }
    for m in p.unique_id.find_iter(text) { d.push(PhiDetection { category: PhiCategory::UniqueId, span: (m.start(), m.end()) }); }
    for m in p.cpt.find_iter(text) { d.push(PhiDetection { category: PhiCategory::CptCode, span: (m.start(), m.end()) }); }
    for m in p.icd10.find_iter(text) { d.push(PhiDetection { category: PhiCategory::Icd10Code, span: (m.start(), m.end()) }); }

    PhiScanResult { contains_phi: !d.is_empty(), detections: d }
}

pub fn redact_phi(text: &str) -> String {
    let result = scan_phi(text);
    if !result.contains_phi { return text.to_string(); }
    let mut detections = result.detections;
    detections.sort_by(|a, b| b.span.0.cmp(&a.span.0));
    let mut redacted = text.to_string();
    for det in &detections {
        redacted.replace_range(det.span.0..det.span.1, &format!("[REDACTED:{}]", det.category));
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssn() {
        assert!(scan_phi("SSN 123-45-6789").detections.iter().any(|d| d.category == PhiCategory::Ssn));
    }
    #[test]
    fn test_phone() {
        assert!(scan_phi("(555) 123-4567").detections.iter().any(|d| d.category == PhiCategory::Phone));
    }
    #[test]
    fn test_email() {
        assert!(scan_phi("john@test.com").detections.iter().any(|d| d.category == PhiCategory::Email));
    }
    #[test]
    fn test_name() {
        assert!(scan_phi("Patient: John Smith").detections.iter().any(|d| d.category == PhiCategory::Name));
    }
    #[test]
    fn test_redact() {
        let r = redact_phi("SSN 123-45-6789");
        assert!(r.contains("[REDACTED:"));
        assert!(!r.contains("123-45-6789"));
    }
}

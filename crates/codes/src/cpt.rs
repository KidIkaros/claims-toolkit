//! CPT (Current Procedural Terminology) codes
//!
//! CPT codes are used to report medical, surgical, and diagnostic procedures.

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// CPT code categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CptCategory {
    /// Evaluation and Management (99201-99499)
    EvaluationManagement,
    /// Anesthesia (00100-01999)
    Anesthesia,
    /// Surgery (10004-69990) - further subdivided by body system
    Surgery,
    /// Radiology (70010-79999)
    Radiology,
    /// Pathology and Laboratory (80002-89356)
    PathologyLaboratory,
    /// Medicine (90281-99199) - immunizations, psychiatry, dialysis, etc.
    Medicine,
    /// Category II codes (performance measurement)
    CategoryII,
    /// Category III codes (emerging technology)
    CategoryIII,
    /// Proprietary codes (telehealth, remote monitoring)
    Proprietary,
    /// Modifiers only
    Modifier,
}

impl CptCategory {
    /// Get the range prefix for this category
    pub fn range_prefix(&self) -> &'static str {
        match self {
            Self::EvaluationManagement => "99",
            Self::Anesthesia => "00-01",
            Self::Surgery => "1-6",
            Self::Radiology => "7",
            Self::PathologyLaboratory => "8",
            Self::Medicine => "9",
            Self::CategoryII => "0",
            Self::CategoryIII => "0",
            Self::Proprietary => "9",
            Self::Modifier => "modifier",
        }
    }
}

impl std::fmt::Display for CptCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// CPT code information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CptCode {
    /// The 5-digit CPT code
    pub code: &'static str,

    /// Full description
    pub description: &'static str,

    /// Category
    pub category: CptCategory,

    /// Whether this code requires a modifier
    pub requires_modifier: bool,

    /// Typical RVU (Relative Value Unit) - approximate
    pub rvu: f32,

    /// Typical work RVU portion
    pub work_rvu: f32,

    /// Whether this is a telehealth-eligible code
    pub telehealth_eligible: bool,

    /// Common modifiers for this code
    pub common_modifiers: Vec<&'static str>,
}

impl CptCode {
    /// Check if this CPT code is typically billable
    pub fn is_billable(&self) -> bool {
        !matches!(
            self.category,
            CptCategory::CategoryII | CptCategory::CategoryIII
        )
    }

    /// Get the global surgical days for this code (0 = none, 10 = 90-day, etc.)
    pub fn global_period(&self) -> u8 {
        // Evaluation & Management typically has 0 global days
        if matches!(self.category, CptCategory::EvaluationManagement) {
            return 0;
        }

        // Major surgeries typically have 10 (90-day) global period
        match &self.code[0..2] {
            "10" | "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18" | "19" => 10, // Integumentary
            "20" | "21" => 10,                      // Respiratory
            "27" => 10,                             // Eye
            "29" => 0,                              // Auditory (usually 0)
            "30" | "31" => 10,                      // Cardiovascular
            "32" | "33" | "34" | "35" | "36" => 10, // Hemic/Lymphatic, Mediastinum, Digestive, Urinary, Male Genital
            "37" | "38" => 10,                      // Female Genital, Maternity
            "39" => 0,                              // Delivery (complex)
            "40" | "41" | "42" | "43" | "44" | "45" | "46" | "47" | "48" | "49" => 10, // Digestive to Musculoskeletal
            "50" | "51" | "52" | "53" | "54" => 10, // Nervous, Spine, Integumentary, Breast, Musculoskeletal
            "58" => 10,                             // Male Genital (reproductive)
            "60" | "61" => 10,                      // Urinary
            _ => 0,
        }
    }

    /// Check if this code requires documentation of time
    pub fn requires_time_documentation(&self) -> bool {
        matches!(self.category, CptCategory::EvaluationManagement)
            || self.description.contains("prolonged service")
            || self.description.contains("prolonged")
    }
}

/// CPT code database
pub static CPT_DATABASE: Lazy<FxHashMap<&'static str, CptCode>> = Lazy::new(|| {
    let mut m = FxHashMap::default();

    // Evaluation and Management (E&M) Codes
    m.insert("99202", CptCode {
        code: "99202",
        description: "Office or other outpatient visit for the evaluation and management of a new patient, requires a medically appropriate history and/or examination and straightforward medical decision making. When using total time on the date of the encounter for code selection, 15-29 minutes must be spent in the face-to-face encounter.",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 0.93,
        work_rvu: 0.70,
        telehealth_eligible: true,
        common_modifiers: vec!["25", "95"],
    });
    m.insert("99203", CptCode {
        code: "99203",
        description: "Office or other outpatient visit for the evaluation and management of a new patient, requires a medically appropriate history and/or examination and low level of medical decision making. When using total time on the date of the encounter for code selection, 30-44 minutes must be spent in the face-to-face encounter.",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.42,
        work_rvu: 1.08,
        telehealth_eligible: true,
        common_modifiers: vec!["25", "95"],
    });
    m.insert("99212", CptCode {
        code: "99212",
        description: "Office or other outpatient visit for the evaluation and management of an established patient, requires a medically appropriate history and/or examination and straightforward medical decision making. When using total time, 10-19 minutes total time.",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 0.48,
        work_rvu: 0.37,
        telehealth_eligible: true,
        common_modifiers: vec!["25", "95"],
    });
    m.insert("99213", CptCode {
        code: "99213",
        description: "Office or other outpatient visit for the evaluation and management of an established patient, requires a medically appropriate history and/or examination and low level of medical decision making. When using total time, 20-29 minutes total time.",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 0.78,
        work_rvu: 0.58,
        telehealth_eligible: true,
        common_modifiers: vec!["25", "95"],
    });
    m.insert("99214", CptCode {
        code: "99214",
        description: "Office or other outpatient visit for the evaluation and management of an established patient, requires a medically appropriate history and/or examination and moderate level of medical decision making. When using total time, 30-39 minutes total time.",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.35,
        work_rvu: 1.01,
        telehealth_eligible: true,
        common_modifiers: vec!["25", "95"],
    });
    m.insert("99215", CptCode {
        code: "99215",
        description: "Office or other outpatient visit for the evaluation and management of an established patient, requires a medically appropriate history and/or examination and high level of medical decision making. When using total time, 40-54 minutes total time.",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 2.11,
        work_rvu: 1.58,
        telehealth_eligible: true,
        common_modifiers: vec!["25", "95"],
    });

    // Radiology
    m.insert(
        "71010",
        CptCode {
            code: "71010",
            description: "Radiologic examination, chest; single view, frontal",
            category: CptCategory::Radiology,
            requires_modifier: false,
            rvu: 0.10,
            work_rvu: 0.09,
            telehealth_eligible: false,
            common_modifiers: vec!["26"],
        },
    );
    m.insert(
        "71020",
        CptCode {
            code: "71020",
            description: "Radiologic examination, chest, 2 views, frontal and lateral",
            category: CptCategory::Radiology,
            requires_modifier: false,
            rvu: 0.13,
            work_rvu: 0.10,
            telehealth_eligible: false,
            common_modifiers: vec!["26"],
        },
    );
    m.insert(
        "70450",
        CptCode {
            code: "70450",
            description: "Computed tomography, head or brain; without contrast material",
            category: CptCategory::Radiology,
            requires_modifier: false,
            rvu: 1.25,
            work_rvu: 0.92,
            telehealth_eligible: false,
            common_modifiers: vec!["26"],
        },
    );
    m.insert(
        "70460",
        CptCode {
            code: "70460",
            description: "Computed tomography, head or brain; with contrast material(s)",
            category: CptCategory::Radiology,
            requires_modifier: false,
            rvu: 1.73,
            work_rvu: 1.27,
            telehealth_eligible: false,
            common_modifiers: vec!["26"],
        },
    );
    m.insert(
        "71250",
        CptCode {
            code: "71250",
            description: "Computed tomography, thorax; without contrast material",
            category: CptCategory::Radiology,
            requires_modifier: false,
            rvu: 1.43,
            work_rvu: 1.05,
            telehealth_eligible: false,
            common_modifiers: vec!["26"],
        },
    );

    // Surgery
    m.insert(
        "12001",
        CptCode {
            code: "12001",
            description: "Incision and removal of foreign body, skin; subcutaneous tissues",
            category: CptCategory::Surgery,
            requires_modifier: false,
            rvu: 1.14,
            work_rvu: 0.85,
            telehealth_eligible: false,
            common_modifiers: vec!["50", "51", "59"],
        },
    );
    m.insert("11400", CptCode {
        code: "11400",
        description: "Excision, benign lesion including margins, except skin tag (unless listed elsewhere), trunk, arms or legs; excised diameter 0.5 cm or less",
        category: CptCategory::Surgery,
        requires_modifier: false,
        rvu: 0.47,
        work_rvu: 0.36,
        telehealth_eligible: false,
        common_modifiers: vec!["50", "51", "59"],
    });
    m.insert("11401", CptCode {
        code: "11401",
        description: "Excision, benign lesion including margins, except skin tag (unless listed elsewhere), trunk, arms or legs; excised diameter 0.6 to 1.0 cm",
        category: CptCategory::Surgery,
        requires_modifier: false,
        rvu: 0.54,
        work_rvu: 0.41,
        telehealth_eligible: false,
        common_modifiers: vec!["50", "51", "59"],
    });
    m.insert("11402", CptCode {
        code: "11402",
        description: "Excision, benign lesion including margins, except skin tag (unless listed elsewhere), trunk, arms or legs; excised diameter 1.1 to 2.0 cm",
        category: CptCategory::Surgery,
        requires_modifier: false,
        rvu: 0.83,
        work_rvu: 0.63,
        telehealth_eligible: false,
        common_modifiers: vec!["50", "51", "59"],
    });

    // Laboratory
    m.insert("80053", CptCode {
        code: "80053",
        description: "Urinalysis, by dip stick or tablet reagent for bilirubin, glucose, hemoglobin, ketones, pH, protein, specific gravity, urobilinogen, any number of these constituents; manual",
        category: CptCategory::PathologyLaboratory,
        requires_modifier: false,
        rvu: 0.35,
        work_rvu: 0.35,
        telehealth_eligible: false,
        common_modifiers: vec!["91"],
    });
    m.insert(
        "83036",
        CptCode {
            code: "83036",
            description: "Hemoglobin; glycosylated (A1c)",
            category: CptCategory::PathologyLaboratory,
            requires_modifier: false,
            rvu: 0.48,
            work_rvu: 0.48,
            telehealth_eligible: false,
            common_modifiers: vec!["91"],
        },
    );

    // Medicine
    m.insert("93000", CptCode {
        code: "93000",
        description: "Electrocardiogram, routine ECG with at least 12 leads; with interpretation and report",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.28,
        work_rvu: 0.21,
        telehealth_eligible: false,
        common_modifiers: vec!["26", "52"],
    });
    m.insert("93010", CptCode {
        code: "93010",
        description: "Electrocardiogram, routine ECG with at least 12 leads; interpretation and report only",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.18,
        work_rvu: 0.13,
        telehealth_eligible: true,
        common_modifiers: vec!["26"],
    });

    // ============================================================================
    // Preventive Medicine Codes (99381-99397)
    // ============================================================================

    // Initial Preventive Medicine (New Patient)
    m.insert("99381", CptCode {
        code: "99381",
        description: "Initial comprehensive preventive medicine evaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, new patient; infant (age under 1 year)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.69,
        work_rvu: 1.39,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99382", CptCode {
        code: "99382",
        description: "Initial comprehensive preventive medicine evaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, new patient; early childhood (age 1 through 4 years)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.93,
        work_rvu: 1.59,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99383", CptCode {
        code: "99383",
        description: "Initial comprehensive preventive medicine evaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, new patient; late childhood (age 5 through 11 years)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 2.15,
        work_rvu: 1.76,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99384", CptCode {
        code: "99384",
        description: "Initial comprehensive preventive medicine evaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, new patient; adolescent (age 12 through 17 years)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 2.31,
        work_rvu: 1.88,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99385", CptCode {
        code: "99385",
        description: "Initial comprehensive preventive medicine evaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, new patient; 18-39 years",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 2.54,
        work_rvu: 2.07,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99386", CptCode {
        code: "99386",
        description: "Initial comprehensive preventive medicine evaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, new patient; 40-64 years",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 2.87,
        work_rvu: 2.32,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99387", CptCode {
        code: "99387",
        description: "Initial comprehensive preventive medicine evaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, new patient; 65 years and older",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 3.38,
        work_rvu: 2.72,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });

    // Periodic Preventive Medicine (Established Patient)
    m.insert("99391", CptCode {
        code: "99391",
        description: "Periodic comprehensive preventive medicine reevaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, established patient; infant (age under 1 year)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.32,
        work_rvu: 1.10,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99392", CptCode {
        code: "99392",
        description: "Periodic comprehensive preventive medicine reevaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, established patient; early childhood (age 1 through 4 years)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.50,
        work_rvu: 1.24,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99393", CptCode {
        code: "99393",
        description: "Periodic comprehensive preventive medicine reevaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, established patient; late childhood (age 5 through 11 years)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.67,
        work_rvu: 1.36,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99394", CptCode {
        code: "99394",
        description: "Periodic comprehensive preventive medicine reevaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, established patient; adolescent (age 12 through 17 years)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.79,
        work_rvu: 1.45,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99395", CptCode {
        code: "99395",
        description: "Periodic comprehensive preventive medicine reevaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, established patient; 18-39 years",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 1.97,
        work_rvu: 1.59,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99396", CptCode {
        code: "99396",
        description: "Periodic comprehensive preventive medicine reevaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, established patient; 40-64 years",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 2.22,
        work_rvu: 1.78,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99397", CptCode {
        code: "99397",
        description: "Periodic comprehensive preventive medicine reevaluation and management of an individual including an age and gender appropriate history, examination, counseling/anticipatory guidance/risk factor reduction interventions, and the ordering of laboratory/diagnostic procedures, established patient; 65 years and older",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 2.62,
        work_rvu: 2.08,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });

    // ============================================================================
    // Medicare G-Codes (Preventive Services)
    // ============================================================================

    m.insert("G0402", CptCode {
        code: "G0402",
        description: "Initial preventive physical examination; face-to-face visit, services limited to new beneficiary during the first 12 months of Medicare enrollment",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 1.73,
        work_rvu: 1.38,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("G0438", CptCode {
        code: "G0438",
        description: "Annual wellness visit; includes a personalized prevention plan of service (PPPS), initial visit",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 1.50,
        work_rvu: 1.13,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("G0439", CptCode {
        code: "G0439",
        description: "Annual wellness visit; includes a personalized prevention plan of service (PPPS), subsequent visit",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 1.19,
        work_rvu: 0.89,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    // Smoking and Tobacco Cessation
    m.insert("99406", CptCode {
        code: "99406",
        description: "Smoking and tobacco-use cessation counseling visit; intermediate, greater than 3 minutes up to 10 minutes",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.17,
        work_rvu: 0.14,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("99407", CptCode {
        code: "99407",
        description: "Smoking and tobacco-use cessation counseling visit; intensive, greater than 10 minutes",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.35,
        work_rvu: 0.29,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    // Alcohol and Substance Abuse Screening
    m.insert("G0425", CptCode {
        code: "G0425",
        description: "Structured screening assessment for depression",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 0.16,
        work_rvu: 0.13,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("G0396", CptCode {
        code: "G0396",
        description: "Alcohol and/or substance (drug) abuse screening (e.g., AUDIT, ASSIST), brief intervention (5-15 minutes), and motivation interview for substance use disorder (SUD) as applicable",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 0.29,
        work_rvu: 0.23,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("H0049", CptCode {
        code: "H0049",
        description: "Alcohol and/or substance (drug) abuse structured screening and brief intervention (SBI) services",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 0.29,
        work_rvu: 0.23,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    // Advance Care Planning
    m.insert("99497", CptCode {
        code: "99497",
        description: "Advance care planning including the explanation and discussion of advance directives such as standard forms (with completion of such forms, if desired), by physician or other qualified health care professional; first 30 minutes, face-to-face with patient, family member(s), and/or surrogate",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 0.82,
        work_rvu: 0.72,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("99498", CptCode {
        code: "99498",
        description: "Advance care planning including the explanation and discussion of advance directives such as standard forms (with completion of such forms, if desired), by physician or other qualified health care professional; each additional 30 minutes (List separately in addition to code for primary procedure)",
        category: CptCategory::EvaluationManagement,
        requires_modifier: false,
        rvu: 0.68,
        work_rvu: 0.60,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    // Chronic Care Management
    m.insert("99490", CptCode {
        code: "99490",
        description: "Chronic care management services, at least 20 minutes of clinical staff time directed by a physician or other qualified health care professional, per calendar month",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.67,
        work_rvu: 0.0,  // Clinical staff time
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("99487", CptCode {
        code: "99487",
        description: "Complex chronic care management services, at least 60 minutes of clinical staff time directed by a physician or other qualified health care professional, per calendar month",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 1.02,
        work_rvu: 0.0,  // Clinical staff time
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("99489", CptCode {
        code: "99489",
        description: "Complex chronic care management services, each additional 30 minutes of clinical staff time directed by a physician or other qualified health care professional, per calendar month (List separately in addition to code for primary procedure)",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.34,
        work_rvu: 0.0,  // Clinical staff time
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    // Principal Care Management
    m.insert("99424", CptCode {
        code: "99424",
        description: "Principal care management services, at least 30 minutes of physician or other qualified health care professional time, per calendar month",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.75,
        work_rvu: 0.75,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    // Remote Physiologic Monitoring
    m.insert("99453", CptCode {
        code: "99453",
        description: "Remote physiologic monitoring treatment management services, initial setup and patient education on use of equipment",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.38,
        work_rvu: 0.19,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99454", CptCode {
        code: "99454",
        description: "Remote physiologic monitoring treatment management services, device(s) supply with daily recording(s) or programmed alert(s) transmission, each 30 days",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.38,
        work_rvu: 0.19,
        telehealth_eligible: false,
        common_modifiers: vec![],
    });
    m.insert("99457", CptCode {
        code: "99457",
        description: "Remote physiologic monitoring treatment management services, 20 minutes of clinical staff/physician time in a calendar month",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.49,
        work_rvu: 0.0,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("99458", CptCode {
        code: "99458",
        description: "Remote physiologic monitoring treatment management services, each additional 20 minutes of clinical staff/physician time in a calendar month",
        category: CptCategory::Medicine,
        requires_modifier: false,
        rvu: 0.19,
        work_rvu: 0.0,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    // Other Proprietary Codes
    m.insert("G2061", CptCode {
        code: "G2061",
        description: "Medication therapy management; 15 minutes",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 0.63,
        work_rvu: 0.47,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });
    m.insert("G2062", CptCode {
        code: "G2062",
        description: "Medication therapy management; each additional 15 minutes",
        category: CptCategory::Proprietary,
        requires_modifier: false,
        rvu: 0.31,
        work_rvu: 0.23,
        telehealth_eligible: true,
        common_modifiers: vec![],
    });

    m
});

/// Find a CPT code by its code string
pub fn find_cpt(code: &str) -> Option<&'static CptCode> {
    CPT_DATABASE.get(code)
}

/// Check if a CPT code is billable
pub fn cpt_is_billable(code: &str) -> bool {
    find_cpt(code).map(|c| c.is_billable()).unwrap_or(false)
}

/// Validate a CPT code format
pub fn validate_cpt_format(code: &str) -> Result<(), String> {
    if code.len() != 5 {
        return Err("CPT code must be 5 digits".to_string());
    }

    if !code.chars().all(|c| c.is_ascii_digit()) {
        return Err("CPT code must contain only digits".to_string());
    }

    Ok(())
}

/// Get all CPT codes in a category
pub fn cpt_codes_by_category(category: CptCategory) -> Vec<&'static CptCode> {
    CPT_DATABASE
        .values()
        .filter(|c| c.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_cpt() {
        let cpt = find_cpt("99215");
        assert!(cpt.is_some());
        assert_eq!(cpt.unwrap().code, "99215");
    }

    #[test]
    fn test_cpt_billable() {
        assert!(cpt_is_billable("99215"));
    }

    #[test]
    fn test_cpt_validation() {
        assert!(validate_cpt_format("99215").is_ok());
        assert!(validate_cpt_format("999").is_err()); // Too short
    }

    #[test]
    fn test_em_codes() {
        assert_eq!(
            find_cpt("99202").unwrap().category,
            CptCategory::EvaluationManagement
        );
        assert_eq!(
            find_cpt("99215").unwrap().category,
            CptCategory::EvaluationManagement
        );
    }

    #[test]
    fn test_telehealth_eligible() {
        assert!(find_cpt("99215").unwrap().telehealth_eligible);
        assert!(!find_cpt("71020").unwrap().telehealth_eligible); // X-ray
    }
}

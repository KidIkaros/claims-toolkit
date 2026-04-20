//! ICD-10-CM diagnosis codes
//!
//! ICD-10-CM codes are used to classify diseases and other health problems.

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// ICD-10 code categories (chapters)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Icd10Category {
    /// Infectious and parasitic diseases (A00-B99)
    InfectiousParasitic,
    /// Neoplasms (C00-D49)
    Neoplasms,
    /// Diseases of the blood and blood-forming organs (D50-D89)
    BloodBloodForming,
    /// Endocrine, nutritional and metabolic diseases (E00-E89)
    EndocrineNutritionalMetabolic,
    /// Mental and behavioral disorders (F01-F99)
    MentalBehavioral,
    /// Diseases of the nervous system (G00-G99)
    NervousSystem,
    /// Diseases of the eye and adnexa (H00-H59)
    EyeAdnexa,
    /// Diseases of the ear and mastoid process (H60-H95)
    EarMastoid,
    /// Diseases of the circulatory system (I00-I99)
    Circulatory,
    /// Diseases of the respiratory system (J00-J99)
    Respiratory,
    /// Diseases of the digestive system (K00-K95)
    Digestive,
    /// Diseases of the skin and subcutaneous tissue (L00-L99)
    SkinSubcutaneous,
    /// Diseases of the musculoskeletal system (M00-M99)
    Musculoskeletal,
    /// Diseases of the genitourinary system (N00-N99)
    Genitourinary,
    /// Pregnancy, childbirth and the puerperium (O00-O9A)
    PregnancyChildbirth,
    /// Certain conditions originating in the perinatal period (P00-P96)
    Perinatal,
    /// Congenital malformations (Q00-Q99)
    CongenitalMalformations,
    /// Symptoms, signs and abnormal findings (R00-R99)
    SymptomsSigns,
    /// Injury, poisoning and external causes (S00-T88)
    InjuryPoisoning,
    /// External causes of morbidity (V00-Y99)
    ExternalCauses,
    /// Factors influencing health status (Z00-Z99)
    HealthStatusFactors,
}

impl Icd10Category {
    /// Get the chapter code range for this category
    pub fn chapter_range(&self) -> &'static str {
        match self {
            Self::InfectiousParasitic => "A00-B99",
            Self::Neoplasms => "C00-D49",
            Self::BloodBloodForming => "D50-D89",
            Self::EndocrineNutritionalMetabolic => "E00-E89",
            Self::MentalBehavioral => "F01-F99",
            Self::NervousSystem => "G00-G99",
            Self::EyeAdnexa => "H00-H59",
            Self::EarMastoid => "H60-H95",
            Self::Circulatory => "I00-I99",
            Self::Respiratory => "J00-J99",
            Self::Digestive => "K00-K95",
            Self::SkinSubcutaneous => "L00-L99",
            Self::Musculoskeletal => "M00-M99",
            Self::Genitourinary => "N00-N99",
            Self::PregnancyChildbirth => "O00-O9A",
            Self::Perinatal => "P00-P96",
            Self::CongenitalMalformations => "Q00-Q99",
            Self::SymptomsSigns => "R00-R99",
            Self::InjuryPoisoning => "S00-T88",
            Self::ExternalCauses => "V00-Y99",
            Self::HealthStatusFactors => "Z00-Z99",
        }
    }
}

/// ICD-10 code information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icd10Code {
    /// The ICD-10 code (with dots for readability, e.g., "I10")
    pub code: &'static str,

    /// Full description
    pub description: &'static str,

    /// Category/chapter
    pub category: Icd10Category,

    /// Whether this is a "billable" or specific code
    /// Non-specific codes end in certain patterns (9, .8, .9)
    pub billable: bool,

    /// HCC (Hierarchical Condition Category) status for Medicare
    pub hcc_relevant: bool,

    /// Common comorbidities associated with this code
    pub common_comorbidities: Vec<&'static str>,
}

impl Icd10Code {
    /// Check if this code is specific enough for billing
    pub fn is_billable(&self) -> bool {
        self.billable
    }

    /// Check if this is an HCC-relevant code
    pub fn is_hcc_relevant(&self) -> bool {
        self.hcc_relevant
    }
}

/// ICD-10 code database (common codes)
pub static ICD10_DATABASE: Lazy<FxHashMap<&'static str, Icd10Code>> = Lazy::new(|| {
    let mut m = FxHashMap::default();

    // Circulatory System (I codes) - High HCC relevance
    m.insert(
        "I10",
        Icd10Code {
            code: "I10",
            description: "Essential (primary) hypertension",
            category: Icd10Category::Circulatory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I50", "E11", "N18"],
        },
    );
    m.insert(
        "I11",
        Icd10Code {
            code: "I11",
            description: "Hypertensive heart disease",
            category: Icd10Category::Circulatory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I50", "E11"],
        },
    );
    m.insert(
        "I12",
        Icd10Code {
            code: "I12",
            description: "Hypertensive chronic kidney disease",
            category: Icd10Category::Circulatory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["N18", "I50"],
        },
    );
    m.insert(
        "I25",
        Icd10Code {
            code: "I25",
            description: "Chronic ischemic heart disease",
            category: Icd10Category::Circulatory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I50", "E11"],
        },
    );
    m.insert(
        "I50",
        Icd10Code {
            code: "I50",
            description: "Heart failure",
            category: Icd10Category::Circulatory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10", "E11", "J44"],
        },
    );
    m.insert(
        "I50.9",
        Icd10Code {
            code: "I50.9",
            description: "Heart failure, unspecified",
            category: Icd10Category::Circulatory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10", "E11"],
        },
    );

    // Endocrine/Metabolic (E codes)
    m.insert(
        "E11",
        Icd10Code {
            code: "E11",
            description: "Type 2 diabetes mellitus",
            category: Icd10Category::EndocrineNutritionalMetabolic,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10", "N18", "I50"],
        },
    );
    m.insert(
        "E11.9",
        Icd10Code {
            code: "E11.9",
            description: "Type 2 diabetes mellitus without complications",
            category: Icd10Category::EndocrineNutritionalMetabolic,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10"],
        },
    );
    m.insert(
        "E11.8",
        Icd10Code {
            code: "E11.8",
            description: "Type 2 diabetes mellitus with unspecified complications",
            category: Icd10Category::EndocrineNutritionalMetabolic,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10", "N18"],
        },
    );
    m.insert(
        "E66",
        Icd10Code {
            code: "E66",
            description: "Overweight and obesity",
            category: Icd10Category::EndocrineNutritionalMetabolic,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["E11", "I10", "M17"],
        },
    );
    m.insert(
        "E78",
        Icd10Code {
            code: "E78",
            description: "Disorders of lipoprotein metabolism and other lipidaemias",
            category: Icd10Category::EndocrineNutritionalMetabolic,
            billable: true,
            hcc_relevant: false,
            common_comorbidities: vec!["I25", "E11"],
        },
    );

    // Respiratory (J codes)
    m.insert(
        "J44",
        Icd10Code {
            code: "J44",
            description: "Other chronic obstructive pulmonary disease",
            category: Icd10Category::Respiratory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I50", "J45"],
        },
    );
    m.insert(
        "J44.9",
        Icd10Code {
            code: "J44.9",
            description: "COPD, unspecified",
            category: Icd10Category::Respiratory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I50"],
        },
    );
    m.insert(
        "J45",
        Icd10Code {
            code: "J45",
            description: "Asthma",
            category: Icd10Category::Respiratory,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["J30"],
        },
    );

    // Neoplasms (C codes)
    m.insert(
        "C50",
        Icd10Code {
            code: "C50",
            description: "Malignant neoplasm of breast",
            category: Icd10Category::Neoplasms,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["C61", "C64"],
        },
    );
    m.insert(
        "C61",
        Icd10Code {
            code: "C61",
            description: "Malignant neoplasm of prostate",
            category: Icd10Category::Neoplasms,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["C50"],
        },
    );
    m.insert(
        "C34",
        Icd10Code {
            code: "C34",
            description: "Malignant neoplasm of bronchus and lung",
            category: Icd10Category::Neoplasms,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["C50"],
        },
    );
    m.insert(
        "C64",
        Icd10Code {
            code: "C64",
            description: "Malignant neoplasm of kidney",
            category: Icd10Category::Neoplasms,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["C50", "C61"],
        },
    );
    m.insert(
        "C67",
        Icd10Code {
            code: "C67",
            description: "Malignant neoplasm of bladder",
            category: Icd10Category::Neoplasms,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["C61"],
        },
    );
    m.insert(
        "C18",
        Icd10Code {
            code: "C18",
            description: "Malignant neoplasm of colon",
            category: Icd10Category::Neoplasms,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["C50"],
        },
    );

    // Musculoskeletal (M codes)
    m.insert(
        "M17",
        Icd10Code {
            code: "M17",
            description: "Gonarthrosis [osteoarthritis of knee]",
            category: Icd10Category::Musculoskeletal,
            billable: true,
            hcc_relevant: false,
            common_comorbidities: vec!["E66", "M16"],
        },
    );
    m.insert(
        "M16",
        Icd10Code {
            code: "M16",
            description: "Osteoarthritis of hip",
            category: Icd10Category::Musculoskeletal,
            billable: true,
            hcc_relevant: false,
            common_comorbidities: vec!["M17", "E66"],
        },
    );
    m.insert(
        "M54",
        Icd10Code {
            code: "M54",
            description: "Dorsalgia (back pain)",
            category: Icd10Category::Musculoskeletal,
            billable: true,
            hcc_relevant: false,
            common_comorbidities: vec!["M43", "M48"],
        },
    );
    m.insert(
        "M25",
        Icd10Code {
            code: "M25",
            description: "Other joint disorder",
            category: Icd10Category::Musculoskeletal,
            billable: true,
            hcc_relevant: false,
            common_comorbidities: vec!["M17", "M16"],
        },
    );

    // Genitourinary (N codes)
    m.insert(
        "N18",
        Icd10Code {
            code: "N18",
            description: "Chronic kidney disease (CKD)",
            category: Icd10Category::Genitourinary,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10", "E11", "I50"],
        },
    );
    m.insert(
        "N18.3",
        Icd10Code {
            code: "N18.3",
            description: "Chronic kidney disease, stage 3",
            category: Icd10Category::Genitourinary,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10", "E11"],
        },
    );
    m.insert(
        "N18.6",
        Icd10Code {
            code: "N18.6",
            description: "End stage renal disease (ESRD)",
            category: Icd10Category::Genitourinary,
            billable: true,
            hcc_relevant: true,
            common_comorbidities: vec!["I10", "E11", "Z49"],
        },
    );

    // Symptoms/Signs (R codes) - typically non-HCC
    m.insert(
        "R06",
        Icd10Code {
            code: "R06",
            description: "Abnormalities of breathing",
            category: Icd10Category::SymptomsSigns,
            billable: false,
            hcc_relevant: false,
            common_comorbidities: vec!["J44", "J45"],
        },
    );
    m.insert(
        "R07",
        Icd10Code {
            code: "R07",
            description: "Pain in throat and chest",
            category: Icd10Category::SymptomsSigns,
            billable: false,
            hcc_relevant: false,
            common_comorbidities: vec!["I25"],
        },
    );
    m.insert(
        "R10",
        Icd10Code {
            code: "R10",
            description:
                "Abdominal and pelvic pain, constipation, and other digestive system symptoms",
            category: Icd10Category::SymptomsSigns,
            billable: false,
            hcc_relevant: false,
            common_comorbidities: vec![],
        },
    );

    // ============================================================================
    // Z Codes - Health Status Factors (Preventive Care & Screening)
    // ============================================================================

    // General Examinations (Z00)
    m.insert("Z00.00", Icd10Code {
        code: "Z00.00",
        description: "Encounter for general adult medical examination without abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z00.01", Icd10Code {
        code: "Z00.01",
        description: "Encounter for general adult medical examination with abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z00.121", Icd10Code {
        code: "Z00.121",
        description: "Encounter for routine child health examination without abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z00.129", Icd10Code {
        code: "Z00.129",
        description: "Encounter for routine child health examination with abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });

    // Special Examinations (Z01)
    m.insert("Z01.00", Icd10Code {
        code: "Z01.00",
        description: "Encounter for examination of ears and hearing without abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z01.10", Icd10Code {
        code: "Z01.10",
        description: "Encounter for examination of eyes and vision without abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z01.311", Icd10Code {
        code: "Z01.311",
        description: "Encounter for hearing examination following failed hearing screening",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z01.411", Icd10Code {
        code: "Z01.411",
        description: "Encounter for gynecological examination (general) (routine) with abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z01.419", Icd10Code {
        code: "Z01.419",
        description: "Encounter for gynecological examination (general) (routine) without abnormal findings",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z01.5", Icd10Code {
        code: "Z01.5",
        description: "Encounter for screening of respiratory tuberculosis",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z01.6", Icd10Code {
        code: "Z01.6",
        description: "Encounter for blood-pressure screening",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec!["I10"],
    });
    m.insert("Z01.7", Icd10Code {
        code: "Z01.7",
        description: "Encounter for examination of sign, symptom, or other specified result",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z01.811", Icd10Code {
        code: "Z01.811",
        description: "Encounter for preprocedural cardiovascular examination",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec!["I10", "I50"],
    });
    m.insert("Z01.812", Icd10Code {
        code: "Z01.812",
        description: "Encounter for preprocedural respiratory examination",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec!["J44"],
    });
    m.insert("Z01.89", Icd10Code {
        code: "Z01.89",
        description: "Encounter for other specified special examinations",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });

    // Screening for Neoplasms (Z12)
    m.insert("Z12.11", Icd10Code {
        code: "Z12.11",
        description: "Encounter for screening for malignant neoplasm of colon",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z12.12", Icd10Code {
        code: "Z12.12",
        description: "Encounter for screening for malignant neoplasm of breast (mammogram)",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z12.13", Icd10Code {
        code: "Z12.13",
        description: "Encounter for screening for malignant neoplasm of cervix",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z12.21", Icd10Code {
        code: "Z12.21",
        description: "Encounter for screening for malignant neoplasm of rectum",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z12.29", Icd10Code {
        code: "Z12.29",
        description: "Encounter for screening for malignant neoplasm of rectum and sigmoid colon",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z12.31", Icd10Code {
        code: "Z12.31",
        description: "Encounter for screening for malignant neoplasm of prostate",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z12.39", Icd10Code {
        code: "Z12.39",
        description: "Encounter for screening for malignant neoplasm of other sites",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });

    // Screening for Other Disorders (Z13)
    m.insert("Z13.1", Icd10Code {
        code: "Z13.1",
        description: "Encounter for screening for diabetes mellitus",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec!["E11"],
    });
    m.insert("Z13.6", Icd10Code {
        code: "Z13.6",
        description: "Encounter for screening for cardiovascular disorders",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec!["I10", "I50"],
    });
    m.insert("Z13.8", Icd10Code {
        code: "Z13.8",
        description: "Encounter for screening for other specified disorders",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z13.9", Icd10Code {
        code: "Z13.9",
        description: "Encounter for screening for unspecified disorder",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });

    // Screening for Infectious Diseases (Z11)
    m.insert("Z11.0", Icd10Code {
        code: "Z11.0",
        description: "Encounter for screening for intestinal infectious diseases",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z11.1", Icd10Code {
        code: "Z11.1",
        description: "Encounter for screening for respiratory tuberculosis",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z11.3", Icd10Code {
        code: "Z11.3",
        description: "Encounter for screening for other bacterial diseases",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z11.8", Icd10Code {
        code: "Z11.8",
        description: "Encounter for screening for other infectious diseases",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z11.9", Icd10Code {
        code: "Z11.9",
        description: "Encounter for screening for unspecified infectious disease",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });

    // Immunization (Z23)
    m.insert("Z23", Icd10Code {
        code: "Z23",
        description: "Encounter for immunization",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });

    // Health supervision (Z39)
    m.insert("Z39.0", Icd10Code {
        code: "Z39.0",
        description: "Encounter for examination and care of mother immediately after delivery",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });
    m.insert("Z39.1", Icd10Code {
        code: "Z39.1",
        description: "Encounter for examination and care of mother during lactation",
        category: Icd10Category::HealthStatusFactors,
        billable: true,
        hcc_relevant: false,
        common_comorbidities: vec![],
    });

    m
});

/// Find an ICD-10 code by its code string
pub fn find_icd10(code: &str) -> Option<&'static Icd10Code> {
    ICD10_DATABASE.get(code)
}

/// Validate an ICD-10 code format
pub fn validate_icd10(code: &str) -> Result<(), String> {
    let code = code.to_uppercase();

    // Remove dots for validation
    let clean_code = code.replace('.', "");

    if clean_code.len() < 3 || clean_code.len() > 8 {
        return Err("ICD-10 code must be 3-8 characters".to_string());
    }

    // First character must be A-Z
    if !clean_code
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false)
    {
        return Err("ICD-10 code must start with a letter".to_string());
    }

    // Remaining characters after the first must include at least one digit
    // and can only be alphanumeric
    let rest: String = clean_code.chars().skip(1).collect();
    if !rest.chars().any(|c| c.is_ascii_digit()) {
        return Err(
            "ICD-10 code must contain at least one digit after the first character".to_string(),
        );
    }

    if !rest.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("ICD-10 code can only contain letters and digits".to_string());
    }

    Ok(())
}

/// Check if an ICD-10 code is HCC-relevant
pub fn icd10_is_hcc_relevant(code: &str) -> bool {
    find_icd10(code)
        .map(|c| c.is_hcc_relevant())
        .unwrap_or(false)
}

/// Get common comorbidities for an ICD-10 code
pub fn icd10_comorbidities(code: &str) -> Vec<&'static str> {
    find_icd10(code)
        .map(|c| c.common_comorbidities.to_vec())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_icd10() {
        let icd = find_icd10("I10");
        assert!(icd.is_some());
        assert_eq!(icd.unwrap().code, "I10");
    }

    #[test]
    fn test_icd10_validation() {
        assert!(validate_icd10("I10").is_ok());
        assert!(validate_icd10("I10.9").is_ok());
        assert!(validate_icd10("E11.8").is_ok());
        assert!(validate_icd10("INVALID").is_err());
        assert!(validate_icd10("A").is_err()); // Too short
    }

    #[test]
    fn test_hcc_relevance() {
        assert!(icd10_is_hcc_relevant("I10")); // Hypertension
        assert!(icd10_is_hcc_relevant("E11")); // Diabetes
        assert!(icd10_is_hcc_relevant("I50")); // Heart failure
        assert!(!icd10_is_hcc_relevant("M17")); // Knee OA
    }

    #[test]
    fn test_comorbidities() {
        let comorbs = icd10_comorbidities("I10");
        assert!(!comorbs.is_empty());
        assert!(comorbs.contains(&"E11") || comorbs.contains(&"I50"));
    }
}

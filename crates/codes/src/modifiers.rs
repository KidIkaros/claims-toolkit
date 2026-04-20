//! CPT/HCPCS modifiers
//!
//! Modifiers provide additional information about procedures without changing the code itself.

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// CPT/HCPCS modifier information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CptModifier {
    /// The 2-character modifier code
    pub code: &'static str,

    /// Full description
    pub description: &'static str,

    /// Whether this modifier affects payment
    pub payment_affecting: bool,

    /// Which CPT sections typically use this modifier
    pub typical_usage: Vec<&'static str>,
}

/// Modifier database (common modifiers)
pub static MODIFIER_DATABASE: Lazy<FxHashMap<&'static str, CptModifier>> = Lazy::new(|| {
    let mut m = FxHashMap::default();

    // E&M modifiers
    m.insert("25", CptModifier {
        code: "25",
        description: "Significant, separately identifiable evaluation and management service by the same physician or other qualified health care professional on the same day of the procedure or other service",
        payment_affecting: true,
        typical_usage: vec!["E&M", "Surgery", "Medicine"],
    });
    m.insert(
        "24",
        CptModifier {
            code: "24",
            description: "Unilateral evaluation and management (E/M) service",
            payment_affecting: false,
            typical_usage: vec!["E&M"],
        },
    );
    m.insert(
        "27",
        CptModifier {
            code: "27",
            description: "Multiple outpatient hospital encounter evaluations on the same date",
            payment_affecting: false,
            typical_usage: vec!["E&M"],
        },
    );

    // Anesthesia modifiers
    m.insert(
        "AA",
        CptModifier {
            code: "AA",
            description: "Anesthesia services performed personally by anesthesiologist",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );
    m.insert(
        "QK",
        CptModifier {
            code: "QK",
            description: "Medical direction by a physician who is not performing the anesthesia",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );
    m.insert(
        "QY",
        CptModifier {
            code: "QY",
            description: "Medical direction of one qualified anesthetist by a physician",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );
    m.insert(
        "QZ",
        CptModifier {
            code: "QZ",
            description: "CRNA service without medical direction by a physician",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );
    m.insert("AD", CptModifier {
        code: "AD",
        description: "Medical supervision by a physician: more than four concurrent anesthesia procedures",
        payment_affecting: false,
        typical_usage: vec!["Anesthesia"],
    });
    m.insert(
        "QS",
        CptModifier {
            code: "QS",
            description: "Monitored anesthesia care service",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );

    // Surgery/procedure modifiers
    m.insert(
        "50",
        CptModifier {
            code: "50",
            description: "Bilateral procedure",
            payment_affecting: true,
            typical_usage: vec!["Surgery", "Radiology"],
        },
    );
    m.insert(
        "51",
        CptModifier {
            code: "51",
            description: "Multiple procedures",
            payment_affecting: true,
            typical_usage: vec!["Surgery", "Medicine", "Radiology"],
        },
    );
    m.insert(
        "52",
        CptModifier {
            code: "52",
            description: "Reduced services",
            payment_affecting: true,
            typical_usage: vec!["Surgery", "E&M"],
        },
    );
    m.insert(
        "53",
        CptModifier {
            code: "53",
            description: "Discontinued procedure",
            payment_affecting: true,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert(
        "54",
        CptModifier {
            code: "54",
            description: "Multiple surgeons",
            payment_affecting: false,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert(
        "55",
        CptModifier {
            code: "55",
            description: "Bilateral procedure (co-surgeons)",
            payment_affecting: false,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert(
        "56",
        CptModifier {
            code: "56",
            description: "Preoperative component only",
            payment_affecting: true,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert(
        "57",
        CptModifier {
            code: "57",
            description: "Postoperative component only",
            payment_affecting: true,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert(
        "58",
        CptModifier {
            code: "58",
            description:
                "Staged or related procedure by the same physician during the postoperative period",
            payment_affecting: false,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert(
        "59",
        CptModifier {
            code: "59",
            description: "Distinct procedural service",
            payment_affecting: true,
            typical_usage: vec!["Surgery", "Radiology", "Pathology"],
        },
    );
    m.insert(
        "62",
        CptModifier {
            code: "62",
            description: "Two surgeons",
            payment_affecting: false,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert(
        "66",
        CptModifier {
            code: "66",
            description:
                "Left side (used to identify procedures performed on the left side of the body)",
            payment_affecting: false,
            typical_usage: vec!["Surgery", "Radiology"],
        },
    );
    m.insert(
        "RT",
        CptModifier {
            code: "RT",
            description:
                "Right side (used to identify procedures performed on the right side of the body)",
            payment_affecting: false,
            typical_usage: vec!["Surgery", "Radiology"],
        },
    );
    m.insert(
        "LT",
        CptModifier {
            code: "LT",
            description: "Left side (alternative to modifier 66)",
            payment_affecting: false,
            typical_usage: vec!["Surgery", "Radiology"],
        },
    );
    m.insert(
        "76",
        CptModifier {
            code: "76",
            description: "Repeat procedure by same physician",
            payment_affecting: false,
            typical_usage: vec!["Surgery", "Radiology"],
        },
    );
    m.insert(
        "77",
        CptModifier {
            code: "77",
            description: "Repeat procedure by another physician",
            payment_affecting: false,
            typical_usage: vec!["Surgery", "Radiology"],
        },
    );
    m.insert(
        "78",
        CptModifier {
            code: "78",
            description: "Unplanned return to the operating/procedure room",
            payment_affecting: false,
            typical_usage: vec!["Surgery"],
        },
    );
    m.insert("79", CptModifier {
        code: "79",
        description: "Unrelated procedure or service by the same physician during the postoperative period",
        payment_affecting: false,
        typical_usage: vec!["Surgery"],
    });

    // Radiology modifiers
    m.insert(
        "26",
        CptModifier {
            code: "26",
            description:
                "Professional component (for certain services billed in multiple components)",
            payment_affecting: true,
            typical_usage: vec!["Radiology", "Pathology"],
        },
    );
    m.insert(
        "TC",
        CptModifier {
            code: "TC",
            description: "Technical component (for certain services billed in multiple components)",
            payment_affecting: true,
            typical_usage: vec!["Radiology", "Pathology"],
        },
    );

    // Place of service modifiers
    m.insert(
        "F1",
        CptModifier {
            code: "F1",
            description: "Dialysis facility",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "F2",
        CptModifier {
            code: "F2",
            description: "Freestanding facility",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "F3",
        CptModifier {
            code: "F3",
            description: "Facility (general)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "F4",
        CptModifier {
            code: "F4",
            description: "Facility (inpatient)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "F5",
        CptModifier {
            code: "F5",
            description: "Facility (outpatient)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "F6",
        CptModifier {
            code: "F6",
            description: "Facility (ambulatory surgical center)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "F7",
        CptModifier {
            code: "F7",
            description: "Facility (skilled nursing facility)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "F8",
        CptModifier {
            code: "F8",
            description: "Facility (nursing facility)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "FB",
        CptModifier {
            code: "FB",
            description: "Facility (comprehensive outpatient rehabilitation facility)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert(
        "FC",
        CptModifier {
            code: "FC",
            description: "Facility (comprehensive inpatient rehabilitation facility)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );

    // Other modifiers
    m.insert(
        "32",
        CptModifier {
            code: "32",
            description: "Mandated services",
            payment_affecting: false,
            typical_usage: vec!["E&M", "Surgery"],
        },
    );
    m.insert("33", CptModifier {
        code: "33",
        description: "Preventive services (excluding smoking cessation and high-intensity clinical behavioral intervention)",
        payment_affecting: false,
        typical_usage: vec!["Medicine"],
    });
    m.insert("90", CptModifier {
        code: "90",
        description: "Reference (outside) laboratory: The reference laboratory takes the component test from an outside lab and re-bills it",
        payment_affecting: false,
        typical_usage: vec!["Pathology"],
    });
    m.insert(
        "91",
        CptModifier {
            code: "91",
            description: "Repeat clinical diagnostic laboratory test",
            payment_affecting: false,
            typical_usage: vec!["Pathology"],
        },
    );
    m.insert(
        "95",
        CptModifier {
            code: "95",
            description: "Telehealth modifier (added during COVID-19 PHE)",
            payment_affecting: false,
            typical_usage: vec!["E&M", "Medicine", "Psychiatry"],
        },
    );
    m.insert(
        "GT",
        CptModifier {
            code: "GT",
            description: "Interactive audio and video telecommunications",
            payment_affecting: false,
            typical_usage: vec!["E&M", "Psychiatry"],
        },
    );
    m.insert(
        "GQ",
        CptModifier {
            code: "GQ",
            description: "Via asynchronous telecommunications system",
            payment_affecting: false,
            typical_usage: vec!["E&M", "Medicine"],
        },
    );
    m.insert(
        "GM",
        CptModifier {
            code: "GM",
            description: "Multiple monitored anesthesia care (concurrent)",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );
    m.insert(
        "GJ",
        CptModifier {
            code: "GJ",
            description: "Monitored anesthesia care (MAC) for a qualifying procedure",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );
    m.insert(
        "GK",
        CptModifier {
            code: "GK",
            description: "Monitored anesthesia care by a qualified anesthetist",
            payment_affecting: false,
            typical_usage: vec!["Anesthesia"],
        },
    );
    m.insert(
        "GN",
        CptModifier {
            code: "GN",
            description: "Service delivered by nurse practitioner",
            payment_affecting: false,
            typical_usage: vec!["E&M", "Medicine"],
        },
    );
    m.insert(
        "GO",
        CptModifier {
            code: "GO",
            description: "Service delivered by clinical nurse specialist",
            payment_affecting: false,
            typical_usage: vec!["E&M", "Medicine"],
        },
    );
    m.insert(
        "GP",
        CptModifier {
            code: "GP",
            description: "Service delivered by physician assistant",
            payment_affecting: false,
            typical_usage: vec!["E&M", "Medicine"],
        },
    );
    m.insert("GC", CptModifier {
        code: "GC",
        description: "This service has been performed in part by a resident under the direction of a teaching physician",
        payment_affecting: false,
        typical_usage: vec!["E&M", "Surgery", "Medicine"],
    });
    m.insert("GE", CptModifier {
        code: "GE",
        description: "This service has been performed by a resident without the presence of a teaching physician under the primary care exception",
        payment_affecting: false,
        typical_usage: vec!["E&M", "Surgery", "Medicine"],
    });
    m.insert(
        "KX",
        CptModifier {
            code: "KX",
            description: "Requirements specified in the medical policy for coverage are met",
            payment_affecting: false,
            typical_usage: vec!["Surgery", "Medicine"],
        },
    );
    m.insert("KJ", CptModifier {
        code: "KJ",
        description: "Reasonable and necessary services per National Coverage Determination (NCD) or Local Coverage Determination (LCD)",
        payment_affecting: false,
        typical_usage: vec!["Surgery", "Medicine"],
    });
    m.insert("KL", CptModifier {
        code: "KL",
        description: "Reasonable and necessary service or supply (for documentation of medical necessity)",
        payment_affecting: false,
        typical_usage: vec!["Surgery", "Medicine"],
    });
    m.insert(
        "KY",
        CptModifier {
            code: "KY",
            description:
                "Treatment of deep, moderate, or severe burns (e.g., hyperbaric oxygen therapy)",
            payment_affecting: false,
            typical_usage: vec!["Medicine"],
        },
    );
    m.insert("KG", CptModifier {
        code: "KG",
        description: "Reasonable and necessary service or supply, when the dollar amount of the bill exceeds the cap",
        payment_affecting: false,
        typical_usage: vec!["Surgery", "Medicine"],
    });
    m.insert("KH", CptModifier {
        code: "KH",
        description: "Reasonable and necessary service or supply, when the frequency or duration of the service exceeds the cap",
        payment_affecting: false,
        typical_usage: vec!["Surgery", "Medicine"],
    });
    m.insert("KI", CptModifier {
        code: "KI",
        description: "Reasonable and necessary service or supply, when the scope of the service exceeds the cap",
        payment_affecting: false,
        typical_usage: vec!["Surgery", "Medicine"],
    });

    m
});

/// Find a modifier by its code
pub fn find_modifier(code: &str) -> Option<&'static CptModifier> {
    MODIFIER_DATABASE.get(code)
}

/// Validate a modifier code
pub fn validate_modifier_code(code: &str) -> Result<(), String> {
    if code.len() != 2 {
        return Err("Modifier must be 2 characters".to_string());
    }

    if !find_modifier(code).is_some() {
        return Err(format!("Modifier '{}' not found", code));
    }

    Ok(())
}

/// Validate that a modifier is appropriate for a given CPT code
pub fn validate_modifier_for_cpt(modifier: &str, cpt_code: &str) -> Result<(), String> {
    let mod_info = find_modifier(modifier)
        .ok_or_else(|| format!("Modifier '{}' not found", modifier))?;

    let cpt = crate::cpt::find_cpt(cpt_code)
        .ok_or_else(|| format!("CPT code '{}' not found", cpt_code))?;

    // Check if the modifier is appropriate for the CPT category
    let appropriate = mod_info.typical_usage.iter().any(|usage| {
        let category_matches = match *usage {
            "E&M" => cpt.category == crate::cpt::CptCategory::EvaluationManagement,
            "Surgery" => matches!(cpt.category, crate::cpt::CptCategory::Surgery),
            "Radiology" => cpt.category == crate::cpt::CptCategory::Radiology,
            "Pathology" => cpt.category == crate::cpt::CptCategory::PathologyLaboratory,
            "Medicine" => cpt.category == crate::cpt::CptCategory::Medicine,
            "Anesthesia" => cpt.category == crate::cpt::CptCategory::Anesthesia,
            _ => false,
        };
        category_matches
    });

    // Telehealth modifiers are always appropriate for E&M
    let is_telehealth = matches!(modifier, "95" | "GT" | "GQ");

    if !appropriate && !is_telehealth && !matches!(modifier, "RT" | "LT") {
        return Err(format!(
            "Modifier {} is not typically used with {} codes",
            modifier, cpt.category
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_modifier() {
        let mod25 = find_modifier("25");
        assert!(mod25.is_some());
        assert!(mod25.unwrap().description.contains("separately"));
    }

    #[test]
    fn test_modifier_validation() {
        assert!(validate_modifier_code("25").is_ok());
        assert!(validate_modifier_code("ZZZ").is_err());
    }

    #[test]
    fn test_modifier_for_cpt() {
        // Modifier 25 is appropriate for E&M codes
        assert!(validate_modifier_for_cpt("25", "99215").is_ok());

        // Modifier 25 should work for surgery too
        assert!(validate_modifier_for_cpt("25", "11400").is_ok());

        // Telehealth modifier 95 works for E&M
        assert!(validate_modifier_for_cpt("95", "99215").is_ok());
    }
}

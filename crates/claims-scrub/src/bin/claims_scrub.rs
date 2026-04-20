use clap::{Parser, Subcommand};
use colored::Colorize;
use claims_scrub::*;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// Claims scrubbing engine — validate claims before submission.
///
/// Checks CPT/ICD-10 codes, NCCI edits, modifier rules, and
/// diagnosis-procedure linkage to prevent denials.
#[derive(Parser)]
#[command(name = "claims-scrub", version, about)]
struct Cli {
    /// Claim JSON file (reads stdin if omitted)
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a claim from JSON
    Validate {
        /// Claim JSON file (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate a sample claim JSON
    Sample,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { file, json }) => cmd_validate(file, json)?,
        Some(Commands::Sample) => cmd_sample()?,
        None => {
            // Default: validate from file or stdin
            cmd_validate(cli.file, false)?
        }
    }

    Ok(())
}

fn cmd_validate(file: Option<PathBuf>, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let raw = match &file {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let claim: Claim = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid claim JSON: {}", e))?;

    let scrubber = ClaimsScrubber::new();
    let result = scrubber.validate_claim(&claim);

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_result(&claim, &result);
    }

    Ok(())
}

fn cmd_sample() -> Result<(), Box<dyn std::error::Error>> {
    let sample = serde_json::json!({
        "claim_id": "CLM-2024-001",
        "patient": {
            "patient_id": "P12345",
            "date_of_birth": "1980-05-15",
            "gender": "F",
            "insurance_id": "INS-67890"
        },
        "provider": {
            "npi": "1234567890",
            "taxonomy_code": "207Q00000X",
            "name": "Dr. Jane Smith"
        },
        "payer": {
            "payer_id": "UHC001",
            "name": "United Healthcare",
            "payer_type": "Commercial"
        },
        "date_of_service": "2024-01-15",
        "lines": [
            {
                "line_number": 1,
                "cpt_code": "99214",
                "modifiers": ["25"],
                "units": 1,
                "charge_amount": 250.0,
                "diagnosis_codes": ["I10", "E11.9"],
                "date_of_service": "2024-01-15",
                "place_of_service": "11"
            },
            {
                "line_number": 2,
                "cpt_code": "85025",
                "modifiers": [],
                "units": 1,
                "charge_amount": 50.0,
                "diagnosis_codes": ["E11.9"],
                "date_of_service": "2024-01-15",
                "place_of_service": "11"
            }
        ],
        "total_charge": 300.0
    });
    println!("{}", serde_json::to_string_pretty(&sample)?);
    Ok(())
}

fn print_result(claim: &Claim, result: &ValidationResult) {
    println!("{}", "═".repeat(60).bright_blue());
    if result.is_clean {
        println!("  {} Claim {} — CLEAN", "✓".bright_green(), claim.claim_id.bright_white());
    } else {
        println!("  {} Claim {} — {} error(s), {} warning(s)",
            "⚠".bright_yellow(),
            claim.claim_id.bright_white(),
            result.error_count.to_string().bright_red(),
            result.warning_count.to_string().bright_yellow()
        );
    }
    println!("{}", "═".repeat(60).bright_blue());
    println!();

    // Denial risk
    let risk_color = if result.denial_risk < 20 {
        result.denial_risk.to_string().bright_green()
    } else if result.denial_risk < 50 {
        result.denial_risk.to_string().bright_yellow()
    } else {
        result.denial_risk.to_string().bright_red()
    };
    println!("  Denial Risk: {}%", risk_color);
    println!();

    // Findings
    for (i, finding) in result.findings.iter().enumerate() {
        let sev = match finding.severity {
            FindingSeverity::Error => "ERROR".bright_red().bold(),
            FindingSeverity::Warning => "WARN ".bright_yellow().bold(),
            FindingSeverity::Info => "INFO ".bright_blue(),
        };

        let loc = finding.line_number
            .map(|l| format!("Line {}", l))
            .unwrap_or_default();

        println!("  {}. [{}] {} {}", i + 1, sev, loc.dimmed(), finding.description);
        if let Some(ref sug) = finding.suggestion {
            println!("     → {}", sug.bright_white());
        }
        println!();
    }

    // Corrections
    if !result.corrections.is_empty() {
        println!("  {} Corrections:", "→".bright_green());
        for (i, correction) in result.corrections.iter().enumerate() {
            println!("    {}. {}", i + 1, correction);
        }
    }

    println!("{}", "═".repeat(60).bright_blue());
}

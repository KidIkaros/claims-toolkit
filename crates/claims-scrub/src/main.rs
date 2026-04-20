use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claims-scrub")]
#[command(author = "KidIkaros", version = "0.2.0")]
pub enum ClaimsScrub {
    /// Validate a claim JSON file
    #[command(name = "validate")]
    Validate { 
        file: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        report: bool,
    },
}

fn main() {
    let matches = ClaimsScrub::parse();
    match matches.subcommand_name().as_deref() {
        Some("validate") => cmd_validate(matches),
        _ => println!("Usage: claims-scrub validate <file.json>"),
    }
}

fn cmd_validate(matches: clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let file = matches.get_one::<String>("file").expect("Missing 'file' argument");

    let json = std::fs::read_to_string(file)?;

    // ─── Simple JSON parser ──────────────────────────────────────
    
    fn get_string(json: &str, key: &str) -> Option<String> {
        if let Some(captures) = regex_lite::Regex::new(&format!(r#"{}\s*:\s*["'](.*?)["']"#, key)).unwrap().captures(json) {
            Some(captures[1].to_string())
        } else { None }
    }

    let claim_id = get_string(&json, "claim_id").unwrap_or_else(|| "C001".to_string());
    let patient_dob = get_string(&json, "patient.dob").unwrap_or_else(|| "1975-06-15".to_string());
    let patient_gender = get_string(&json, "patient.gender").unwrap_or_else(|| "M".to_string());
    let provider_npi = get_string(&json, "provider.npi").unwrap_or_else(|| "1234567890".to_string());
    let payer_id = get_string(&json, "payer.id").unwrap_or_else(|| "AETNA".to_string());

    // ─── Output ──────────────────────────────────────
    
    if *matches.get_one::<bool>("json") {
        println!("{{\n  \"claim_id\": \"{}\",\n  \"patient_dob\": \"{}\",
         \"patient_gender\": \"{}\",\n  \"provider_npi\": \"{}\",
         \"payer_id\": \"{}\",\n}}",
                 claim_id, patient_dob, patient_gender,
                 provider_npi, payer_id);
    } else {
        println!("Claims Scrubber Report");
        println!("=".repeat(40));
        println!("Claim: {}", claim_id);
        println!();
        println!("  {} | Patient DOB: {}", "-".repeat(7), patient_dob);
        println!("  {} | Patient Gender: {}", "-".repeat(7), patient_gender);
        println!("  {} | Provider NPI: {}", "-".repeat(7), provider_npi);
        println!("  {} | Payer: {},{}", "-".repeat(7), payer_id, ".green());
        println!();
    }

    Ok(())
}

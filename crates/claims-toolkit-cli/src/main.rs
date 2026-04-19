use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use colored::Colorize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// claims-toolkit — healthcare data tools for X12 835 ERA files and PHI scanning.
///
/// Parse remittance files, generate synthetic test data, and scan for PHI.
#[derive(Parser)]
#[command(name = "claims-toolkit", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an X12 835 ERA file
    Parse {
        /// ERA/835 file to parse
        file: PathBuf,

        #[command(subcommand)]
        output: Option<ParseOutput>,
    },

    /// Generate synthetic X12 835 ERA files for testing
    Generate {
        /// Number of claims to generate
        #[arg(short = 'n', long, default_value_t = 5)]
        count: usize,

        /// Output file (prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Random seed for reproducibility
        #[arg(short, long)]
        seed: Option<u64>,

        /// Output as JSON instead of X12 text
        #[arg(long)]
        json: bool,
    },

    /// Scan text for Protected Health Information (PHI)
    Scan {
        /// File to scan (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Redact PHI instead of reporting
        #[arg(short, long)]
        redact: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Show information about the toolkit
    Info,
}

#[derive(Subcommand)]
enum ParseOutput {
    /// Full claim-by-claim report (default)
    Full,
    /// Financial summary only
    Summary {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Denial report with appeal recommendations
    Denials {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output as CSV
        #[arg(long)]
        csv: bool,
    },
    /// Raw JSON output of parsed structure
    Json,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file, output } => cmd_parse(&file, output)?,
        Commands::Generate { count, output, seed, json } => cmd_generate(count, output, seed, json)?,
        Commands::Scan { file, redact, json } => cmd_scan(file, redact, json)?,
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Info => cmd_info(),
    }

    Ok(())
}

// ── Parse Command ─────────────────────────────────────────────

fn cmd_parse(file: &PathBuf, output: Option<ParseOutput>) -> Result<(), Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(file)
        .map_err(|e| format!("Cannot read '{}': {}", file.display(), e))?;

    let era = era835::parse_era835(&raw)
        .map_err(|e| format_parse_error(file, &e))?;

    match output {
        Some(ParseOutput::Summary { json }) => {
            if json {
                let summary = serde_json::json!({
                    "file": file.display().to_string(),
                    "payer": era.payer.name,
                    "payee": era.payee.name,
                    "npi": era.payee.npi,
                    "total_claims": era.claims.len(),
                    "total_charged": era.total_charged(),
                    "total_paid": era.total_paid(),
                    "total_denied": era.total_denied(),
                    "denial_rate_pct": era.denial_rate(),
                    "denied_claims": era.denied_claims().len(),
                });
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                print_summary(&era);
            }
        }
        Some(ParseOutput::Denials { json, csv }) => {
            let summaries = era.denial_summaries();
            if csv {
                export_denials_csv(&era);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&summaries)?);
            } else {
                print_denials(&era, &summaries);
            }
        }
        Some(ParseOutput::Json) => {
            println!("{}", serde_json::to_string_pretty(&era)?);
        }
        Some(ParseOutput::Full) | None => {
            print_full(&era);
        }
    }

    Ok(())
}

// ── Generate Command ──────────────────────────────────────────

fn cmd_generate(count: usize, output: Option<PathBuf>, seed: Option<u64>, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let batch = era835_synth::generate_synthetic_era835(count, seed);

    let text = if json {
        serde_json::to_string_pretty(&batch)?
    } else {
        era835_synth::serialize_era835(&batch)
    };

    match output {
        Some(path) => {
            fs::write(&path, &text)?;
            eprintln!("{} Generated {} claims -> {}", "✓".bright_green(), count, path.display());
        }
        None => print!("{}", text),
    }

    Ok(())
}

// ── Scan Command ──────────────────────────────────────────────

fn cmd_scan(file: Option<PathBuf>, redact: bool, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let raw = match &file {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    if redact {
        let result = phi_scan::redact_phi(&raw);
        print!("{}", result);
    } else {
        let result = phi_scan::scan_phi(&raw);
        if json {
            let detections: Vec<_> = result.detections.iter().map(|d| {
                serde_json::json!({
                    "category": d.category.to_string(),
                    "span": [d.span.0, d.span.1],
                    "text": &raw[d.span.0..d.span.1],
                })
            }).collect();
            let output = serde_json::json!({
                "file": file.as_ref().map(|p| p.display().to_string()),
                "contains_phi": result.contains_phi,
                "total_detections": result.detections.len(),
                "detections": detections,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            print_phi_results(&raw, &result, file.as_ref());
        }
    }

    Ok(())
}

// ── Info Command ──────────────────────────────────────────────

fn cmd_info() {
    println!("{}", "claims-toolkit v0.1.0".bright_white().bold());
    println!("Healthcare data tools for X12 835 ERA files and PHI scanning.");
    println!();
    println!("Tools:");
    println!("  {} Parse X12 835 ERA remittance files", "parse".bright_cyan());
    println!("  {} Generate synthetic 835 test files", "generate".bright_cyan());
    println!("  {} Scan and redact PHI in clinical text", "scan".bright_cyan());
    println!();
    println!("Quick start:");
    println!("  claims-toolkit generate -n 5 -o test.835");
    println!("  claims-toolkit parse test.835");
    println!("  echo 'text' | claims-toolkit scan");
    println!();
    println!("Libraries: era835, era835-synth, phi-scan");
    println!("License: Apache-2.0 OR MIT");
}

// ── Display Helpers ───────────────────────────────────────────

fn print_summary(era: &era835::Remittance) {
    println!("{}", "═".repeat(60).bright_blue());
    println!("  {} ERA 835 Summary", "▶".bright_green());
    println!("{}", "═".repeat(60).bright_blue());
    println!();
    println!("  Payer:   {}", era.payer.name);
    println!("  Payee:   {} (NPI: {})", era.payee.name, era.payee.npi);
    if let Some(ref trace) = era.trace_number {
        println!("  Trace:   {}", trace);
    }
    println!();
    println!("  Claims:        {}", era.claims.len());
    println!("  Total Charged: ${:.2}", era.total_charged());
    println!("  Total Paid:    ${:.2}", era.total_paid());
    println!("  Total Denied:  ${:.2}", era.total_denied());
    println!("  Denial Rate:   {:.1}%", era.denial_rate());
    println!("{}", "═".repeat(60).bright_blue());
}

fn print_denials(era: &era835::Remittance, summaries: &[era835::DenialSummary]) {
    if summaries.is_empty() {
        println!("{}", "No denied or underpaid claims found.".bright_green());
        return;
    }

    println!("{}", "═".repeat(60).bright_red());
    println!("  Denial Report — {} claim(s) affected", summaries.len());
    println!("  Total at risk: ${:.2}", era.total_denied());
    println!("{}", "═".repeat(60).bright_red());
    println!();

    for (i, d) in summaries.iter().enumerate() {
        let type_label = match d.denial_type {
            era835::DenialType::FullDenial => "FULL DENIAL".bright_red().bold().to_string(),
            era835::DenialType::PartialDenial => "PARTIAL".bright_yellow().bold().to_string(),
            era835::DenialType::Underpayment => "UNDERPAID".bright_yellow().bold().to_string(),
        };

        println!("  {}. Claim {} [{}] — ${:.2} denied", i + 1, d.claim_id.bright_white(), type_label, d.denied_amount);

        for reason in &d.denial_reasons {
            println!("     {}", reason.dimmed());
        }
        for rec in &d.appeal_recommendations {
            println!("     {} {}", "→".bright_yellow(), rec.bright_white());
        }
        println!();
    }
}

fn print_full(era: &era835::Remittance) {
    print_summary(era);
    println!();

    for (i, claim) in era.claims.iter().enumerate() {
        let status = match claim.claim_status_code.as_str() {
            "1" => claim.claim_status_desc.bright_green(),
            "4" => claim.claim_status_desc.bright_red(),
            "22" => claim.claim_status_desc.bright_yellow(),
            _ => claim.claim_status_desc.dimmed(),
        };

        println!("  Claim {} — {} [{}]",
            format!("#{}", i + 1).bright_blue(),
            claim.patient_control_number.bright_white(),
            status
        );
        println!("    Charged: ${:.2}  Paid: ${:.2}  Patient: ${:.2}",
            claim.charge_amount, claim.paid_amount, claim.patient_responsibility);

        if let Some(ref name) = claim.patient_name {
            println!("    Patient: {}", name);
        }

        for svc in &claim.service_lines {
            let mods = if svc.modifiers.is_empty() { String::new() } else { format!(" ({})", svc.modifiers.join(",")) };
            println!("    ├─ {}{}: ${:.2} → ${:.2}",
                svc.procedure_code.bright_cyan(), mods.dimmed(), svc.charge_amount, svc.paid_amount);
            for adj in &svc.adjustments {
                let desc = era835::carc_description(&adj.reason_code).unwrap_or("");
                println!("    │  {} {} ${:.2} — {}",
                    adj.group_code_label().bright_magenta(),
                    adj.reason_code.bright_magenta(),
                    adj.amount, desc.dimmed());
            }
        }
        println!();
    }
}

fn print_phi_results(text: &str, result: &phi_scan::PhiScanResult, file: Option<&PathBuf>) {
    if !result.contains_phi {
        println!("{}", "No PHI detected.".bright_green());
        return;
    }

    let source = file.map(|p| p.display().to_string()).unwrap_or_else(|| "stdin".to_string());
    println!("{}", "═".repeat(60).bright_red());
    println!("  PHI Scan: {} — {} detection(s)", source, result.detections.len());
    println!("{}", "═".repeat(60).bright_red());
    println!();

    let mut by_cat: std::collections::HashMap<String, Vec<&phi_scan::PhiDetection>> = std::collections::HashMap::new();
    for det in &result.detections {
        by_cat.entry(det.category.to_string()).or_default().push(det);
    }

    let mut cats: Vec<&String> = by_cat.keys().collect();
    cats.sort();

    for cat in &cats {
        let dets = &by_cat[*cat];
        println!("  {} ({} found)", cat.as_str().bright_white().bold(), dets.len());
        for det in dets {
            let snippet = &text[det.span.0..det.span.1.min(text.len())];
            let preview = if snippet.len() > 50 { format!("{}...", &snippet[..47]) } else { snippet.to_string() };
            println!("    {}: {}", format!("[{}:{}]", det.span.0, det.span.1).dimmed(), preview.bright_red());
        }
        println!();
    }

    println!("  Total: {} detections across {} categories", result.detections.len(), cats.len());
    println!("{}", "═".repeat(60).bright_red());
}

// ── Helpers ──────────────────────────────────────────────────

fn format_parse_error(file: &PathBuf, error: &era835::Era835Error) -> String {
    match error {
        era835::Era835Error::InvalidFormat(msg) => {
            format!(
                "Invalid 835 format in '{}':\n  {}\n\nThis file does not appear to be a valid X12 835 ERA file.\nCheck that:\n  - The file uses ~ as segment terminator\n  - The file starts with an ISA segment\n  - The file is not corrupted or truncated",
                file.display(), msg
            )
        }
        era835::Era835Error::MissingSegment(seg) => {
            format!(
                "Missing required segment in '{}':\n  Expected segment '{}' but it was not found.\n\nThe 835 file may be incomplete or from an unsupported format variant.",
                file.display(), seg
            )
        }
        era835::Era835Error::SegmentError { segment, detail } => {
            format!(
                "Error parsing segment '{}' in '{}':\n  {}\n\nThe segment may contain unexpected data or be malformed.",
                segment, file.display(), detail
            )
        }
    }
}

fn export_denials_csv(era: &era835::Remittance) {
    println!("claim_id,denial_type,denied_amount,carc_codes,denial_reasons,appeal_recommendations");
    for d in era.denial_summaries() {
        let type_str = match d.denial_type {
            era835::DenialType::FullDenial => "full",
            era835::DenialType::PartialDenial => "partial",
            era835::DenialType::Underpayment => "underpayment",
        };
        let carc = d.carc_codes.join(";");
        let reasons: String = d.denial_reasons.iter()
            .map(|r| r.replace('"', "'"))
            .collect::<Vec<_>>()
            .join("; ");
        let recs: String = d.appeal_recommendations.iter()
            .map(|r| r.replace('"', "'"))
            .collect::<Vec<_>>()
            .join("; ");
        println!(
            "{},{},{:.2},\"{}\",\"{}\",\"{}\"",
            d.claim_id, type_str, d.denied_amount, carc, reasons, recs
        );
    }
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

fn cmd_completions(shell: Shell) {
    let mut cmd = Cli::command();
    match shell {
        Shell::Bash => generate(shells::Bash, &mut cmd, "claims-toolkit", &mut io::stdout()),
        Shell::Zsh => generate(shells::Zsh, &mut cmd, "claims-toolkit", &mut io::stdout()),
        Shell::Fish => generate(shells::Fish, &mut cmd, "claims-toolkit", &mut io::stdout()),
        Shell::PowerShell => generate(shells::PowerShell, &mut cmd, "claims-toolkit", &mut io::stdout()),
        Shell::Elvish => generate(shells::Elvish, &mut cmd, "claims-toolkit", &mut io::stdout()),
    }
}

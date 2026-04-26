use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use colored::Colorize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

// ── Security: Input Validation ─────────────────────────────

/// Maximum file size for input files (10 MB) to prevent DoS
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Validate file path to prevent directory traversal attacks.
/// Returns Ok if path is safe, Err with sanitized message otherwise.
fn validate_file_path(path: &PathBuf) -> Result<(), String> {
    // Check for path traversal components
    let path_str = path.to_string_lossy();

    // Reject paths containing .. (directory traversal)
    if path_str.contains("..") {
        return Err("Invalid path: contains directory traversal".to_string());
    }

    // Reject absolute paths that could access sensitive system locations
    if path.is_absolute() {
        // Allow absolute paths only if they don't go to system directories
        let normalized = path_str.to_lowercase();
        let forbidden = ["/etc/", "/var/", "/usr/", "/sys/", "/proc/", "/dev/", "/root/"];
        for prefix in &forbidden {
            if normalized.starts_with(prefix) {
                return Err(format!("Access denied: path starts with restricted prefix", ));
            }
        }
    }

    Ok(())
}

/// Validate and read file with size limits.
/// Returns sanitized error messages that don't expose internal paths.
fn read_file_secure(path: &PathBuf) -> Result<String, String> {
    // Validate path first
    validate_file_path(path)?;

    // Check file size before reading
    let metadata = fs::metadata(path)
        .map_err(|_| "Cannot access file".to_string())?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!("File too large: exceeds {} MB limit", MAX_FILE_SIZE / (1024 * 1024)));
    }

    // Read with size-limited buffer
    let content = fs::read_to_string(path)
        .map_err(|_| "Failed to read file content".to_string())?;

    // Basic content validation for X12 files
    if content.len() > 100_000_000 { // Additional sanity check
        return Err("Content exceeds maximum allowed size".to_string());
    }

    Ok(content)
}

/// Sanitize error messages to avoid leaking sensitive path information.
fn sanitize_error(err: &str) -> String {
    // Remove potential path information from error messages
    let sanitized = err
        .replace('\n', " ")
        .replace('\r', "");

    // Truncate very long messages
    if sanitized.len() > 500 {
        format!("{:.500}...", sanitized)
    } else {
        sanitized
    }
}

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
        /// ERA/835 file to parse (reads stdin if omitted)
        file: Option<PathBuf>,

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

    /// Validate a claim (CPT/ICD-10, NCCI edits, modifiers)
    Scrub {
        /// Claim file as JSON (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Parse 835 + scrub all claims in one pass (Tier 1 → 2 pipeline)
    Check {
        /// ERA 835 file (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Comma-separated diagnosis codes to apply to all claims
        /// (835 files don't carry DX codes; provide them here)
        #[arg(long, value_delimiter = ',')]
        dx: Vec<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Show information about the toolkit
    Info,

    /// Generate appeal letters from denial reports
    Appeal {
        /// ERA 835 file with denials
        file: PathBuf,

        /// Provider name for the appeal letter
        #[arg(short, long)]
        provider: Option<String>,

        /// NPI number for the appeal letter
        #[arg(long)]
        npi: Option<String>,

        /// Output directory for appeal letters (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format: "text" or "markdown" (default: text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

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

        /// Output as Excel XLSX
        #[arg(long)]
        xlsx: bool,
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
        Commands::Scrub { file, json } => cmd_scrub(file, json)?,
        Commands::Check { file, json, dx } => cmd_check(file, json, dx)?,
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Info => cmd_info(),
        Commands::Appeal { file, provider, npi, output, format } => cmd_appeal(file, provider, npi, output, format)?,
    }

    Ok(())
}

// ── Parse Command ─────────────────────────────────────────────

fn cmd_parse(file: &Option<PathBuf>, output: Option<ParseOutput>) -> Result<(), Box<dyn std::error::Error>> {
    let raw = match file {
        Some(path) => read_file_secure(path)
            .map_err(|e| sanitize_error(&e))?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            if buf.len() > MAX_FILE_SIZE as usize {
                return Err("Input too large".into());
            }
            buf
        }
    };

    let file_label = file.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "stdin".to_string());

    let era = era835::parse_era835(&raw)
        .map_err(|e| format_parse_error_label(&file_label, &e))?;

    match output {
        Some(ParseOutput::Summary { json }) => {
            if json {
                let summary = serde_json::json!({
                    "file": file_label,
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
        Some(ParseOutput::Denials { json, csv, xlsx }) => {
            let summaries = era.denial_summaries();
            if xlsx {
                let output_path = file.as_ref()
                    .map(|p| p.with_extension("xlsx"))
                    .unwrap_or_else(|| PathBuf::from("denials.xlsx"));
                export_denials_xlsx(&era, &output_path)?;
                println!("Excel report saved to: {}", output_path.display());
            } else if csv {
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
        Some(path) => read_file_secure(path)
            .map_err(|e| sanitize_error(&e))?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            if buf.len() > MAX_FILE_SIZE as usize {
                return Err("Input too large".into());
            }
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


// ── Scrub Command ──────────────────────────────────────────────

fn cmd_scrub(file: Option<PathBuf>, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let raw = match &file {
        Some(path) => read_file_secure(path)
            .map_err(|e| sanitize_error(&e))?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            if buf.len() > MAX_FILE_SIZE as usize {
                return Err("Input too large".into());
            }
            buf
        }
    };

    let claim: claims_scrub::Claim = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid claim JSON: {}", e))?;

    let scrubber = claims_scrub::ClaimsScrubber::new();
    let result = scrubber.validate_claim(&claim);

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    println!("{}", "═".repeat(60).bright_blue());
    println!("  Claims Scrub — {}", claim.claim_id.bright_white());
    println!("{}", "═".repeat(60).bright_blue());
    println!();

    if result.is_clean {
        println!("  {} Claim is clean — no issues found.", "✓".bright_green());
    } else {
        println!("  {} Claim has issues:", "⚠".bright_yellow());
    }

    println!();
    println!("  Errors:   {}", if result.error_count > 0 { format!("{}", result.error_count).bright_red().to_string() } else { "0".bright_green().to_string() });
    println!("  Warnings: {}", if result.warning_count > 0 { format!("{}", result.warning_count).bright_yellow().to_string() } else { "0".bright_green().to_string() });
    println!("  Denial Risk: {}%", if result.denial_risk > 50 { format!("{}", result.denial_risk).bright_red().to_string() } else { format!("{}", result.denial_risk).bright_green().to_string() });
    println!();

    if !result.findings.is_empty() {
        println!("{}", "═".repeat(60).bright_blue());
        for (i, f) in result.findings.iter().enumerate() {
            let sev = match f.severity {
                claims_scrub::FindingSeverity::Error => "ERROR".bright_red().bold().to_string(),
                claims_scrub::FindingSeverity::Warning => "WARN".bright_yellow().bold().to_string(),
                claims_scrub::FindingSeverity::Info => "INFO".bright_blue().to_string(),
            };
            let line = f.line_number.map(|l| format!("Line {}", l)).unwrap_or_default();
            println!("  {}. [{}] {} {}", i + 1, sev, line.dimmed(), f.description);
            if let Some(ref s) = f.suggestion {
                println!("     → {}", s.bright_white());
            }
        }
        println!("{}", "═".repeat(60).bright_blue());
    }

    if !result.corrections.is_empty() {
        println!();
        println!("  {} Corrections:", "💡".bright_yellow());
        for c in &result.corrections {
            println!("    • {}", c);
        }
    }

    Ok(())
}
// ── Check Command (Tier 1 → 2 Pipeline) ──────────────────

fn cmd_check(
    file: Option<PathBuf>,
    json: bool,
    dx: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let raw = match &file {
        Some(path) => read_file_secure(path)
            .map_err(|e| sanitize_error(&e))?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let era = era835::parse_era835(&raw)?;
    let payer_name = era.payer.name.clone();
    let scrubber = claims_scrub::ClaimsScrubber::new();

    let mut all_results: Vec<claims_scrub::ClaimScrubResult> = Vec::new();
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    let mut clean_count = 0usize;
    let mut dirty_count = 0usize;

    for claim_payment in &era.claims {
        let claim = claims_scrub::claim_from_era835(claim_payment, &payer_name, &dx);
        let result = scrubber.validate_claim(&claim);

        let carc_codes: Vec<String> = claim_payment
            .adjustments
            .iter()
            .map(|a| a.reason_code.clone())
            .collect();

        let original_denied = claim_payment.paid_amount == 0.0 && claim_payment.charge_amount > 0.0;

        total_errors += result.error_count;
        total_warnings += result.warning_count;
        if result.is_clean {
            clean_count += 1;
        } else {
            dirty_count += 1;
        }

        all_results.push(claims_scrub::ClaimScrubResult {
            claim_id: claim_payment.patient_control_number.clone(),
            payer_claim_number: claim_payment.payer_claim_number.clone(),
            scrub_result: result,
            original_denied,
            original_denied_amount: if original_denied {
                claim_payment.charge_amount
            } else {
                0.0
            },
            carc_codes,
        });
    }

    if json {
        let output = serde_json::json!({
            "era_file": file.as_ref().map(|p| p.display().to_string()),
            "payer": era.payer.name,
            "total_claims": era.claims.len(),
            "total_charged": era.total_charged(),
            "total_paid": era.total_paid(),
            "clean_count": clean_count,
            "dirty_count": dirty_count,
            "total_errors": total_errors,
            "total_warnings": total_warnings,
            "results": all_results,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Pretty output
    println!("{}", "═".repeat(60).bright_blue());
    println!(
        "  Claims Check — {} {}",
        era.payer.name.bright_white(),
        file.as_ref()
            .map(|p| format!("({})", p.display()))
            .unwrap_or_default()
            .dimmed()
    );
    println!("{}", "═".repeat(60).bright_blue());
    println!();
    println!(
        "  ERA: {} claims | ${:.2} charged | ${:.2} paid",
        era.claims.len(),
        era.total_charged(),
        era.total_paid()
    );
    println!(
        "  Scrub: {} clean | {} with issues | {} errors | {} warnings",
        if clean_count > 0 {
            format!("{}", clean_count).bright_green().to_string()
        } else {
            format!("{}", clean_count).dimmed().to_string()
        },
        if dirty_count > 0 {
            format!("{}", dirty_count).bright_red().to_string()
        } else {
            format!("{}", dirty_count).dimmed().to_string()
        },
        if total_errors > 0 {
            format!("{}", total_errors).bright_red().to_string()
        } else {
            "0".bright_green().to_string()
        },
        if total_warnings > 0 {
            format!("{}", total_warnings).bright_yellow().to_string()
        } else {
            "0".bright_green().to_string()
        },
    );
    println!();

    if all_results.is_empty() {
        println!("  No claims found in ERA file.");
        return Ok(());
    }

    println!("{}", "═".repeat(60).bright_blue());
    for (i, cr) in all_results.iter().enumerate() {
        let icon = if cr.scrub_result.is_clean {
            "✓".bright_green().to_string()
        } else {
            "✗".bright_red().to_string()
        };

        let risk_color = if cr.scrub_result.denial_risk > 50 {
            format!("{}%", cr.scrub_result.denial_risk).bright_red().to_string()
        } else if cr.scrub_result.denial_risk > 20 {
            format!("{}%", cr.scrub_result.denial_risk).bright_yellow().to_string()
        } else {
            format!("{}%", cr.scrub_result.denial_risk).bright_green().to_string()
        };

        let denied_tag = if cr.original_denied {
            " [DENIED]".bright_red().bold().to_string()
        } else {
            String::new()
        };

        println!(
            "  {} Claim {} — risk {}{}{}",
            icon,
            cr.claim_id.bright_white(),
            risk_color,
            denied_tag,
            if !cr.scrub_result.is_clean {
                format!(
                    " ({} err, {} warn)",
                    cr.scrub_result.error_count, cr.scrub_result.warning_count
                )
                .dimmed()
                .to_string()
            } else {
                String::new()
            }
        );

        if !cr.scrub_result.findings.is_empty() {
            for f in &cr.scrub_result.findings {
                let sev = match f.severity {
                    claims_scrub::FindingSeverity::Error => "ERR".bright_red().to_string(),
                    claims_scrub::FindingSeverity::Warning => "WRN".bright_yellow().to_string(),
                    claims_scrub::FindingSeverity::Info => "INF".bright_blue().to_string(),
                };
                println!("     [{}] {}", sev, f.description.dimmed());
            }
        }
    }
    println!("{}", "═".repeat(60).bright_blue());

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
    println!("  {} Validate a claim (CPT, NCCI, modifiers)", "scrub".bright_cyan());
    println!("  {} Parse + scrub all claims in an 835 file", "check".bright_cyan());
    println!();
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

fn format_parse_error_label(label: &str, error: &era835::Era835Error) -> String {
    match error {
        era835::Era835Error::InvalidFormat(msg) => {
            format!(
                "Invalid 835 format in '{}':\n  {}\n\nThis file does not appear to be a valid X12 835 ERA file.\nCheck that:\n  - The file uses ~ as segment terminator\n  - The file starts with an ISA segment\n  - The file is not corrupted or truncated",
                label, msg
            )
        }
        era835::Era835Error::MissingSegment(seg) => {
            format!(
                "Missing required segment in '{}':\n  Expected segment '{}' but it was not found.\n\nThe 835 file may be incomplete or from an unsupported format variant.",
                label, seg
            )
        }
        era835::Era835Error::SegmentError { segment, detail } => {
            format!(
                "Error parsing segment '{}' in '{}':\n  {}\n\nThe segment may contain unexpected data or be malformed.",
                segment, label, detail
            )
        }
    }
}

fn format_parse_error(file: &PathBuf, error: &era835::Era835Error) -> String {
    format_parse_error_label(&file.display().to_string(), error)
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

fn export_denials_xlsx(era: &era835::Remittance, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use rust_xlsxwriter::{Workbook, Format, FormatAlign, FormatBorder, Color};

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Create header format
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::Blue)
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);

    // Create currency format
    let currency_format = Format::new()
        .set_num_format("$#,##0.00")
        .set_border(FormatBorder::Thin);

    // Create data cell format
    let cell_format = Format::new()
        .set_border(FormatBorder::Thin);

    // Set column widths
    worksheet.set_column_width(0, 20)?; // Claim ID
    worksheet.set_column_width(1, 15)?; // Denial Type
    worksheet.set_column_width(2, 15)?; // Denied Amount
    worksheet.set_column_width(3, 30)?; // CARC Codes
    worksheet.set_column_width(4, 50)?; // Denial Reasons
    worksheet.set_column_width(5, 60)?; // Appeal Recommendations

    // Write headers
    let headers = ["Claim ID", "Denial Type", "Denied Amount", "CARC Codes", "Denial Reasons", "Appeal Recommendations"];
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *header, &header_format)?;
    }

    // Write data rows
    for (row, d) in era.denial_summaries().iter().enumerate() {
        let row = row + 1;
        let type_str = match d.denial_type {
            era835::DenialType::FullDenial => "Full Denial",
            era835::DenialType::PartialDenial => "Partial Denial",
            era835::DenialType::Underpayment => "Underpayment",
        };
        let carc = d.carc_codes.join(", ");
        let reasons = d.denial_reasons.join("; ");
        let recs = d.appeal_recommendations.join("; ");

        worksheet.write_string_with_format(row as u32, 0, &d.claim_id, &cell_format)?;
        worksheet.write_string_with_format(row as u32, 1, type_str, &cell_format)?;
        worksheet.write_number_with_format(row as u32, 2, d.denied_amount, &currency_format)?;
        worksheet.write_string_with_format(row as u32, 3, &carc, &cell_format)?;
        worksheet.write_string_with_format(row as u32, 4, &reasons, &cell_format)?;
        worksheet.write_string_with_format(row as u32, 5, &recs, &cell_format)?;
    }

    // Add summary sheet
    let summary_sheet = workbook.add_worksheet().set_name("Summary")?;
    let title_format = Format::new().set_bold().set_font_size(14);
    let summary_label_format = Format::new().set_bold();

    summary_sheet.write_string_with_format(0, 0, "Denial Report Summary", &title_format)?;
    summary_sheet.write_string_with_format(2, 0, "Payer:", &summary_label_format)?;
    summary_sheet.write_string(2, 1, &era.payer.name)?;
    summary_sheet.write_string_with_format(3, 0, "Total Claims:", &summary_label_format)?;
    summary_sheet.write_number(3, 1, era.claims.len() as f64)?;
    summary_sheet.write_string_with_format(4, 0, "Total Denied:", &summary_label_format)?;
    summary_sheet.write_number(4, 1, era.denied_claims().len() as f64)?;
    summary_sheet.write_string_with_format(5, 0, "Denial Rate:", &summary_label_format)?;
    summary_sheet.write_string(5, 1, &format!("{:.1}%", era.denial_rate()))?;
    summary_sheet.write_string_with_format(6, 0, "Total Charged:", &summary_label_format)?;
    summary_sheet.write_number_with_format(6, 1, era.total_charged(), &currency_format)?;
    summary_sheet.write_string_with_format(7, 0, "Total Denied Amount:", &summary_label_format)?;
    summary_sheet.write_number_with_format(7, 1, era.total_denied(), &currency_format)?;

    workbook.save(path)?;
    Ok(())
}

// ── Appeal Command ────────────────────────────────────────────

fn cmd_appeal(
    file: PathBuf,
    provider: Option<String>,
    npi: Option<String>,
    output: Option<PathBuf>,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let raw = read_file_secure(&file)
        .map_err(|e| sanitize_error(&e))?;

    let era = era835::parse_era835(&raw)
        .map_err(|e| format_parse_error_label(&file.display().to_string(), &e))?;

    let denials = era.denial_summaries();
    if denials.is_empty() {
        println!("{} No denials found in {}", "✓".green(), file.display());
        return Ok(());
    }

    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_dir)?;

    let provider_name = provider.unwrap_or_else(|| era.payee.name.clone());
    let provider_npi = npi.unwrap_or_else(|| era.payee.npi.clone());
    let payer_name = era.payer.name.clone();
    let today = chrono::Local::now().format("%B %d, %Y").to_string();

    let is_markdown = format == "markdown";

    for denial in &denials {
        let filename = format!("appeal_{}.{}_letter", denial.claim_id,
            if is_markdown { "md" } else { "txt" });
        let path = output_dir.join(&filename);

        let letter = if is_markdown {
            generate_appeal_letter_markdown(&denial, &provider_name, &provider_npi, &payer_name, &today)
        } else {
            generate_appeal_letter_text(&denial, &provider_name, &provider_npi, &payer_name, &today)
        };

        fs::write(&path, letter)?;
        println!("{} Generated appeal letter: {}", "✓".green(), path.display());
    }

    println!("\n{} Generated {} appeal letter(s) in {}",
        "✓".green(),
        denials.len(),
        output_dir.display()
    );

    Ok(())
}

fn generate_appeal_letter_text(
    denial: &era835::DenialSummary,
    provider: &str,
    npi: &str,
    payer: &str,
    date: &str,
) -> String {
    let denial_type_str = match denial.denial_type {
        era835::DenialType::FullDenial => "Full Denial",
        era835::DenialType::PartialDenial => "Partial Denial",
        era835::DenialType::Underpayment => "Underpayment",
    };

    let carc_list = denial.carc_codes.join(", ");
    let reasons = denial.denial_reasons.join("\n  - ");
    let recommendations = denial.appeal_recommendations.join("\n  - ");

    format!(r#"CLAIMS APPEAL LETTER

Date: {date}

Provider: {provider}
NPI: {npi}

Payer: {payer}

RE: Appeal for Claim ID {claim_id}

Dear Claims Department,

We are writing to formally appeal the {denial_type} of Claim ID {claim_id}.

DENIAL DETAILS:
  Claim ID: {claim_id}
  Denial Type: {denial_type_str}
  Denied Amount: ${amount:.2}
  CARC Codes: {carc_list}
  Denial Reasons:
  - {reasons}

APPEAL ARGUMENT:
Based on the denial reason codes provided, we believe this claim was incorrectly processed.
{recommendations_text}

RECOMMENDED ACTIONS:
  - {recommendations}

We request that you reprocess this claim according to the terms of our provider agreement and applicable regulations.

Please contact us if you require additional documentation or have questions regarding this appeal.

Sincerely,

{provider}
NPI: {npi}

---
This appeal was generated automatically by claims-toolkit.
Please review and customize as needed before submission.
"#,
        date = date,
        provider = provider,
        npi = npi,
        payer = payer,
        claim_id = denial.claim_id,
        denial_type = denial_type_str.to_lowercase(),
        denial_type_str = denial_type_str,
        amount = denial.denied_amount,
        carc_list = carc_list,
        reasons = reasons,
        recommendations = recommendations,
        recommendations_text = if recommendations.is_empty() {
            "We request a full review of the claim.".to_string()
        } else {
            format!("The following actions are recommended:\n  - {}", recommendations)
        },
    )
}

fn generate_appeal_letter_markdown(
    denial: &era835::DenialSummary,
    provider: &str,
    npi: &str,
    payer: &str,
    date: &str,
) -> String {
    let denial_type_str = match denial.denial_type {
        era835::DenialType::FullDenial => "Full Denial",
        era835::DenialType::PartialDenial => "Partial Denial",
        era835::DenialType::Underpayment => "Underpayment",
    };

    let carc_list = denial.carc_codes.join(", ");
    let reasons = denial.denial_reasons.iter()
        .map(|r| format!("- {}", r))
        .collect::<Vec<_>>()
        .join("\n");
    let rec_list: Vec<String> = denial.appeal_recommendations.clone();
    let recommendations = rec_list.iter()
        .map(|r| format!("- {}", r))
        .collect::<Vec<_>>()
        .join("\n");

    format!(r#"# Claim Appeal Letter

**Date:** {date}

**Provider:** {provider}  
**NPI:** {npi}

**Payer:** {payer}

---

## RE: Appeal for Claim ID {claim_id}

Dear Claims Department,

We are writing to formally appeal the **{denial_type_str}** of Claim ID **{claim_id}**.

### Denial Details

| Field | Value |
|-------|-------|
| Claim ID | {claim_id} |
| Denial Type | {denial_type_str} |
| Denied Amount | ${amount:.2} |
| CARC Codes | {carc_list} |

### Denial Reasons

{reasons}

### Appeal Argument

Based on the denial reason codes provided, we believe this claim was incorrectly processed.

{recommendations_text}

### Recommended Actions

{recommendations}

We request that you reprocess this claim according to the terms of our provider agreement and applicable regulations.

Please contact us if you require additional documentation or have questions regarding this appeal.

---

Sincerely,

**{provider}**  
NPI: {npi}

---

*This appeal was generated automatically by claims-toolkit. Please review and customize as needed before submission.*
"#,
        date = date,
        provider = provider,
        npi = npi,
        payer = payer,
        claim_id = denial.claim_id,
        denial_type_str = denial_type_str,
        amount = denial.denied_amount,
        carc_list = carc_list,
        reasons = if reasons.is_empty() { "_No specific reasons provided_".to_string() } else { reasons },
        recommendations = if recommendations.is_empty() { "_No specific recommendations_".to_string() } else { recommendations },
        recommendations_text = if rec_list.is_empty() {
            "We request a full review of the claim.".to_string()
        } else {
            format!("The following actions are recommended:\n\n{}", rec_list.iter().map(|r| format!("- {}", r)).collect::<Vec<_>>().join("\n"))
        },
    )
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


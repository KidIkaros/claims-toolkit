use clap::{Parser, Subcommand};
use colored::Colorize;
use era835::*;
use serde_json::json;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// X12 835 Electronic Remittance Advice (ERA) parser.
///
/// Parses ERA/835 files from payers to extract claim payment details,
/// denials, adjustments, and remark codes.
#[derive(Parser)]
#[command(name = "era835", version, about)]
struct Cli {
    /// Input file (reads stdin if omitted)
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show denial summary for denied/underpaid claims
    Denials {
        /// Input file (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show financial summary (totals, denial rate)
    Summary {
        /// Input file (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Parse and output full remittance as JSON
    Json {
        /// Input file (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Pretty-print JSON
        #[arg(long, default_value_t = true)]
        pretty: bool,
    },
}

fn read_input(file: Option<&PathBuf>) -> Result<String, Box<dyn std::error::Error>> {
    match file {
        Some(path) => Ok(fs::read_to_string(path)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let file = cli.file.as_ref();
    let raw = match &cli.command {
        Some(Commands::Denials { file: f, .. }) => read_input(file.or(f.as_ref()))?,
        Some(Commands::Summary { file: f, .. }) => read_input(file.or(f.as_ref()))?,
        Some(Commands::Json { file: f, .. }) => read_input(file.or(f.as_ref()))?,
        None => read_input(file)?,
    };

    let remittance = parse_era835(&raw)?;

    match &cli.command {
        Some(Commands::Denials { json, .. }) => {
            let summaries = remittance.denial_summaries();
            if *json {
                println!("{}", serde_json::to_string_pretty(&summaries)?);
            } else {
                print_denials(&remittance, &summaries);
            }
        }
        Some(Commands::Summary { json, .. }) => {
            if *json {
                let summary = json!({
                    "payer": remittance.payer.name,
                    "payee": remittance.payee.name,
                    "total_claims": remittance.claims.len(),
                    "total_charged": remittance.total_charged(),
                    "total_paid": remittance.total_paid(),
                    "total_denied": remittance.total_denied(),
                    "denial_rate_pct": remittance.denial_rate(),
                    "denied_claims": remittance.denied_claims().len(),
                });
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                print_summary(&remittance);
            }
        }
        Some(Commands::Json { pretty, .. }) => {
            if *pretty {
                println!("{}", serde_json::to_string_pretty(&remittance)?);
            } else {
                println!("{}", serde_json::to_string(&remittance)?);
            }
        }
        None => {
            // Default: print human-readable summary
            print_full(&remittance);
        }
    }

    Ok(())
}

fn print_summary(r: &Remittance) {
    println!("{}", "═".repeat(60).bright_blue());
    println!("  {} ERA 835 Summary", "▶".bright_green());
    println!("{}", "═".repeat(60).bright_blue());
    println!();
    println!("  {} {}", "Payer:".bright_white(), r.payer.name);
    println!("  {} {}", "Payee:".bright_white(), r.payee.name);
    println!("  {} {}", "NPI:".bright_white(), r.payee.npi);
    if let Some(ref trace) = r.trace_number {
        println!("  {} {}", "Trace:".bright_white(), trace);
    }
    println!();
    println!("  {} {}", "Total Claims:".bright_white(), r.claims.len());
    println!(
        "  {} ${:.2}",
        "Total Charged:".bright_white(),
        r.total_charged()
    );
    println!("  {} ${:.2}", "Total Paid:".bright_green(), r.total_paid());
    println!(
        "  {} ${:.2}",
        "Total Denied:".bright_red(),
        r.total_denied()
    );
    println!(
        "  {} {:.1}%",
        "Denial Rate:".bright_yellow(),
        r.denial_rate()
    );
    println!();

    let denied = r.denied_claims();
    if !denied.is_empty() {
        println!(
            "  {} {} denied/underpaid claim(s)",
            "⚠".bright_yellow(),
            denied.len()
        );
    }
    println!("{}", "═".repeat(60).bright_blue());
}

fn colorize_group(code: &str) -> colored::ColoredString {
    match code {
        "CO" => "CO".bright_red(),
        "PR" => "PR".bright_yellow(),
        "OA" => "OA".bright_blue(),
        "PI" => "PI".bright_magenta(),
        "CR" => "CR".bright_green(),
        _ => code.dimmed(),
    }
}

fn print_denials(_r: &Remittance, summaries: &[DenialSummary]) {
    if summaries.is_empty() {
        println!("{}", "✓ No denied or underpaid claims found.".bright_green());
        return;
    }

    println!("{}", "═".repeat(60).bright_red());
    println!("  {} Denial Report — {} claim(s)", "✕".bright_red(), summaries.len());
    println!("{}", "═".repeat(60).bright_red());
    println!();

    for (i, denial) in summaries.iter().enumerate() {
        let type_label = match denial.denial_type {
            DenialType::FullDenial => "FULL DENIAL".bright_red().bold(),
            DenialType::PartialDenial => "PARTIAL DENIAL".bright_yellow().bold(),
            DenialType::Underpayment => "UNDERPAYMENT".bright_yellow().bold(),
        };

        println!("  {}. Claim: {} [{}]", i + 1, denial.claim_id.bright_white(), type_label);
        println!("     Denied: ${:.2}", denial.denied_amount);

        if !denial.carc_codes.is_empty() {
            println!("     CARC codes: {}", denial.carc_codes.join(", ").bright_cyan());
        }

        for reason in &denial.denial_reasons {
            println!("     → {}", reason.dimmed());
        }

        for rec in &denial.appeal_recommendations {
            println!("     {} {}", "💡".bright_yellow(), rec.bright_white());
        }
        println!();
    }
}

fn print_full(r: &Remittance) {
    print_summary(r);
    println!();

    for (i, claim) in r.claims.iter().enumerate() {
        let status_color = match claim.claim_status_code.as_str() {
            "4" => format!("{} {}", "●".bright_red(), claim.claim_status_desc.bright_red()),
            "1" => format!("{} {}", "●".bright_green(), claim.claim_status_desc.bright_green()),
            "22" => format!("{} {}", "●".bright_yellow(), claim.claim_status_desc.bright_yellow()),
            _ => format!("{} {}", "●".bright_white(), claim.claim_status_desc),
        };

        println!(
            "  Claim {}: {} {}",
            format!("#{}", i + 1).bright_blue(),
            claim.patient_control_number.bright_white(),
            status_color
        );
        println!(
            "    Charged: ${:.2}  Paid: ${:.2}  Patient Resp: ${:.2}",
            claim.charge_amount, claim.paid_amount, claim.patient_responsibility
        );

        if let Some(ref name) = claim.patient_name {
            println!("    Patient: {}", name);
        }

        for svc in &claim.service_lines {
            println!(
                "    ├─ {} ({}): charged ${:.2}, paid ${:.2}, units {}",
                svc.procedure_code.bright_cyan(),
                if svc.modifiers.is_empty() {
                    "no modifier".dimmed().to_string()
                } else {
                    svc.modifiers.join(",").bright_yellow().to_string()
                },
                svc.charge_amount,
                svc.paid_amount,
                svc.units
            );
            for adj in &svc.adjustments {
                let desc = carc_description(&adj.reason_code)
                    .unwrap_or("unknown");
                println!(
                    "    │  {} {} {} ${:.2}",
                    colorize_group(adj.group_code_label()),
                    adj.reason_code.bright_magenta(),
                    desc.dimmed(),
                    adj.amount
                );
            }
        }

        for adj in &claim.adjustments {
            let desc = carc_description(&adj.reason_code).unwrap_or("unknown");
            println!(
                "    {} {} {} ${:.2}",
                colorize_group(adj.group_code_label()),
                adj.reason_code.bright_magenta(),
                desc.dimmed(),
                adj.amount
            );
        }
        println!();
    }

    if !r.provider_adjustments.is_empty() {
        println!("  {} Provider-Level Adjustments:", "ℹ".bright_blue());
        for adj in &r.provider_adjustments {
            println!(
                "    {} ${:.2} (reason: {})",
                adj.provider_id.bright_white(),
                adj.amount,
                adj.reason_code
            );
        }
    }
}

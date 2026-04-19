use clap::{Parser, Subcommand};
use colored::Colorize;
use phi_scan::*;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// PHI Scanner — detect and redact Protected Health Information.
#[derive(Parser)]
#[command(name = "phi-scan", version, about)]
struct Cli {
    /// Input file (reads stdin if omitted)
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan text for PHI and report detections
    Scan {
        /// Input file (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Redact PHI, replacing matches with [REDACTED:category]
    Redact {
        /// Input file (reads stdin if omitted)
        file: Option<PathBuf>,
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
        Some(Commands::Scan { file: f, .. }) => read_input(file.or(f.as_ref()))?,
        Some(Commands::Redact { file: f }) => read_input(file.or(f.as_ref()))?,
        None => read_input(file)?,
    };

    match &cli.command {
        Some(Commands::Scan { json, .. }) => {
            let result = scan_phi(&raw);
            if *json {
                let detections: Vec<_> = result.detections.iter().map(|d| {
                    serde_json::json!({
                        "category": d.category.to_string(),
                        "start": d.span.0,
                        "end": d.span.1,
                        "text": &raw[d.span.0..d.span.1],
                    })
                }).collect();
                let output = serde_json::json!({
                    "contains_phi": result.contains_phi,
                    "total_detections": result.detections.len(),
                    "detections": detections,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print_scan_results(&raw, &result);
            }
        }
        Some(Commands::Redact { .. }) => {
            let redacted = redact_phi(&raw);
            print!("{}", redacted);
        }
        None => {
            let result = scan_phi(&raw);
            print_scan_results(&raw, &result);
        }
    }

    Ok(())
}

fn print_scan_results(text: &str, result: &PhiScanResult) {
    if !result.contains_phi {
        println!("{}", "No PHI detected.".bright_green());
        return;
    }

    println!("{}", "=".repeat(60).bright_red());
    println!("  PHI Scan Report — {} detection(s)", result.detections.len());
    println!("{}", "=".repeat(60).bright_red());
    println!();

    let mut by_category: std::collections::HashMap<String, Vec<&PhiDetection>> = std::collections::HashMap::new();
    for det in &result.detections {
        by_category.entry(det.category.to_string()).or_default().push(det);
    }

    let mut categories: Vec<&String> = by_category.keys().collect();
    categories.sort();

    for cat in &categories {
        let dets = &by_category[*cat];
        println!("  {} ({} found)", cat.as_str().bright_white().bold(), dets.len());
        for det in dets {
            let snippet = &text[det.span.0..det.span.1.min(text.len())];
            let preview = if snippet.len() > 60 { format!("{}...", &snippet[..57]) } else { snippet.to_string() };
            println!("    [{}:{}] {}", det.span.0, det.span.1, preview.bright_red());
        }
        println!();
    }

    println!("{}", "=".repeat(60).bright_red());
    println!("  Summary:");
    for cat in &categories {
        println!("    {}: {}", cat, by_category[*cat].len());
    }
    println!("{}", "=".repeat(60).bright_red());
}

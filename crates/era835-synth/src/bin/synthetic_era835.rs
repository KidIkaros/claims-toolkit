use clap::Parser;
use era835_synth::*;
use std::fs;
use std::path::PathBuf;

/// Synthetic Claims Engine — generate realistic X12 835 ERA files for testing.
#[derive(Parser)]
#[command(name = "synthetic-era835", version, about)]
struct Cli {
    /// Number of claims to generate
    #[arg(short, long, default_value_t = 5)]
    count: usize,

    /// Output file path (prints to stdout if omitted)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Random seed for reproducibility
    #[arg(short, long)]
    seed: Option<u64>,

    /// Output as JSON instead of X12 text
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let batch = generate_synthetic_era835(cli.count, cli.seed);

    let output = if cli.json {
        serde_json::to_string_pretty(&batch).expect("JSON serialization failed")
    } else {
        serialize_era835(&batch)
    };

    match cli.output {
        Some(path) => {
            fs::write(&path, &output).expect("Failed to write output file");
            eprintln!("Generated {} claims -> {}", cli.count, path.display());
        }
        None => {
            print!("{}", output);
        }
    }
}

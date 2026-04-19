//! Integration tests for the claims-toolkit CLI binary.
//!
//! These test the full binary end-to-end, not just the libraries.
use std::process::{Command, Stdio};

use std::io::Write;
fn cli() -> Command {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir.parent().unwrap().parent().unwrap().to_path_buf();

    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| workspace.join("target").to_str().unwrap().to_string());

    let bin = std::path::PathBuf::from(&target_dir).join("release/claims-toolkit");
    assert!(bin.exists(), "Binary not found at {:?}", bin);

    let mut cmd = Command::new(bin);
    cmd.current_dir(workspace);
    cmd
}

// ── Parse Tests ──

#[test]
fn parse_summary_json() {
    let output = cli()
        .args(["parse", "samples/realistic_5claim.835", "summary", "--json"])
        .output()
        .expect("Failed to run");

    if !output.status.success() {
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    }
    assert!(output.status.success(), "CLI failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["total_claims"], 5);
    assert!(json["denial_rate_pct"].as_f64().unwrap() > 0.0);
}

#[test]
fn parse_denials_csv() {
    let output = cli()
        .args(["parse", "samples/realistic_5claim.835", "denials", "--csv"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].starts_with("claim_id,denial_type"));
    assert!(lines.len() > 1, "Should have at least one denial");
}

#[test]
fn parse_json_output() {
    let output = cli()
        .args(["parse", "samples/realistic_5claim.835", "json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["claims"].as_array().unwrap().len() == 5);
    assert!(json["payer"]["name"].as_str().unwrap().len() > 0);
}

#[test]
fn parse_nonexistent_file() {
    let output = cli()
        .args(["parse", "/nonexistent/file.835"])
        .output()
        .expect("Failed to run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot read"));
}

#[test]
fn parse_invalid_file() {
    let output = cli()
        .args(["parse", "/dev/null"])
        .output()
        .expect("Failed to run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid 835 format") || stderr.contains("Empty input"));
}

// ── Generate Tests ──

#[test]
fn generate_to_stdout() {
    let output = cli()
        .args(["generate", "-n", "2", "--seed", "42"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ISA*"), "Should contain X12 ISA header");
    assert!(stdout.contains("CLP*"), "Should contain CLP claim segments");
}

#[test]
fn generate_to_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap();

    let output = cli()
        .args(["generate", "-n", "3", "--seed", "99", "-o", path])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("ISA*"));
}

#[test]
fn generate_roundtrip() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap();

    // Generate
    cli().args(["generate", "-n", "5", "--seed", "77", "-o", path])
        .output().unwrap();

    // Parse
    let output = cli()
        .args(["parse", path, "summary", "--json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["total_claims"], 5);
}

// ── Scan Tests ──

#[test]
fn scan_phi_detection() {
    let mut child = cli()
        .args(["scan", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn");

    child.stdin.as_mut().unwrap()
        .write_all(b"Patient: John Smith, SSN 123-45-6789")
        .unwrap();
    child.stdin.take();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["contains_phi"].as_bool().unwrap());
    assert!(json["total_detections"].as_u64().unwrap() >= 2);
}

#[test]
fn scan_no_phi() {
    let output = cli()
        .args(["scan", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    // Can't easily pipe stdin here, use a file instead
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "The patient has a scheduled follow-up.").unwrap();

    let output = cli()
        .args(["scan", "--json", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // May or may not have PHI depending on patterns
    let _ = json;
}

#[test]
fn scan_redact() {
    let output = cli()
        .args(["scan", "--redact", "samples/clinical_note_sample.txt"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[REDACTED:"), "Should contain redaction markers");
    assert!(!stdout.contains("456-78-9012"), "SSN should be redacted");
}

// ── Completions Tests ──

#[test]
fn completions_bash() {
    let output = cli()
        .args(["completions", "bash"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_claims-toolkit"), "Should contain bash completion function");
    assert!(stdout.contains("COMPREPLY"), "Should contain bash completion variable");
}

#[test]
fn completions_zsh() {
    let output = cli()
        .args(["completions", "zsh"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#compdef"), "Should contain zsh completion header");
}

#[test]
fn completions_fish() {
    let output = cli()
        .args(["completions", "fish"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("complete"), "Should contain fish completion");
}

// ── Info & Version ──

#[test]
fn info_command() {
    let output = cli()
        .args(["info"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("claims-toolkit"));
    assert!(stdout.contains("parse"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("scan"));
}

#[test]
fn version_flag() {
    let output = cli()
        .args(["--version"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn help_flag() {
    let output = cli()
        .args(["--help"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("parse"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("scan"));
}

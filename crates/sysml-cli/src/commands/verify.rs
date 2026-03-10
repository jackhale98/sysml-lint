/// Verification domain CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::VerifyCommand;

pub fn run(cli: &crate::Cli, kind: &VerifyCommand) -> ExitCode {
    match kind {
        VerifyCommand::Coverage { files } => run_coverage(cli, files),
        VerifyCommand::List { files } => run_list(cli, files),
        VerifyCommand::Status { files } => run_status(cli, files),
        VerifyCommand::Add { file, inside } => run_add(file.as_ref(), inside.as_deref()),
    }
}

fn run_coverage(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // Extract verification relationships from model
    let mut total_reqs = 0usize;
    let mut verified_reqs = 0usize;

    for model in &models {
        let req_names: Vec<&str> = model
            .definitions
            .iter()
            .filter(|d| d.kind == sysml_core::model::DefKind::Requirement)
            .map(|d| d.name.as_str())
            .collect();

        let verified_names: std::collections::HashSet<&str> = model
            .verifications
            .iter()
            .map(|v| v.requirement.as_str())
            .collect();

        total_reqs += req_names.len();
        for req in &req_names {
            if verified_names.contains(req) {
                verified_reqs += 1;
            }
        }
    }

    let pct = if total_reqs > 0 {
        (verified_reqs as f64 / total_reqs as f64) * 100.0
    } else {
        100.0
    };

    if cli.format == "json" {
        println!(
            "{{\"total_requirements\":{},\"verified_requirements\":{},\"coverage_pct\":{:.1}}}",
            total_reqs, verified_reqs, pct
        );
    } else {
        println!("Verification Coverage");
        println!("  Requirements: {}", total_reqs);
        println!("  With verify:  {} ({:.1}%)", verified_reqs, pct);
        if total_reqs > verified_reqs {
            println!("  Missing:      {}", total_reqs - verified_reqs);
        }
    }

    ExitCode::SUCCESS
}

fn run_list(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let mut cases = Vec::new();
    for model in &models {
        cases.extend(sysml_verify::extract_verification_cases(model));
    }

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&cases).unwrap_or_default());
    } else {
        if cases.is_empty() {
            println!("No verification cases found.");
        } else {
            println!("Verification Cases ({}):", cases.len());
            for vc in &cases {
                println!("  {} ({} steps, verifies: {})",
                    vc.name,
                    vc.steps.len(),
                    if vc.requirements.is_empty() {
                        "none".to_string()
                    } else {
                        vc.requirements.join(", ")
                    }
                );
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_status(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // Build a map of requirement -> verification status
    let mut req_status: Vec<(&str, bool, bool)> = Vec::new();

    for model in &models {
        let verified_names: std::collections::HashSet<&str> = model
            .verifications
            .iter()
            .map(|v| v.requirement.as_str())
            .collect();

        let satisfied_names: std::collections::HashSet<&str> = model
            .satisfactions
            .iter()
            .map(|s| s.requirement.as_str())
            .collect();

        for def in &model.definitions {
            if def.kind == sysml_core::model::DefKind::Requirement {
                req_status.push((
                    &def.name,
                    satisfied_names.contains(def.name.as_str()),
                    verified_names.contains(def.name.as_str()),
                ));
            }
        }
    }

    if cli.format == "json" {
        let items: Vec<serde_json::Value> = req_status.iter().map(|(name, sat, ver)| {
            serde_json::json!({
                "requirement": name,
                "satisfied": sat,
                "verified": ver,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap_or_default());
    } else {
        if req_status.is_empty() {
            println!("No requirements found.");
        } else {
            println!("{:<30} {:>10} {:>10}", "Requirement", "Satisfied", "Verified");
            println!("{}", "-".repeat(52));
            for (name, sat, ver) in &req_status {
                let sat_mark = if *sat { "yes" } else { "NO" };
                let ver_mark = if *ver { "yes" } else { "NO" };
                println!("{:<30} {:>10} {:>10}", name, sat_mark, ver_mark);
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_add(file: Option<&PathBuf>, inside: Option<&str>) -> ExitCode {
    use sysml_core::interactive::{run_wizard, WizardRunner};
    use crate::wizard::CliWizardRunner;

    let runner = CliWizardRunner::new();
    if !runner.is_interactive() {
        eprintln!("error: `verify add` requires an interactive terminal");
        return ExitCode::FAILURE;
    }

    let steps = sysml_verify::build_verify_add_wizard(None);
    let result = match run_wizard(&runner, &steps) {
        Some(r) => r,
        None => {
            eprintln!("Cancelled.");
            return ExitCode::FAILURE;
        }
    };

    let (name, sysml_text) = match sysml_verify::interpret_verify_add_wizard(&result) {
        Some(pair) => pair,
        None => {
            eprintln!("error: incomplete wizard answers");
            return ExitCode::FAILURE;
        }
    };

    eprintln!("\nPreview:");
    for line in sysml_text.lines() {
        eprintln!("  {}", line);
    }
    eprintln!();

    if let Some(target) = file {
        match crate::model_writer::write_to_model(target, &sysml_text, inside) {
            Ok(()) => {
                eprintln!("Wrote {} to {}", name, target.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {}", e);
                ExitCode::FAILURE
            }
        }
    } else {
        println!("{}", sysml_text);
        ExitCode::SUCCESS
    }
}

fn parse_files(files: &[PathBuf]) -> Option<Vec<sysml_core::model::Model>> {
    let mut models = Vec::new();
    for f in files {
        let (path, source) = match crate::read_source(f) {
            Ok(ps) => ps,
            Err(_) => return None,
        };
        models.push(sysml_core::parser::parse_file(&path, &source));
    }
    Some(models)
}

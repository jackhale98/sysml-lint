/// Risk management CLI commands.
///
/// Implements hazard analysis (MIL-STD-882E / ISO 14971) and FMEA
/// (AIAG/VDA, SAE J1739) workflows.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::RiskCommand;

pub fn run(cli: &crate::Cli, kind: &RiskCommand) -> ExitCode {
    match kind {
        RiskCommand::List { files } => run_list(cli, files),
        RiskCommand::Matrix { files } => run_matrix(cli, files),
        RiskCommand::Fmea { files } => run_fmea(cli, files),
        RiskCommand::Coverage { files } => run_coverage(cli, files),
        RiskCommand::Add { file, inside } => run_add(file.as_ref(), inside.as_deref()),
    }
}

fn run_list(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let mut risks = Vec::new();
    for model in &models {
        risks.extend(sysml_risk::extract_risks(model));
    }

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&risks).unwrap_or_default());
    } else if risks.is_empty() {
        println!("No risks found in model.");
    } else {
        println!("Risks ({}):", risks.len());
        for r in &risks {
            let rpn_str = r.rpn.map_or("n/a".to_string(), |v| v.to_string());
            let sev = r.severity.map_or("-".to_string(), |s| s.to_string());
            let occ = r.occurrence.map_or("-".to_string(), |o| o.to_string());
            let acc = r.acceptance
                .map(|a| a.label().to_string())
                .unwrap_or_default();
            let label = if r.failure_mode.is_empty() { &r.id } else { &r.failure_mode };
            let assigned = r.assigned_to.as_deref().unwrap_or("");
            if assigned.is_empty() {
                println!("  {} [S:{} O:{} RPN:{} {}]", label, sev, occ, rpn_str, acc);
            } else {
                println!("  {} [S:{} O:{} RPN:{} {}] → {}", label, sev, occ, rpn_str, acc, assigned);
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_matrix(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let mut risks = Vec::new();
    for model in &models {
        risks.extend(sysml_risk::extract_risks(model));
    }

    let matrix = sysml_risk::generate_risk_matrix(&risks);

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&matrix).unwrap_or_default());
    } else {
        println!("{}", matrix.to_text());
    }

    ExitCode::SUCCESS
}

fn run_fmea(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let mut risks = Vec::new();
    for model in &models {
        risks.extend(sysml_risk::extract_risks(model));
    }

    let rows = sysml_risk::generate_fmea_table(&risks);

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&rows).unwrap_or_default());
    } else if rows.is_empty() {
        println!("No risks for FMEA worksheet.");
    } else {
        // Print FMEA header
        println!("{:<20} {:<20} {:<15} {:<15} {:>3} {:>3} {:>3} {:>5} {:<13} {:<20} {:<10} {}",
            "Item", "Failure Mode", "Effect", "Cause",
            "S", "O", "D", "RPN", "Risk Level", "Rec. Action", "Status", "Assigned To");
        println!("{}", "-".repeat(145));
        for row in &rows {
            let assigned = row.assigned_to.as_deref().unwrap_or("-");
            println!("{:<20} {:<20} {:<15} {:<15} {:>3} {:>3} {:>3} {:>5} {:<13} {:<20} {:<10} {}",
                truncate(&row.item, 19),
                truncate(&row.failure_mode, 19),
                truncate(&row.failure_effect, 14),
                truncate(&row.failure_cause, 14),
                row.severity,
                row.occurrence,
                row.detection,
                row.rpn,
                truncate(&row.acceptance, 12),
                truncate(&row.recommended_action, 19),
                row.status,
                assigned,
            );
        }
    }

    ExitCode::SUCCESS
}

fn run_coverage(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    use sysml_core::model::DefKind;

    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // Collect all risk-assignable elements (parts, actions, use cases)
    // and which ones have risks assigned to them.
    let mut risks = Vec::new();
    let mut risk_parents: std::collections::HashSet<String> = std::collections::HashSet::new();

    struct Element {
        name: String,
        kind: String,
    }
    let mut elements: Vec<Element> = Vec::new();

    for model in &models {
        let extracted = sysml_risk::extract_risks(model);
        for r in &extracted {
            if let Some(parent) = &r.assigned_to {
                risk_parents.insert(parent.clone());
            }
        }
        risks.extend(extracted);

        for def in &model.definitions {
            match def.kind {
                DefKind::Part | DefKind::Action | DefKind::UseCase => {
                    // Skip the RiskDef definitions themselves
                    let is_risk = def.name.contains("Risk")
                        || def.name.contains("risk")
                        || def.name.contains("Hazard")
                        || def.name.contains("hazard")
                        || def.super_type.as_deref()
                            .map(|s| s == "RiskDef" || s.ends_with("::RiskDef"))
                            .unwrap_or(false);
                    if !is_risk {
                        elements.push(Element {
                            name: def.name.clone(),
                            kind: format!("{:?}", def.kind).to_lowercase(),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    let total = elements.len();
    let covered: Vec<&Element> = elements.iter()
        .filter(|e| risk_parents.contains(&e.name))
        .collect();
    let uncovered: Vec<&Element> = elements.iter()
        .filter(|e| !risk_parents.contains(&e.name))
        .collect();

    let pct = if total > 0 {
        (covered.len() as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    if cli.format == "json" {
        let output = serde_json::json!({
            "total_elements": total,
            "covered": covered.len(),
            "uncovered": uncovered.len(),
            "coverage_pct": (pct * 10.0).round() / 10.0,
            "uncovered_elements": uncovered.iter().map(|e| {
                serde_json::json!({ "name": e.name, "kind": e.kind })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("Risk Coverage");
        println!("  Elements (parts/actions/use cases): {}", total);
        println!("  With risks:    {} ({:.1}%)", covered.len(), pct);
        println!("  Without risks: {}", uncovered.len());

        if !uncovered.is_empty() {
            println!();
            println!("Uncovered elements:");
            for e in &uncovered {
                println!("  {} ({})", e.name, e.kind);
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
        eprintln!("error: `risk add` requires an interactive terminal");
        return ExitCode::FAILURE;
    }

    let steps = sysml_risk::build_risk_add_wizard(None);
    let result = match run_wizard(&runner, &steps) {
        Some(r) => r,
        None => {
            eprintln!("Cancelled.");
            return ExitCode::FAILURE;
        }
    };

    let (name, sysml_text) = match sysml_risk::interpret_risk_add_wizard(&result) {
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

    let (target, parent) = if let Some(f) = file {
        (f.clone(), inside.map(|s| s.to_string()))
    } else {
        let cwd = std::env::current_dir().unwrap_or_default();
        match crate::model_writer::select_target_file(&cwd) {
            Some(f) => {
                let parent = crate::model_writer::select_parent_def(&f);
                (f, parent)
            }
            None => {
                println!("{}", sysml_text);
                return ExitCode::SUCCESS;
            }
        }
    };

    match crate::model_writer::write_to_model(&target, &sysml_text, parent.as_deref()) {
        Ok(()) => {
            eprintln!("Wrote {} to {}", name, target.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
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

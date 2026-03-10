/// Risk management CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::RiskCommand;

pub fn run(cli: &crate::Cli, kind: &RiskCommand) -> ExitCode {
    match kind {
        RiskCommand::List { files } => run_list(cli, files),
        RiskCommand::Matrix { files } => run_matrix(cli, files),
        RiskCommand::Fmea { files } => run_fmea(cli, files),
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
            let sev = r.severity.as_ref().map_or("-", |s| s.label());
            let lik = r.likelihood.as_ref().map_or("-", |l| l.label());
            println!("  {} [S:{} L:{} RPN:{}]", r.title, sev, lik, rpn_str);
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
        println!("{:<25} {:>5} {:>5} {:>5} {:>5} {:<20} {}",
            "Failure Mode", "S", "L", "D", "RPN", "Mitigation", "Status");
        println!("{}", "-".repeat(85));
        for row in &rows {
            println!("{:<25} {:>5} {:>5} {:>5} {:>5} {:<20} {}",
                truncate(&row.failure_mode, 24),
                row.severity,
                row.likelihood,
                row.detectability,
                row.rpn,
                truncate(&row.mitigation, 19),
                row.status,
            );
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

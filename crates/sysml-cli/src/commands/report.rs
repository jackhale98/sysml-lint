/// Cross-domain reporting CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::ReportCommand;

pub fn run(cli: &crate::Cli, kind: &ReportCommand) -> ExitCode {
    match kind {
        ReportCommand::Dashboard { files } => run_dashboard(cli, files),
        ReportCommand::Traceability { files, requirement } => {
            run_traceability(cli, files, requirement)
        }
        ReportCommand::Gate {
            files,
            gate_name,
            min_coverage,
        } => run_gate(cli, files, gate_name, *min_coverage),
    }
}

fn run_dashboard(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // For now, records are empty since we don't have a record store path.
    // The dashboard still works from model data alone.
    let records: Vec<sysml_core::record::RecordEnvelope> = Vec::new();
    let dashboard = sysml_report::generate_dashboard(&models, &records);

    if cli.format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&dashboard).unwrap_or_default()
        );
    } else {
        print!("{}", sysml_report::format_dashboard_text(&dashboard));
    }

    ExitCode::SUCCESS
}

fn run_traceability(
    cli: &crate::Cli,
    files: &[PathBuf],
    requirement: &str,
) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let thread = sysml_report::trace_requirement(&models, requirement);

    if cli.format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&thread).unwrap_or_default()
        );
    } else {
        print!("{}", sysml_report::format_traceability_text(&thread));
    }

    ExitCode::SUCCESS
}

fn run_gate(
    cli: &crate::Cli,
    files: &[PathBuf],
    gate_name: &str,
    min_coverage: f64,
) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let records: Vec<sysml_core::record::RecordEnvelope> = Vec::new();
    let result = sysml_report::check_gate(
        &models,
        &records,
        gate_name,
        min_coverage,
        0, // max_critical_risks
        0, // max_open_ncrs
    );

    if cli.format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    } else {
        println!("Gate Check: {}", result.gate_name);
        println!("  Coverage:    {:.1}% (required: {:.1}%)", result.coverage_pct, min_coverage);
        println!("  Open risks:  {}", result.open_risks);
        println!("  Open NCRs:   {}", result.open_ncrs);
        println!(
            "  Result:      {}",
            if result.passed { "PASS" } else { "FAIL" }
        );

        if !result.blocking_items.is_empty() {
            println!("  Blocking items:");
            for item in &result.blocking_items {
                println!("    - {item}");
            }
        }
    }

    if result.passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
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

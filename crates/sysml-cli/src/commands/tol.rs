/// Tolerance analysis CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::TolCommand;

pub fn run(cli: &crate::Cli, kind: &TolCommand) -> ExitCode {
    match kind {
        TolCommand::Analyze {
            files,
            method,
            iterations,
        } => run_analyze(cli, files, method, *iterations),
        TolCommand::Sensitivity { files } => run_sensitivity(cli, files),
        TolCommand::Add { file, inside } => run_add(file.as_ref(), inside.as_deref()),
    }
}

fn run_analyze(
    cli: &crate::Cli,
    files: &[PathBuf],
    method_str: &str,
    iterations: usize,
) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let method = match method_str {
        "worst-case" | "wc" => sysml_tol::AnalysisMethod::WorstCase,
        "rss" => sysml_tol::AnalysisMethod::Rss,
        "monte-carlo" | "mc" => sysml_tol::AnalysisMethod::MonteCarlo,
        other => {
            eprintln!("error: unknown analysis method `{}`. Use: worst-case, rss, monte-carlo", other);
            return ExitCode::FAILURE;
        }
    };

    let mut chains = Vec::new();
    for model in &models {
        chains.extend(sysml_tol::extract_dimension_chains(model));
    }

    if chains.is_empty() {
        // If no explicit chains, try to build one from all tolerances
        let mut all_tolerances = Vec::new();
        for model in &models {
            all_tolerances.extend(sysml_tol::extract_tolerances(model));
        }

        if all_tolerances.is_empty() {
            println!("No dimension chains or tolerances found in model.");
            return ExitCode::SUCCESS;
        }

        let chain = sysml_tol::DimensionChain {
            name: "auto".to_string(),
            tolerances: all_tolerances,
            closing_dimension: "closing".to_string(),
        };
        chains.push(chain);
    }

    for chain in &chains {
        let result = sysml_tol::analyze(chain, &method, iterations);

        if cli.format == "json" {
            println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
        } else {
            println!("Dimension Chain: {}", chain.name);
            println!("  Method:  {:?}", result.method);
            println!("  Nominal: {:.4}", result.nominal_result);
            println!("  Min:     {:.4}", result.min_result);
            println!("  Max:     {:.4}", result.max_result);
            if let Some(sigma) = result.sigma {
                println!("  Sigma:   {:.4}", sigma);
            }
            if let Some(cpk) = result.cpk {
                println!("  Cpk:     {:.2}", cpk);
            }
            if !result.contributors.is_empty() {
                println!("  Contributors:");
                for c in &result.contributors {
                    println!("    {:<20} {:>6.1}%  (sensitivity: {:.3})", c.name, c.contribution_pct, c.sensitivity);
                }
            }
            println!();
        }
    }

    ExitCode::SUCCESS
}

fn run_sensitivity(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let mut chains = Vec::new();
    for model in &models {
        chains.extend(sysml_tol::extract_dimension_chains(model));
    }

    if chains.is_empty() {
        let mut all_tolerances = Vec::new();
        for model in &models {
            all_tolerances.extend(sysml_tol::extract_tolerances(model));
        }

        if all_tolerances.is_empty() {
            println!("No dimension chains or tolerances found in model.");
            return ExitCode::SUCCESS;
        }

        let chain = sysml_tol::DimensionChain {
            name: "auto".to_string(),
            tolerances: all_tolerances,
            closing_dimension: "closing".to_string(),
        };
        chains.push(chain);
    }

    for chain in &chains {
        let contributors = sysml_tol::sensitivity_analysis(chain);

        if cli.format == "json" {
            println!("{}", serde_json::to_string_pretty(&contributors).unwrap_or_default());
        } else {
            println!("Sensitivity Analysis: {}", chain.name);
            println!("  {:<25} {:>12} {:>12}", "Contributor", "Contribution", "Sensitivity");
            println!("  {}", "-".repeat(52));
            for c in &contributors {
                println!("  {:<25} {:>10.1}% {:>12.3}", c.name, c.contribution_pct, c.sensitivity);
            }
            println!();
        }
    }

    ExitCode::SUCCESS
}

fn run_add(file: Option<&PathBuf>, inside: Option<&str>) -> ExitCode {
    use sysml_core::interactive::{run_wizard, WizardRunner};
    use crate::wizard::CliWizardRunner;

    let runner = CliWizardRunner::new();
    if !runner.is_interactive() {
        eprintln!("error: `tol add` requires an interactive terminal");
        return ExitCode::FAILURE;
    }

    let steps = sysml_tol::build_tol_add_wizard(None);
    let result = match run_wizard(&runner, &steps) {
        Some(r) => r,
        None => {
            eprintln!("Cancelled.");
            return ExitCode::FAILURE;
        }
    };

    let (name, sysml_text) = match sysml_tol::interpret_tol_add_wizard(&result) {
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

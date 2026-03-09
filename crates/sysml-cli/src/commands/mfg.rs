/// Manufacturing execution CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::MfgCommand;

pub fn run(cli: &crate::Cli, kind: &MfgCommand) -> ExitCode {
    match kind {
        MfgCommand::List { files } => run_list(cli, files),
        MfgCommand::Spc {
            parameter,
            values,
        } => run_spc(cli, parameter, values),
    }
}

fn run_list(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // Collect action definitions that could be manufacturing routings.
    let mut routings: Vec<(String, usize)> = Vec::new();
    for model in &models {
        for def in &model.definitions {
            if def.kind == sysml_core::model::DefKind::Action && def.has_body {
                // Try to extract a routing to count steps.
                let step_count = sysml_mfg::extract_routing(model, &def.name)
                    .map(|steps| steps.len())
                    .unwrap_or(0);
                routings.push((def.name.clone(), step_count));
            }
        }
    }

    if cli.format == "json" {
        let entries: Vec<serde_json::Value> = routings
            .iter()
            .map(|(name, steps)| {
                serde_json::json!({
                    "name": name,
                    "steps": steps,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries).unwrap_or_default());
    } else if routings.is_empty() {
        println!("No manufacturing routings (action definitions) found.");
    } else {
        println!("Manufacturing Routings ({}):", routings.len());
        for (name, steps) in &routings {
            let step_str = if *steps > 0 {
                format!("{steps} steps")
            } else {
                "no steps extracted".to_string()
            };
            println!("  {name} ({step_str})");
        }
    }

    ExitCode::SUCCESS
}

fn run_spc(cli: &crate::Cli, parameter: &str, values: &[f64]) -> ExitCode {
    if values.is_empty() {
        eprintln!("error: no values provided. Use --values to supply comma-separated readings.");
        return ExitCode::FAILURE;
    }

    // Compute control limits from the data (mean +/- 3*sigma).
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let sigma = if n > 1.0 {
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        variance.sqrt()
    } else {
        0.0
    };
    let ucl = mean + 3.0 * sigma;
    let lcl = mean - 3.0 * sigma;
    // Use control limits as spec limits when none are explicitly provided.
    let usl = ucl;
    let lsl = lcl;

    let mut spc = sysml_mfg::compute_spc(values, ucl, lcl, usl, lsl);
    spc.parameter_name = parameter.to_string();

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&spc).unwrap_or_default());
    } else {
        print!("{}", sysml_mfg::format_spc_text(&spc));
    }

    ExitCode::SUCCESS
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

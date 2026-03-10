/// Manufacturing execution CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::MfgCommand;

pub fn run(cli: &crate::Cli, kind: &MfgCommand) -> ExitCode {
    match kind {
        MfgCommand::List { files } => run_list(cli, files),
        MfgCommand::Spc { parameter, values } => run_spc(cli, parameter, values),
        MfgCommand::StartLot { files, routing, quantity, lot_type, author } => {
            run_start_lot(files, routing.as_deref(), *quantity, lot_type, author)
        }
        MfgCommand::Step { lot_id, author } => run_step(lot_id, author),
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

fn run_start_lot(
    files: &[PathBuf],
    routing: Option<&str>,
    quantity: u32,
    lot_type_str: &str,
    author: &str,
) -> ExitCode {
    use sysml_core::interactive::WizardRunner;
    use crate::wizard::CliWizardRunner;

    let runner = CliWizardRunner::new();
    if !runner.is_interactive() {
        eprintln!("error: `mfg start-lot` requires an interactive terminal");
        return ExitCode::FAILURE;
    }

    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // Find action definitions (potential routings)
    let mut routings: Vec<(String, Vec<sysml_mfg::ProcessStep>)> = Vec::new();
    for model in &models {
        for def in &model.definitions {
            if def.kind == sysml_core::model::DefKind::Action && def.has_body {
                if let Some(steps) = sysml_mfg::extract_routing(model, &def.name) {
                    if !steps.is_empty() {
                        routings.push((def.name.clone(), steps));
                    }
                }
            }
        }
    }

    if routings.is_empty() {
        eprintln!("No manufacturing routings found in the provided files.");
        return ExitCode::FAILURE;
    }

    // Select routing
    let (routing_name, steps) = if let Some(name) = routing {
        match routings.into_iter().find(|(n, _)| n == name) {
            Some(r) => r,
            None => {
                eprintln!("error: routing '{}' not found", name);
                return ExitCode::FAILURE;
            }
        }
    } else {
        let names: Vec<&str> = routings.iter().map(|(n, _)| n.as_str()).collect();
        let choice = crate::select_item("Select routing:", &names);
        match choice {
            Some(idx) => routings.into_iter().nth(idx).unwrap(),
            None => {
                eprintln!("Cancelled.");
                return ExitCode::FAILURE;
            }
        }
    };

    let lt = match lot_type_str {
        "prototype" => sysml_mfg::LotType::Prototype,
        "first-article" => sysml_mfg::LotType::FirstArticle,
        _ => sysml_mfg::LotType::Production,
    };

    let lot = sysml_mfg::create_lot(&routing_name, steps, quantity, lt);
    eprintln!("{}", sysml_mfg::lot_summary(&lot));

    // Write lot record
    let record = sysml_mfg::create_lot_record(&lot, author);
    let records_dir = crate::records::resolve_records_dir();
    match crate::records::write_record(&record, &records_dir) {
        Ok(path) => {
            eprintln!("Lot record written: {}", path.display());
            println!("{}", lot.id);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error writing record: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_step(lot_id: &str, author: &str) -> ExitCode {
    use sysml_core::interactive::{run_wizard, WizardRunner};
    use crate::wizard::CliWizardRunner;

    let runner = CliWizardRunner::new();

    // Find the lot record
    let records_dir = crate::records::resolve_records_dir();
    let record = match crate::records::find_record(&records_dir, lot_id) {
        Some(r) => r,
        None => {
            eprintln!("error: lot '{}' not found in {}", lot_id, records_dir.display());
            return ExitCode::FAILURE;
        }
    };

    // Reconstruct the lot from the record
    let mut lot = match sysml_mfg::reconstruct_lot(&record) {
        Some(l) => l,
        None => {
            eprintln!("error: could not reconstruct lot from record");
            return ExitCode::FAILURE;
        }
    };

    if lot.current_step >= lot.steps.len() {
        eprintln!("Lot {} is already completed.", lot.id);
        return ExitCode::FAILURE;
    }

    let step = &lot.steps[lot.current_step];
    eprintln!("Lot: {} — Step {}/{}: {} [{}]",
        lot.id, lot.current_step + 1, lot.steps.len(),
        step.name, step.process_type);
    eprintln!();

    if !runner.is_interactive() {
        eprintln!("error: `mfg step` requires an interactive terminal");
        return ExitCode::FAILURE;
    }

    // Build and run wizard for this step
    let wizard_steps = sysml_mfg::build_step_wizard(step);
    let result = match run_wizard(&runner, &wizard_steps) {
        Some(r) => r,
        None => {
            eprintln!("Cancelled.");
            return ExitCode::FAILURE;
        }
    };

    // Interpret readings
    let readings = sysml_mfg::interpret_step_result(&result, step);

    // Display readings
    if !readings.is_empty() {
        eprintln!("\nReadings:");
        for r in &readings {
            let ctrl = if r.within_control { "OK" } else { "OUT OF CONTROL" };
            let spec = if r.within_spec { "OK" } else { "OUT OF SPEC" };
            eprintln!("  {} = {:.4} [ctrl: {}, spec: {}]",
                r.parameter_name, r.value, ctrl, spec);
        }
    }

    // Advance the lot
    match sysml_mfg::advance_step(&mut lot, readings) {
        Ok(status) => {
            eprintln!("\nStep status: {}", status);
            if lot.status == sysml_mfg::LotStatus::Completed {
                eprintln!("Lot {} is now COMPLETED.", lot.id);
            }
        }
        Err(sysml_mfg::MfgError::ParameterOutOfSpec(msg)) => {
            eprintln!("\nFAILED: Parameter out of spec — {}", msg);
            eprintln!("Lot {} is on hold.", lot.id);
        }
        Err(e) => {
            eprintln!("\nerror: {:?}", e);
            return ExitCode::FAILURE;
        }
    }

    // Write updated lot record
    let updated_record = sysml_mfg::create_lot_record(&lot, author);
    match crate::records::write_record(&updated_record, &records_dir) {
        Ok(path) => {
            eprintln!("Record updated: {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error writing record: {}", e);
            ExitCode::FAILURE
        }
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

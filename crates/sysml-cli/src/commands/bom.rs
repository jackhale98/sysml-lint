/// Bill of materials CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::BomCommand;

pub fn run(cli: &crate::Cli, kind: &BomCommand) -> ExitCode {
    match kind {
        BomCommand::Rollup {
            files,
            root,
            include_mass,
            include_cost,
        } => run_rollup(cli, files, root, *include_mass, *include_cost),
        BomCommand::WhereUsed { files, part } => run_where_used(cli, files, part),
        BomCommand::Export { files, root, format } => run_export(cli, files, root, format),
        BomCommand::Add { file, inside } => run_add(file.as_ref(), inside.as_deref()),
    }
}

fn run_rollup(
    cli: &crate::Cli,
    files: &[PathBuf],
    root: &str,
    include_mass: bool,
    include_cost: bool,
) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // Merge all models to search across files.
    let merged = merge_models(&models);

    let tree = match sysml_bom::build_bom_tree(&merged, root) {
        Some(t) => t,
        None => {
            eprintln!("error: no part definition `{root}` found in model");
            return ExitCode::FAILURE;
        }
    };

    if cli.format == "json" {
        let summary = sysml_bom::bom_summary(&tree);
        let output = serde_json::json!({
            "tree": serde_json::to_value(&tree).unwrap_or_default(),
            "summary": serde_json::to_value(&summary).unwrap_or_default(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        let text = sysml_bom::format_bom_tree(&tree, include_mass, include_cost);
        print!("{text}");

        let summary = sysml_bom::bom_summary(&tree);
        if !cli.quiet {
            eprintln!(
                "BOM: {} total parts, {} unique, depth {}",
                summary.total_parts, summary.unique_parts, summary.max_depth,
            );
            if let Some(mass) = summary.total_mass_kg {
                eprintln!("  Total mass: {mass:.3} kg");
            }
            if let Some(cost) = summary.total_cost {
                eprintln!("  Total cost: {cost:.2}");
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_where_used(cli: &crate::Cli, files: &[PathBuf], part: &str) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let merged = merge_models(&models);
    let parents = sysml_bom::where_used(&merged, part);

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&parents).unwrap_or_default());
    } else if parents.is_empty() {
        println!("Part `{part}` is not used in any definition.");
    } else {
        println!("Part `{part}` is used in:");
        for p in &parents {
            println!("  {p}");
        }
    }

    ExitCode::SUCCESS
}

fn run_export(
    cli: &crate::Cli,
    files: &[PathBuf],
    root: &str,
    _format: &str,
) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let merged = merge_models(&models);

    let tree = match sysml_bom::build_bom_tree(&merged, root) {
        Some(t) => t,
        None => {
            eprintln!("error: no part definition `{root}` found in model");
            return ExitCode::FAILURE;
        }
    };

    if cli.format == "json" {
        let rows = sysml_bom::flatten_bom(&tree);
        println!("{}", serde_json::to_string_pretty(&rows).unwrap_or_default());
    } else {
        print!("{}", sysml_bom::format_bom_csv(&tree));
    }

    ExitCode::SUCCESS
}

fn run_add(file: Option<&PathBuf>, inside: Option<&str>) -> ExitCode {
    use sysml_core::interactive::{run_wizard, WizardRunner};
    use crate::wizard::CliWizardRunner;

    let runner = CliWizardRunner::new();
    if !runner.is_interactive() {
        eprintln!("error: `bom add` requires an interactive terminal");
        return ExitCode::FAILURE;
    }

    let steps = sysml_bom::build_bom_add_wizard(None);
    let result = match run_wizard(&runner, &steps) {
        Some(r) => r,
        None => {
            eprintln!("Cancelled.");
            return ExitCode::FAILURE;
        }
    };

    let (name, sysml_text) = match sysml_bom::interpret_bom_add_wizard(&result) {
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

/// Merge multiple models into a single model for cross-file BOM lookups.
fn merge_models(models: &[sysml_core::model::Model]) -> sysml_core::model::Model {
    let mut merged = sysml_core::model::Model::new("merged".to_string());
    for m in models {
        merged.definitions.extend(m.definitions.iter().cloned());
        merged.usages.extend(m.usages.iter().cloned());
        merged.connections.extend(m.connections.iter().cloned());
        merged.satisfactions.extend(m.satisfactions.iter().cloned());
        merged.verifications.extend(m.verifications.iter().cloned());
    }
    merged
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

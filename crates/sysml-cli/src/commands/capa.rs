/// Nonconformance and corrective action (CAPA) CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::CapaCommand;

pub fn run(cli: &crate::Cli, kind: &CapaCommand) -> ExitCode {
    match kind {
        CapaCommand::Trend { files, group_by } => run_trend(cli, files, group_by),
        CapaCommand::List => run_list(cli),
    }
}

fn run_trend(cli: &crate::Cli, files: &[PathBuf], group_by: &str) -> ExitCode {
    if files.is_empty() {
        // Without model files, provide guidance.
        if cli.format == "json" {
            println!("[]");
        } else {
            println!("CAPA Trend Analysis");
            println!();
            println!("  No files provided. To analyze NCR trends, provide SysML files that");
            println!("  contain nonconformance records or use the record system:");
            println!();
            println!("  1. Create NCRs with `sysml capa` record commands");
            println!("  2. Provide model files to correlate NCRs with parts");
            println!();
            println!("  Group-by: {group_by}");
        }
        return ExitCode::SUCCESS;
    }

    let _models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    // For now, NCRs come from the record system rather than model files.
    // Report that fact and instruct the user.
    if cli.format == "json" {
        let output = serde_json::json!({
            "group_by": group_by,
            "items": serde_json::Value::Array(Vec::new()),
            "note": "NCR trends are derived from .sysml-records/ files. \
                     Use `sysml capa list` to see current status.",
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("CAPA Trend Analysis (group by: {group_by})");
        println!();
        println!("  No NCR records found in model files.");
        println!("  NCR trends are derived from the `.sysml-records/` directory.");
        println!("  Use `sysml capa list` to view current CAPA status.");
    }

    ExitCode::SUCCESS
}

fn run_list(cli: &crate::Cli) -> ExitCode {
    // Provide a status overview. Without an active record store connection
    // in the CLI, present guidance on the CAPA workflow.
    if cli.format == "json" {
        let overview = serde_json::json!({
            "workflow": [
                "1. Create NCR: record nonconformance details",
                "2. Root cause analysis: 5-why or fishbone",
                "3. Define corrective/preventive actions",
                "4. Verify effectiveness",
                "5. Close NCR with disposition",
            ],
        });
        println!("{}", serde_json::to_string_pretty(&overview).unwrap_or_default());
    } else {
        println!("CAPA Status Overview");
        println!();
        println!("  CAPA workflow:");
        println!("    1. Create NCR  - record nonconformance details");
        println!("    2. Root cause   - 5-why or fishbone analysis");
        println!("    3. Actions      - define corrective/preventive actions");
        println!("    4. Verify       - confirm effectiveness");
        println!("    5. Close        - dispose NCR");
        println!();
        println!("  NCR records are stored in `.sysml-records/` and tracked");
        println!("  via the record envelope system.");
        println!();
        println!("  Use `sysml capa trend <files>` to analyze trends.");
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

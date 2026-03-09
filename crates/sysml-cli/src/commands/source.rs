/// Supplier management CLI commands.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::SourceCommand;

pub fn run(cli: &crate::Cli, kind: &SourceCommand) -> ExitCode {
    match kind {
        SourceCommand::List { files } => run_list(cli, files),
        SourceCommand::Asl { files } => run_asl(cli, files),
        SourceCommand::Rfq {
            part,
            description,
            quantity,
        } => run_rfq(cli, part, description, *quantity),
    }
}

fn run_list(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let mut suppliers = Vec::new();
    for model in &models {
        suppliers.extend(sysml_source::extract_suppliers(model));
    }

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&suppliers).unwrap_or_default());
    } else if suppliers.is_empty() {
        println!("No suppliers found in model.");
        println!("  Tip: define suppliers as `part def MySupplier :> SupplierDef {{ ... }}`");
    } else {
        println!("Suppliers ({}):", suppliers.len());
        for s in &suppliers {
            let code = if s.code.is_empty() {
                String::new()
            } else {
                format!(" ({})", s.code)
            };
            let certs = if s.certifications.is_empty() {
                String::new()
            } else {
                format!("  certs: {}", s.certifications.join(", "))
            };
            println!(
                "  {}{} [{}]{}",
                s.name,
                code,
                s.qualification_status.label(),
                certs,
            );
        }
    }

    ExitCode::SUCCESS
}

fn run_asl(cli: &crate::Cli, files: &[PathBuf]) -> ExitCode {
    let models = match parse_files(files) {
        Some(m) => m,
        None => return ExitCode::FAILURE,
    };

    let mut all_suppliers = Vec::new();
    let mut all_sources = Vec::new();
    for model in &models {
        all_suppliers.extend(sysml_source::extract_suppliers(model));
        all_sources.extend(sysml_source::extract_sources(model));
    }

    let approved = sysml_source::approved_source_list(&all_sources, &all_suppliers);

    if cli.format == "json" {
        println!("{}", serde_json::to_string_pretty(&approved).unwrap_or_default());
    } else if approved.is_empty() {
        println!("No approved sources found.");
        println!("  Only suppliers with status `approved` or `preferred` appear here.");
    } else {
        println!("Approved Source List ({} entries):", approved.len());
        println!(
            "  {:<20} {:<20} {:<15} {:>10} {:>8} {:>6}",
            "Part", "Supplier", "Supplier P/N", "Lead Time", "MOQ", "Price",
        );
        println!("  {}", "-".repeat(82));
        for src in &approved {
            println!(
                "  {:<20} {:<20} {:<15} {:>8}d {:>8} {:>6.2}",
                truncate(&src.part_name, 19),
                truncate(&src.supplier_name, 19),
                truncate(&src.supplier_part_number, 14),
                src.lead_time_days,
                src.moq,
                src.unit_price,
            );
        }
    }

    ExitCode::SUCCESS
}

fn run_rfq(cli: &crate::Cli, part: &str, description: &str, quantity: u32) -> ExitCode {
    let text = sysml_source::generate_rfq_text(part, description, quantity, "");

    if cli.format == "json" {
        let output = serde_json::json!({
            "part": part,
            "description": description,
            "quantity": quantity,
            "text": &text,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        print!("{text}");
    }

    ExitCode::SUCCESS
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

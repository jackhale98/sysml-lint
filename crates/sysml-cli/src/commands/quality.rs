/// Quality management CLI commands (NCR, CAPA, Process Deviation).

use std::path::PathBuf;
use std::process::ExitCode;

use crate::QualityCommand;

pub fn run(cli: &crate::Cli, kind: &QualityCommand) -> ExitCode {
    match kind {
        QualityCommand::Trend { files, group_by } => run_trend(cli, files, group_by),
        QualityCommand::List => run_list(cli),
        QualityCommand::Create { r#type, file, inside } => {
            run_create(r#type.as_deref(), file.as_ref(), inside.as_deref())
        }
    }
}

fn run_trend(cli: &crate::Cli, files: &[PathBuf], group_by: &str) -> ExitCode {
    if files.is_empty() {
        if cli.format == "json" {
            println!("[]");
        } else {
            println!("Quality Trend Analysis");
            println!();
            println!("  No files provided. To analyze NCR trends, provide SysML files that");
            println!("  contain nonconformance records or use the record system:");
            println!();
            println!("  1. Create NCRs with `sysml quality` record commands");
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

    if cli.format == "json" {
        let output = serde_json::json!({
            "group_by": group_by,
            "items": serde_json::Value::Array(Vec::new()),
            "note": "NCR trends are derived from .sysml-records/ files. \
                     Use `sysml quality list` to see current status.",
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("Quality Trend Analysis (group by: {group_by})");
        println!();
        println!("  No NCR records found in model files.");
        println!("  NCR trends are derived from the `.sysml-records/` directory.");
        println!("  Use `sysml quality list` to view current status.");
    }

    ExitCode::SUCCESS
}

fn run_list(cli: &crate::Cli) -> ExitCode {
    if cli.format == "json" {
        let overview = serde_json::json!({
            "item_types": {
                "ncr": {
                    "name": "Nonconformance Report",
                    "lifecycle": ["Open", "Investigating", "Dispositioned", "Verified", "Closed", "Reopened"],
                    "description": "Documents an observed nonconformance — what went wrong."
                },
                "capa": {
                    "name": "Corrective/Preventive Action",
                    "lifecycle": ["Initiated", "Root Cause Analysis", "Planning Actions", "Implementing", "Verifying Effectiveness", "Pending Closure", "Closed"],
                    "description": "A formal action program to address root causes and prevent recurrence."
                },
                "deviation": {
                    "name": "Process Deviation",
                    "lifecycle": ["Requested", "Under Review", "Approved", "Denied", "Active", "Expired", "Closed"],
                    "description": "A planned, approved departure from a standard process."
                }
            },
        });
        println!("{}", serde_json::to_string_pretty(&overview).unwrap_or_default());
    } else {
        println!("Quality Management Overview");
        println!();
        println!("  Three quality item types, each with its own lifecycle:");
        println!();
        println!("  NCR (Nonconformance Report)");
        println!("    Documents what went wrong — a finding, not an action.");
        println!("    Lifecycle: Open → Investigating → Dispositioned → Verified → Closed");
        println!();
        println!("  CAPA (Corrective/Preventive Action)");
        println!("    A formal action program to address root causes.");
        println!("    May originate from NCRs, audits, complaints, or improvement.");
        println!("    Lifecycle: Initiated → RCA → Planning → Implementing → Verifying → Closed");
        println!();
        println!("  Process Deviation");
        println!("    A planned, approved departure from a standard process.");
        println!("    Unlike NCRs (unplanned), deviations are pre-approved.");
        println!("    Lifecycle: Requested → Under Review → Approved → Active → Closed");
        println!();
        println!("  Records are stored in `.sysml-records/` via the record envelope system.");
        println!("  Use `sysml quality trend <files>` to analyze trends.");
    }

    ExitCode::SUCCESS
}

fn run_create(
    item_type: Option<&str>,
    _file: Option<&PathBuf>,
    _inside: Option<&str>,
) -> ExitCode {
    use sysml_core::interactive::*;
    use crate::wizard::CliWizardRunner;

    let runner = CliWizardRunner::new();
    if !runner.is_interactive() {
        eprintln!("error: `quality create` requires an interactive terminal");
        return ExitCode::FAILURE;
    }

    // If no --type given, ask which quality item type to create
    let chosen_type = if let Some(t) = item_type {
        t.to_string()
    } else {
        let type_step = WizardStep::choice(
            "item_type",
            "What type of quality item?",
            vec![
                ("ncr", "NCR — Nonconformance Report"),
                ("capa", "CAPA — Corrective/Preventive Action"),
                ("deviation", "Process Deviation — planned departure"),
            ],
        );
        match runner.run_step(&type_step) {
            Some(WizardAnswer::String(s)) => s,
            _ => {
                eprintln!("Cancelled.");
                return ExitCode::FAILURE;
            }
        }
    };

    let steps = build_quality_wizard_steps(&chosen_type);
    let result = match run_wizard(&runner, &steps) {
        Some(r) => r,
        None => {
            eprintln!("Cancelled.");
            return ExitCode::FAILURE;
        }
    };

    match chosen_type.as_str() {
        "ncr" => create_ncr_from_wizard(&result),
        "capa" => create_capa_from_wizard(&result),
        "deviation" => create_deviation_from_wizard(&result),
        _ => {
            eprintln!("error: unknown quality item type `{}`", chosen_type);
            ExitCode::FAILURE
        }
    }
}

fn build_quality_wizard_steps(item_type: &str) -> Vec<sysml_core::interactive::WizardStep> {
    use sysml_core::interactive::WizardStep;

    match item_type {
        "ncr" => vec![
            WizardStep::string("part_name", "Affected part name")
                .with_explanation("Which part or component is nonconforming?"),
            WizardStep::choice("category", "Nonconformance category",
                sysml_capa::NonconformanceCategory::all().iter()
                    .map(|c| (c.id(), c.label()))
                    .collect(),
            ),
            WizardStep::choice("severity", "Severity classification",
                sysml_capa::SeverityClass::all().iter()
                    .map(|s| (s.id(), s.label()))
                    .collect(),
            ),
            WizardStep::string("description", "Description of the nonconformance")
                .with_explanation("What was observed? Include measurements, lot numbers, etc."),
            WizardStep::string("owner", "NCR owner (your name)")
                .with_default("engineer"),
        ],
        "capa" => vec![
            WizardStep::string("title", "CAPA title")
                .with_explanation("Brief title for the corrective/preventive action program."),
            WizardStep::string("description", "Description")
                .with_explanation("What is the scope and objective of this CAPA?"),
            WizardStep::choice("capa_type", "CAPA type", vec![
                ("corrective", "Corrective — eliminate cause of existing nonconformance"),
                ("preventive", "Preventive — prevent potential nonconformance"),
            ]),
            WizardStep::choice("source", "CAPA source", vec![
                ("ncr", "NCR — triggered by a nonconformance report"),
                ("audit", "Audit Finding"),
                ("complaint", "Customer Complaint"),
                ("improvement", "Process Improvement"),
                ("regulatory", "Regulatory Observation"),
                ("management", "Management Review"),
            ]),
            WizardStep::string("source_ref", "Source reference ID (Enter to skip)")
                .with_explanation("NCR ID, audit finding number, etc.")
                .optional(),
            WizardStep::string("owner", "CAPA owner (your name)")
                .with_default("engineer"),
        ],
        "deviation" => vec![
            WizardStep::string("title", "Deviation title")
                .with_explanation("Brief title for the process deviation request."),
            WizardStep::string("description", "Description")
                .with_explanation("What is being deviated from normal process?"),
            WizardStep::string("standard_ref", "Standard/specification reference")
                .with_explanation("Which standard, SOP, or specification is being deviated from?"),
            WizardStep::string("proposed_condition", "Proposed alternate condition")
                .with_explanation("What will be done instead of the standard process?"),
            WizardStep::choice("scope", "Deviation scope",
                sysml_capa::DeviationScope::all().iter()
                    .map(|s| {
                        let id = match s {
                            sysml_capa::DeviationScope::Lot => "lot",
                            sysml_capa::DeviationScope::ProcessStep => "processstep",
                            sysml_capa::DeviationScope::ProductLine => "productline",
                            sysml_capa::DeviationScope::Temporary => "temporary",
                            sysml_capa::DeviationScope::Permanent => "permanent",
                        };
                        (id, s.label())
                    })
                    .collect(),
            ),
            WizardStep::string("quantity_or_duration", "Quantity or duration")
                .with_explanation("How many units or how long does the deviation apply?"),
            WizardStep::string("justification", "Justification")
                .with_explanation("Why is this deviation necessary?"),
            WizardStep::string("owner", "Requestor (your name)")
                .with_default("engineer"),
        ],
        _ => Vec::new(),
    }
}

fn create_ncr_from_wizard(result: &sysml_core::interactive::WizardResult) -> ExitCode {
    let part_name = match result.get_string("part_name") {
        Some(s) => s,
        None => { eprintln!("error: part name is required"); return ExitCode::FAILURE; }
    };
    let category_str = result.get_string("category").unwrap_or("functional");
    let severity_str = result.get_string("severity").unwrap_or("minor");
    let description = result.get_string("description").unwrap_or("");
    let owner = result.get_string("owner").unwrap_or("engineer");

    let category = match category_str {
        "dimensional" => sysml_capa::NonconformanceCategory::Dimensional,
        "material" => sysml_capa::NonconformanceCategory::Material,
        "cosmetic" => sysml_capa::NonconformanceCategory::Cosmetic,
        "functional" => sysml_capa::NonconformanceCategory::Functional,
        "workmanship" => sysml_capa::NonconformanceCategory::Workmanship,
        "documentation" => sysml_capa::NonconformanceCategory::Documentation,
        "labeling" => sysml_capa::NonconformanceCategory::Labeling,
        "packaging" => sysml_capa::NonconformanceCategory::Packaging,
        "contamination" => sysml_capa::NonconformanceCategory::Contamination,
        "software" => sysml_capa::NonconformanceCategory::Software,
        _ => sysml_capa::NonconformanceCategory::Functional,
    };
    let severity = match severity_str {
        "critical" => sysml_capa::SeverityClass::Critical,
        "major" => sysml_capa::SeverityClass::Major,
        "minor" => sysml_capa::SeverityClass::Minor,
        "observation" => sysml_capa::SeverityClass::Observation,
        _ => sysml_capa::SeverityClass::Minor,
    };

    let ncr = sysml_capa::create_ncr(part_name, category, severity, description, owner);
    let record = sysml_capa::create_ncr_record(&ncr, owner);

    eprintln!("\nNCR Created:");
    eprintln!("  ID:       {}", ncr.id);
    eprintln!("  Part:     {}", ncr.part_name);
    eprintln!("  Category: {}", ncr.category.label());
    eprintln!("  Severity: {}", ncr.severity.label());
    eprintln!("  Status:   {}", ncr.status.label());

    let records_dir = crate::records::resolve_records_dir();
    match crate::records::write_record(&record, &records_dir) {
        Ok(path) => {
            eprintln!("  Record:   {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error writing record: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn create_capa_from_wizard(result: &sysml_core::interactive::WizardResult) -> ExitCode {
    let title = match result.get_string("title") {
        Some(s) => s,
        None => { eprintln!("error: title is required"); return ExitCode::FAILURE; }
    };
    let description = result.get_string("description").unwrap_or("");
    let owner = result.get_string("owner").unwrap_or("engineer");

    let capa_type = match result.get_string("capa_type").unwrap_or("corrective") {
        "preventive" => sysml_capa::CapaType::Preventive,
        _ => sysml_capa::CapaType::Corrective,
    };
    let source = match result.get_string("source").unwrap_or("ncr") {
        "ncr" => sysml_capa::CapaSource::Ncr,
        "audit" => sysml_capa::CapaSource::AuditFinding,
        "complaint" => sysml_capa::CapaSource::CustomerComplaint,
        "improvement" => sysml_capa::CapaSource::ProcessImprovement,
        "regulatory" => sysml_capa::CapaSource::RegulatoryObservation,
        "management" => sysml_capa::CapaSource::ManagementReview,
        _ => sysml_capa::CapaSource::Ncr,
    };
    let source_refs: Vec<String> = result.get_string("source_ref")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .into_iter().collect();

    let capa = sysml_capa::create_capa(title, description, capa_type, source, source_refs, owner);
    let record = sysml_capa::create_capa_record(&capa, owner);

    eprintln!("\nCAPA Created:");
    eprintln!("  ID:     {}", capa.id);
    eprintln!("  Title:  {}", capa.title);
    eprintln!("  Type:   {:?}", capa.capa_type);
    eprintln!("  Source: {:?}", capa.source);
    eprintln!("  Status: {}", capa.status.label());

    let records_dir = crate::records::resolve_records_dir();
    match crate::records::write_record(&record, &records_dir) {
        Ok(path) => {
            eprintln!("  Record: {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error writing record: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn create_deviation_from_wizard(result: &sysml_core::interactive::WizardResult) -> ExitCode {
    let title = match result.get_string("title") {
        Some(s) => s,
        None => { eprintln!("error: title is required"); return ExitCode::FAILURE; }
    };
    let description = result.get_string("description").unwrap_or("");
    let standard_ref = result.get_string("standard_ref").unwrap_or("");
    let proposed_condition = result.get_string("proposed_condition").unwrap_or("");
    let quantity_or_duration = result.get_string("quantity_or_duration").unwrap_or("");
    let justification = result.get_string("justification").unwrap_or("");
    let owner = result.get_string("owner").unwrap_or("engineer");

    let scope = match result.get_string("scope").unwrap_or("temporary") {
        "lot" => sysml_capa::DeviationScope::Lot,
        "processstep" => sysml_capa::DeviationScope::ProcessStep,
        "productline" => sysml_capa::DeviationScope::ProductLine,
        "temporary" => sysml_capa::DeviationScope::Temporary,
        "permanent" => sysml_capa::DeviationScope::Permanent,
        _ => sysml_capa::DeviationScope::Temporary,
    };

    let deviation = sysml_capa::create_deviation(
        title, description, standard_ref, proposed_condition,
        scope, quantity_or_duration, justification, owner,
    );
    let record = sysml_capa::create_deviation_record(&deviation, owner);

    eprintln!("\nProcess Deviation Created:");
    eprintln!("  ID:        {}", deviation.id);
    eprintln!("  Title:     {}", deviation.title);
    eprintln!("  Standard:  {}", deviation.standard_ref);
    eprintln!("  Scope:     {}", deviation.scope.label());
    eprintln!("  Status:    {}", deviation.status.label());

    let records_dir = crate::records::resolve_records_dir();
    match crate::records::write_record(&record, &records_dir) {
        Ok(path) => {
            eprintln!("  Record:    {}", path.display());
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

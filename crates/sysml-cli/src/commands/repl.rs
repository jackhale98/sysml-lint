/// Interactive REPL for exploring SysML v2 models.

use std::path::PathBuf;
use std::process::ExitCode;

use rustyline::DefaultEditor;
use sysml_core::model::Model;
use sysml_core::parser as sysml_parser;

pub fn run(files: &[PathBuf]) -> ExitCode {
    let (files, _) = crate::files_or_project(files);
    if files.is_empty() {
        eprintln!("error: no SysML files found.");
        return ExitCode::FAILURE;
    }

    let model = load_model(&files);
    let def_count = model.definitions.len();
    let usage_count = model.usages.len();

    println!("sysml repl — {} definitions, {} usages loaded from {} file(s)",
        def_count, usage_count, files.len());
    println!("Type 'help' for commands, 'quit' to exit.\n");

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("error: cannot initialize line editor: {}", e);
            return ExitCode::FAILURE;
        }
    };

    loop {
        let readline = rl.readline("sysml> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);
                if !dispatch(&model, line) {
                    break;
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                eprintln!("error: {}", e);
                break;
            }
        }
    }

    ExitCode::SUCCESS
}

fn load_model(files: &[PathBuf]) -> Model {
    let mut merged = Model::new("repl".to_string());
    for file_path in files {
        let path_str = file_path.to_string_lossy().to_string();
        if let Ok(source) = std::fs::read_to_string(file_path) {
            let m = sysml_parser::parse_file(&path_str, &source);
            merged.definitions.extend(m.definitions);
            merged.usages.extend(m.usages);
            merged.connections.extend(m.connections);
            merged.flows.extend(m.flows);
            merged.satisfactions.extend(m.satisfactions);
            merged.verifications.extend(m.verifications);
            merged.allocations.extend(m.allocations);
            merged.type_references.extend(m.type_references);
            merged.imports.extend(m.imports);
            merged.comments.extend(m.comments);
            merged.views.extend(m.views);
        }
    }
    merged
}

/// Dispatch a REPL command. Returns false to quit.
fn dispatch(model: &Model, input: &str) -> bool {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).unwrap_or(&"").trim();

    match cmd {
        "quit" | "exit" | "q" => return false,

        "help" | "h" | "?" => {
            println!("Commands:");
            println!("  list [kind]          List elements (parts, ports, actions, states, ...)");
            println!("  show <name>          Show element details");
            println!("  find <pattern>       Search by name pattern");
            println!("  deps <name>          Show dependencies");
            println!("  trace                Show requirements traceability");
            println!("  rollup <root> <attr> Compute attribute rollup");
            println!("  stats                Model statistics");
            println!("  defs                 List all definitions");
            println!("  usages               List all usages");
            println!("  help                 Show this help");
            println!("  quit                 Exit REPL");
        }

        "list" | "ls" => {
            let kind_filter = if args.is_empty() { None } else { Some(args) };
            for def in &model.definitions {
                if let Some(k) = kind_filter {
                    if !def.kind.label().contains(k) {
                        continue;
                    }
                }
                println!("  {:14} {} {}",
                    def.kind.label(),
                    def.name,
                    def.parent_def.as_ref().map(|p| format!("(in {})", p)).unwrap_or_default());
            }
            for usage in &model.usages {
                if let Some(k) = kind_filter {
                    if !usage.kind.contains(k) {
                        continue;
                    }
                }
                println!("  {:14} {}{}{}",
                    usage.kind,
                    usage.name,
                    usage.type_ref.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default(),
                    usage.parent_def.as_ref().map(|p| format!(" (in {})", p)).unwrap_or_default());
            }
        }

        "defs" => {
            for def in &model.definitions {
                println!("  {:14} {}", def.kind.label(), def.name);
            }
            println!("({} definitions)", model.definitions.len());
        }

        "usages" => {
            for usage in &model.usages {
                println!("  {:14} {}{}",
                    usage.kind, usage.name,
                    usage.type_ref.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default());
            }
            println!("({} usages)", model.usages.len());
        }

        "show" => {
            if args.is_empty() {
                println!("Usage: show <element_name>");
                return true;
            }
            if let Some(def) = model.find_def(args) {
                println!("  {} `{}`", def.kind.label(), def.name);
                if let Some(ref st) = def.super_type {
                    println!("  Specializes: {}", st);
                }
                if let Some(ref doc) = def.doc {
                    println!("  Doc: {}", doc);
                }
                if let Some(ref parent) = def.parent_def {
                    println!("  In: {}", parent);
                }
                let members = model.usages_in_def(&def.name);
                if !members.is_empty() {
                    println!("  Members ({}):", members.len());
                    for u in &members {
                        println!("    {} {}{}",
                            u.kind, u.name,
                            u.type_ref.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default());
                    }
                }
            } else if let Some(usage) = model.usages.iter().find(|u| u.name == args) {
                println!("  {} `{}`", usage.kind, usage.name);
                if let Some(ref tr) = usage.type_ref {
                    println!("  Type: {}", tr);
                }
                if let Some(ref parent) = usage.parent_def {
                    println!("  In: {}", parent);
                }
            } else {
                println!("Element `{}` not found.", args);
            }
        }

        "find" => {
            if args.is_empty() {
                println!("Usage: find <pattern>");
                return true;
            }
            let pat = args.to_lowercase();
            let mut count = 0;
            for def in &model.definitions {
                if def.name.to_lowercase().contains(&pat) {
                    println!("  {:14} {}", def.kind.label(), def.name);
                    count += 1;
                }
            }
            for usage in &model.usages {
                if usage.name.to_lowercase().contains(&pat) {
                    println!("  {:14} {}{}", usage.kind, usage.name,
                        usage.type_ref.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default());
                    count += 1;
                }
            }
            println!("({} matches)", count);
        }

        "deps" => {
            if args.is_empty() {
                println!("Usage: deps <element_name>");
                return true;
            }
            let deps = sysml_core::query::dependency_analysis(model, args);
            println!("  Referenced by ({}):", deps.referenced_by.len());
            for r in &deps.referenced_by {
                println!("    {} ({}) via {}", r.name, r.kind, r.relationship);
            }
            println!("  Depends on ({}):", deps.depends_on.len());
            for d in &deps.depends_on {
                println!("    {} ({}) via {}", d.name, d.kind, d.relationship);
            }
        }

        "trace" => {
            let trace = sysml_core::query::trace_requirements(model);
            if trace.is_empty() {
                println!("No requirements found.");
            } else {
                println!("{:25} {:20} {:20}", "Requirement", "Satisfied By", "Verified By");
                println!("{}", "-".repeat(67));
                for t in &trace {
                    let sat = if t.satisfied_by.is_empty() { "-".to_string() } else { t.satisfied_by.join(", ") };
                    let ver = if t.verified_by.is_empty() { "-".to_string() } else { t.verified_by.join(", ") };
                    println!("{:25} {:20} {:20}", t.requirement, sat, ver);
                }
            }
        }

        "rollup" => {
            let parts: Vec<&str> = args.splitn(2, ' ').collect();
            if parts.len() < 2 {
                println!("Usage: rollup <root_def> <attribute>");
                return true;
            }
            let root = parts[0];
            let attr = parts[1];
            use sysml_core::sim::rollup::{evaluate_rollup, AggregationMethod, format_rollup_text};
            if model.find_def(root).is_none() {
                println!("Definition `{}` not found.", root);
                return true;
            }
            let result = evaluate_rollup(model, root, attr, AggregationMethod::Sum);
            print!("{}", format_rollup_text(&result));
        }

        "stats" => {
            println!("  Definitions:    {}", model.definitions.len());
            println!("  Usages:         {}", model.usages.len());
            println!("  Connections:    {}", model.connections.len());
            println!("  Flows:          {}", model.flows.len());
            println!("  Satisfactions:  {}", model.satisfactions.len());
            println!("  Verifications:  {}", model.verifications.len());
            println!("  Allocations:    {}", model.allocations.len());
            println!("  Imports:        {}", model.imports.len());
        }

        _ => {
            println!("Unknown command: `{}`. Type 'help' for available commands.", cmd);
        }
    }

    true
}

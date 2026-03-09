use std::process::ExitCode;
use std::time::Instant;

use sysml_core::cache::Cache;
use sysml_core::index::Indexer;
use sysml_core::project::discover_project;

use crate::Cli;

pub(crate) fn run(cli: &Cli, full: bool, stats: bool) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot determine current directory: {e}");
            return ExitCode::from(1);
        }
    };

    let (project_root, config) = match discover_project(&cwd) {
        Some(result) => result,
        None => {
            eprintln!("error: no sysml project found (looked from {}).", cwd.display());
            eprintln!("  Run `sysml init` to create a project.");
            return ExitCode::from(1);
        }
    };

    let model_root = project_root.join(&config.project.model_root);
    if !model_root.is_dir() {
        eprintln!(
            "error: model root `{}` is not a directory.",
            model_root.display()
        );
        return ExitCode::from(1);
    }

    let mut cache = Cache::new();

    // If full rebuild, also index records.
    let started = Instant::now();

    Indexer::index_directory(&mut cache, &model_root);

    if full {
        let records_dir = project_root.join(&config.defaults.output_dir);
        Indexer::index_records(&mut cache, &records_dir);
    }

    let elapsed = started.elapsed();
    let cache_stats = cache.stats();

    if stats {
        print_stats(cli, &cache_stats, elapsed);
    } else {
        print_summary(cli, &config.project.name, &model_root, &cache_stats, elapsed, full);
    }

    ExitCode::SUCCESS
}

fn print_stats(
    cli: &Cli,
    stats: &sysml_core::cache::CacheStats,
    elapsed: std::time::Duration,
) {
    if cli.format == "json" {
        let json = serde_json::json!({
            "nodes": stats.nodes,
            "edges": stats.edges,
            "records": stats.records,
            "ref_edges": stats.ref_edges,
            "elapsed_ms": elapsed.as_millis(),
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Cache Statistics");
        println!("{}", "=".repeat(40));
        println!("Nodes (elements):   {}", stats.nodes);
        println!("Edges (relations):  {}", stats.edges);
        println!("Records:            {}", stats.records);
        println!("Reference edges:    {}", stats.ref_edges);
        println!();
        println!("Indexed in {:.1}ms", elapsed.as_secs_f64() * 1000.0);
    }
}

fn print_summary(
    cli: &Cli,
    project_name: &str,
    model_root: &std::path::Path,
    stats: &sysml_core::cache::CacheStats,
    elapsed: std::time::Duration,
    full: bool,
) {
    if cli.format == "json" {
        let mut json = serde_json::json!({
            "project": project_name,
            "model_root": model_root.to_string_lossy(),
            "nodes": stats.nodes,
            "edges": stats.edges,
            "elapsed_ms": elapsed.as_millis(),
        });
        if full {
            json["records"] = serde_json::json!(stats.records);
            json["ref_edges"] = serde_json::json!(stats.ref_edges);
        }
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        let project_label = if project_name.is_empty() {
            "(unnamed)".to_string()
        } else {
            project_name.to_string()
        };

        if !cli.quiet {
            println!(
                "Indexed project `{}` from `{}`",
                project_label,
                model_root.display()
            );
            println!(
                "  {} elements, {} relationships indexed in {:.1}ms",
                stats.nodes,
                stats.edges,
                elapsed.as_secs_f64() * 1000.0
            );
            if full {
                println!(
                    "  {} records, {} reference edges",
                    stats.records, stats.ref_edges
                );
            }
        }
    }
}

/// Quality control CLI commands.

use std::process::ExitCode;

use crate::QcCommand;

pub fn run(cli: &crate::Cli, kind: &QcCommand) -> ExitCode {
    match kind {
        QcCommand::SampleSize {
            lot_size,
            aql,
            level,
        } => run_sample_size(cli, *lot_size, *aql, level),
        QcCommand::Capability {
            usl,
            lsl,
            values,
        } => run_capability(cli, *usl, *lsl, values),
    }
}

fn run_sample_size(
    cli: &crate::Cli,
    lot_size: usize,
    aql: f64,
    level_str: &str,
) -> ExitCode {
    let level = match level_str.to_lowercase().as_str() {
        "reduced" => sysml_qc::InspectionLevel::Reduced,
        "normal" => sysml_qc::InspectionLevel::Normal,
        "tightened" => sysml_qc::InspectionLevel::Tightened,
        other => {
            eprintln!(
                "error: unknown inspection level `{other}`. Use: reduced, normal, tightened"
            );
            return ExitCode::FAILURE;
        }
    };

    let (sample_size, accept, reject) = sysml_qc::sample_size_z14(lot_size, aql, &level);

    if cli.format == "json" {
        let result = serde_json::json!({
            "lot_size": lot_size,
            "aql": aql,
            "level": level_str,
            "sample_size": sample_size,
            "accept_number": accept,
            "reject_number": reject,
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
        println!("ANSI Z1.4 Sampling Plan");
        println!("  Lot size:      {lot_size}");
        println!("  AQL:           {aql}%");
        println!("  Level:         {level_str}");
        println!("  Sample size:   {sample_size}");
        println!("  Accept number: {accept}");
        println!("  Reject number: {reject}");
    }

    ExitCode::SUCCESS
}

fn run_capability(cli: &crate::Cli, usl: f64, lsl: f64, values: &[f64]) -> ExitCode {
    if values.is_empty() {
        eprintln!("error: no values provided. Use --values to supply comma-separated measurements.");
        return ExitCode::FAILURE;
    }

    if usl <= lsl {
        eprintln!("error: USL ({usl}) must be greater than LSL ({lsl})");
        return ExitCode::FAILURE;
    }

    let (cp, cpk) = sysml_qc::process_capability(values, usl, lsl);

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let sigma = variance.sqrt();

    if cli.format == "json" {
        let result = serde_json::json!({
            "usl": usl,
            "lsl": lsl,
            "n": values.len(),
            "mean": mean,
            "sigma": sigma,
            "cp": cp,
            "cpk": cpk,
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
        println!("Process Capability Analysis");
        println!("  USL:   {usl:.4}");
        println!("  LSL:   {lsl:.4}");
        println!("  N:     {}", values.len());
        println!("  Mean:  {mean:.4}");
        println!("  Sigma: {sigma:.4}");
        if cp.is_infinite() {
            println!("  Cp:    inf (zero variation)");
            println!("  Cpk:   inf (zero variation)");
        } else {
            println!("  Cp:    {cp:.4}");
            println!("  Cpk:   {cpk:.4}");
        }

        // Provide a quick interpretation.
        if !cp.is_infinite() {
            let assessment = if cpk >= 1.33 {
                "capable"
            } else if cpk >= 1.0 {
                "marginally capable"
            } else {
                "NOT capable"
            };
            println!("  Assessment: {assessment} (Cpk threshold: 1.33)");
        }
    }

    ExitCode::SUCCESS
}

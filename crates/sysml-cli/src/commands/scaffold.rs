/// Scaffolding CLI commands — generate templates and example projects.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::ScaffoldCommand;

pub fn run(kind: &ScaffoldCommand) -> ExitCode {
    match kind {
        ScaffoldCommand::Element {
            kind: elem_kind,
            name,
            extends,
            doc,
            no_comments,
        } => run_element(elem_kind, name, extends.as_deref(), doc.as_deref(), *no_comments),
        ScaffoldCommand::Example { name, output } => run_example(name, output.as_ref()),
        ScaffoldCommand::ListExamples => run_list_examples(),
        ScaffoldCommand::ListKinds => run_list_kinds(),
    }
}

fn run_element(
    kind: &str,
    name: &str,
    extends: Option<&str>,
    doc: Option<&str>,
    no_comments: bool,
) -> ExitCode {
    let options = sysml_scaffold::ScaffoldOptions {
        extends: extends.map(|s| s.to_string()),
        doc: doc.map(|s| s.to_string()),
        members: Vec::new(),
        with_teaching_comments: !no_comments,
    };

    match sysml_scaffold::scaffold_element(kind, name, &options) {
        Ok(text) => {
            print!("{}", text);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_example(name: &str, output: Option<&PathBuf>) -> ExitCode {
    let files = match sysml_scaffold::scaffold_example(name) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {}", e);
            let examples = sysml_scaffold::list_examples();
            eprintln!("Available examples:");
            for (n, desc) in examples {
                eprintln!("  {:<20} {}", n, desc);
            }
            return ExitCode::FAILURE;
        }
    };

    let out_dir = output.cloned().unwrap_or_else(|| PathBuf::from("."));

    for (filename, content) in &files {
        let path = out_dir.join(filename);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("error: cannot create directory `{}`: {}", parent.display(), e);
                    return ExitCode::FAILURE;
                }
            }
        }

        if let Err(e) = std::fs::write(&path, content) {
            eprintln!("error: cannot write `{}`: {}", path.display(), e);
            return ExitCode::FAILURE;
        }
        eprintln!("  created {}", path.display());
    }

    eprintln!("Example `{}` scaffolded ({} files).", name, files.len());
    ExitCode::SUCCESS
}

fn run_list_examples() -> ExitCode {
    let examples = sysml_scaffold::list_examples();
    println!("Available example projects:");
    for (name, desc) in examples {
        println!("  {:<20} {}", name, desc);
    }
    ExitCode::SUCCESS
}

fn run_list_kinds() -> ExitCode {
    let kinds = sysml_scaffold::list_element_kinds();
    println!("Available element kinds for scaffolding:");
    for (kind, desc) in kinds {
        println!("  {:<20} {}", kind, desc);
    }
    ExitCode::SUCCESS
}

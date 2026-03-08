use std::path::Path;

fn main() {
    // Look for tree-sitter-sysml grammar source in multiple locations
    let candidates = [
        "tree-sitter-sysml/src",     // vendored / submodule
        "../tree-sitter-sysml/src",  // sibling directory
    ];

    let grammar_dir = candidates
        .iter()
        .map(Path::new)
        .find(|p| p.join("parser.c").exists())
        .unwrap_or_else(|| {
            panic!(
                "tree-sitter-sysml grammar not found. Tried:\n{}",
                candidates
                    .iter()
                    .map(|c| format!("  - {}/parser.c", c))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });

    cc::Build::new()
        .include(grammar_dir)
        .file(grammar_dir.join("parser.c"))
        .warnings(false)
        .compile("tree-sitter-sysml");

    println!("cargo:rerun-if-changed={}", grammar_dir.display());
}

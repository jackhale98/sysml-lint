use std::path::Path;

fn main() {
    // Look for tree-sitter-sysml grammar source relative to workspace root.
    // Build scripts run from the crate directory (crates/sysml-core/),
    // so we look up two levels to the workspace root.
    let candidates = [
        "../../tree-sitter-sysml/src",    // submodule in workspace root
        "../../../tree-sitter-sysml/src", // sibling of workspace root
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

use sysml_core::model::{simple_name, Model, Span};

use crate::state::WorldState;

/// Find the identifier name at a given byte offset in the source.
/// Checks type_references, usage type_refs, and definition super_types.
pub fn find_identifier_at_offset(model: &Model, source: &str, offset: usize) -> Option<String> {
    // Check type_references first (most common case)
    for tr in &model.type_references {
        if tr.span.start_byte <= offset && offset < tr.span.end_byte {
            return Some(tr.name.clone());
        }
    }

    // Check usage type_refs
    for usage in &model.usages {
        if let Some(ref type_ref) = usage.type_ref {
            // The type_ref text is at the end of the usage span; approximate by
            // checking if offset falls in the usage span and the text at offset
            // matches the type_ref name
            if usage.span.start_byte <= offset && offset < usage.span.end_byte {
                let usage_text = &source[usage.span.start_byte..usage.span.end_byte];
                if let Some(pos) = usage_text.rfind(simple_name(type_ref)) {
                    let abs_start = usage.span.start_byte + pos;
                    let abs_end = abs_start + simple_name(type_ref).len();
                    if abs_start <= offset && offset < abs_end {
                        return Some(simple_name(type_ref).to_string());
                    }
                }
            }
        }
    }

    // Check definition super_types
    for def in &model.definitions {
        if let Some(ref super_type) = def.super_type {
            if def.span.start_byte <= offset && offset < def.span.end_byte {
                let def_text = &source[def.span.start_byte..def.span.end_byte];
                let st_name = simple_name(super_type);
                if let Some(pos) = def_text.rfind(st_name) {
                    let abs_start = def.span.start_byte + pos;
                    let abs_end = abs_start + st_name.len();
                    if abs_start <= offset && offset < abs_end {
                        return Some(st_name.to_string());
                    }
                }
            }
        }
    }

    None
}

#[allow(dead_code)] // used in tests
/// Resolve a name to a definition span within the same file.
pub fn goto_definition_in_file(model: &Model, name: &str) -> Option<Span> {
    let simple = simple_name(name);
    model.find_def(simple).map(|d| d.span.clone())
}

/// Resolve a name: try in-file first, then workspace defs. Returns (uri, span).
pub fn goto_definition(
    model: &Model,
    name: &str,
    current_uri: &str,
    state: &WorldState,
) -> Option<(String, Span)> {
    let simple = simple_name(name);

    // Try same-file first
    if let Some(def) = model.find_def(simple) {
        return Some((current_uri.to_string(), def.span.clone()));
    }

    // Try workspace defs
    if let Some(loc) = state.workspace_defs.get(simple) {
        return Some((loc.uri.clone(), loc.span.clone()));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysml_core::parser::parse_file;

    #[test]
    fn find_type_ref_at_cursor() {
        let source = "part def Engine;\npart def Vehicle {\n    part engine : Engine;\n}\n";
        let model = parse_file("test.sysml", source);
        // Find the byte offset of "Engine" in the type reference on line 3
        let engine_ref_pos = source.rfind("Engine").unwrap();
        let name = find_identifier_at_offset(&model, source, engine_ref_pos);
        assert_eq!(name.as_deref(), Some("Engine"));
    }

    #[test]
    fn goto_def_resolves_type_ref() {
        let source = "part def Engine;\npart def Vehicle {\n    part engine : Engine;\n}\n";
        let model = parse_file("test.sysml", source);
        let span = goto_definition_in_file(&model, "Engine");
        assert!(span.is_some());
        let span = span.unwrap();
        // Engine def is on line 1 (1-based)
        assert_eq!(span.start_row, 1);
    }

    #[test]
    fn cursor_on_definition_name_still_resolves() {
        // Clicking on a definition name should still find the definition
        let source = "part def Engine;\n";
        let model = parse_file("test.sysml", source);
        let span = goto_definition_in_file(&model, "Engine");
        assert!(span.is_some());
    }

    #[test]
    fn unknown_type_returns_none() {
        let source = "part def Vehicle;\n";
        let model = parse_file("test.sysml", source);
        let span = goto_definition_in_file(&model, "Unknown");
        assert!(span.is_none());
    }

    #[test]
    fn supertype_navigation() {
        let source = "part def Base;\npart def Sub :> Base;\n";
        let model = parse_file("test.sysml", source);
        // Find "Base" in the supertype position
        let base_pos = source.rfind("Base").unwrap();
        let name = find_identifier_at_offset(&model, source, base_pos);
        assert_eq!(name.as_deref(), Some("Base"));
        // Resolve it
        let span = goto_definition_in_file(&model, "Base");
        assert!(span.is_some());
        assert_eq!(span.unwrap().start_row, 1);
    }

    #[test]
    fn cross_file_resolution() {
        let source_a = "part def Engine;\n";
        let source_b = "part def Vehicle {\n    part engine : Engine;\n}\n";
        let model_a = parse_file("a.sysml", source_a);
        let model_b = parse_file("b.sysml", source_b);

        let state = WorldState::new();
        state.index_model_defs("file:///a.sysml", &model_a);
        state.index_model_defs("file:///b.sysml", &model_b);

        // From b.sysml, resolve Engine -> should find it in a.sysml
        let result = goto_definition(&model_b, "Engine", "file:///b.sysml", &state);
        assert!(result.is_some());
        let (uri, _span) = result.unwrap();
        assert_eq!(uri, "file:///a.sysml");
    }
}

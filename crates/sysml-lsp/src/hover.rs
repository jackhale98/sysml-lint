use sysml_core::model::{simple_name, Model};

/// Build a markdown hover string for a definition name in the model.
pub fn hover_info(model: &Model, name: &str) -> Option<String> {
    let simple = simple_name(name);
    let def = model.find_def(simple)?;

    let mut parts = Vec::new();

    // Header: kind + name + supertype
    let mut header = format!("**{}** `{}`", def.kind.label(), def.name);
    if let Some(ref st) = def.super_type {
        header.push_str(&format!(" : `{}`", st));
    }
    parts.push(header);

    // Doc comment
    if let Some(ref doc) = def.doc {
        parts.push(String::new());
        parts.push(doc.clone());
    }

    // Members (usages in this def)
    let members = model.usages_in_def(&def.name);
    if !members.is_empty() {
        parts.push(String::new());
        parts.push("**Members:**".to_string());
        for u in &members {
            let mut member = format!("- `{}`", u.name);
            if let Some(ref tr) = u.type_ref {
                member.push_str(&format!(" : `{}`", tr));
            }
            if let Some(ref mult) = u.multiplicity {
                member.push_str(&format!(" `{}`", mult));
            }
            parts.push(member);
        }
    }

    Some(parts.join("\n"))
}

/// Build a markdown hover string for a usage name in the model.
pub fn hover_usage_info(model: &Model, name: &str) -> Option<String> {
    let usage = model.usages.iter().find(|u| u.name == name)?;

    let mut parts = Vec::new();

    let mut header = format!("**{}** `{}`", usage.kind, usage.name);
    if let Some(ref tr) = usage.type_ref {
        header.push_str(&format!(" : `{}`", tr));
    }
    parts.push(header);

    if let Some(ref parent) = usage.parent_def {
        parts.push(format!("In `{}`", parent));
    }

    Some(parts.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysml_core::parser::parse_file;

    #[test]
    fn hover_def_with_doc_and_members() {
        let source = "part def Vehicle {\n    doc /* A motor vehicle. */\n    part engine : Engine;\n}\n";
        let model = parse_file("test.sysml", source);
        let info = hover_info(&model, "Vehicle");
        assert!(info.is_some());
        let text = info.unwrap();
        assert!(text.contains("**part def**"), "should show kind: {}", text);
        assert!(text.contains("`Vehicle`"), "should show name: {}", text);
        assert!(
            text.contains("A motor vehicle"),
            "should show doc: {}",
            text
        );
        assert!(
            text.contains("`engine`"),
            "should show member: {}",
            text
        );
        assert!(
            text.contains("`Engine`"),
            "should show member type: {}",
            text
        );
    }

    #[test]
    fn hover_def_with_supertype() {
        let source = "part def Base;\npart def Sub :> Base;\n";
        let model = parse_file("test.sysml", source);
        let info = hover_info(&model, "Sub");
        assert!(info.is_some());
        let text = info.unwrap();
        assert!(text.contains(": `Base`"), "should show supertype: {}", text);
    }

    #[test]
    fn hover_usage() {
        let source =
            "part def Engine;\npart def Vehicle {\n    part engine : Engine;\n}\n";
        let model = parse_file("test.sysml", source);
        let info = hover_usage_info(&model, "engine");
        assert!(info.is_some());
        let text = info.unwrap();
        assert!(text.contains("**part**"), "should show kind: {}", text);
        assert!(
            text.contains("`Engine`"),
            "should show type ref: {}",
            text
        );
        assert!(
            text.contains("In `Vehicle`"),
            "should show parent: {}",
            text
        );
    }

    #[test]
    fn hover_unknown_returns_none() {
        let source = "part def Vehicle;\n";
        let model = parse_file("test.sysml", source);
        assert!(hover_info(&model, "Unknown").is_none());
    }
}

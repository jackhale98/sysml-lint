/// SysML v2 boilerplate template generation.

use crate::model::DefKind;

/// Options for generating a SysML v2 definition template.
#[derive(Debug, Clone)]
pub struct TemplateOptions {
    pub kind: DefKind,
    pub name: String,
    pub super_type: Option<String>,
    pub is_abstract: bool,
    pub short_name: Option<String>,
    pub doc: Option<String>,
    pub members: Vec<MemberSpec>,
    /// View-specific: expose clauses (e.g., "Vehicle::*").
    pub exposes: Vec<String>,
    /// View-specific: filter kind (e.g., "part", "port").
    pub filter: Option<String>,
    /// Base indent level (number of spaces).
    pub indent: usize,
}

/// A member (usage) to include in a generated template.
#[derive(Debug, Clone)]
pub struct MemberSpec {
    pub usage_kind: String,
    pub name: String,
    pub type_ref: Option<String>,
    pub direction: Option<String>,
    pub multiplicity: Option<String>,
}

/// Parse a template kind string into a `DefKind`.
pub fn parse_template_kind(s: &str) -> Option<DefKind> {
    match s.to_lowercase().replace('-', " ").as_str() {
        "part def" | "part" => Some(DefKind::Part),
        "port def" | "port" => Some(DefKind::Port),
        "connection def" | "connection" => Some(DefKind::Connection),
        "interface def" | "interface" => Some(DefKind::Interface),
        "flow def" | "flow" => Some(DefKind::Flow),
        "action def" | "action" => Some(DefKind::Action),
        "state def" | "state" => Some(DefKind::State),
        "constraint def" | "constraint" => Some(DefKind::Constraint),
        "calc def" | "calc" => Some(DefKind::Calc),
        "requirement def" | "requirement" | "req" => Some(DefKind::Requirement),
        "use case def" | "use case" | "usecase" => Some(DefKind::UseCase),
        "enum def" | "enum" => Some(DefKind::Enum),
        "attribute def" | "attribute" | "attr" => Some(DefKind::Attribute),
        "item def" | "item" => Some(DefKind::Item),
        "view def" | "view" => Some(DefKind::View),
        "viewpoint def" | "viewpoint" => Some(DefKind::Viewpoint),
        "package" | "pkg" => Some(DefKind::Package),
        "allocation def" | "allocation" => Some(DefKind::Allocation),
        _ => None,
    }
}

/// Generate SysML v2 text for a definition template.
pub fn generate_template(opts: &TemplateOptions) -> String {
    let indent = " ".repeat(opts.indent);
    let inner_indent = " ".repeat(opts.indent + 4);
    let mut out = String::new();

    // Abstract modifier
    if opts.is_abstract {
        out.push_str(&format!("{}abstract ", indent));
    } else {
        out.push_str(&indent);
    }

    // Definition keyword
    out.push_str(opts.kind.label());

    // Short name (comes before name in SysML v2)
    if let Some(ref sn) = opts.short_name {
        out.push_str(&format!(" <{}>", sn));
    }

    // Name
    out.push_str(&format!(" {}", opts.name));

    // Supertype
    if let Some(ref st) = opts.super_type {
        out.push_str(&format!(" :> {}", st));
    }

    // Body
    let has_body = opts.doc.is_some() || !opts.members.is_empty()
        || !opts.exposes.is_empty() || opts.filter.is_some();
    if has_body {
        out.push_str(" {\n");

        // Doc comment
        if let Some(ref doc) = opts.doc {
            out.push_str(&format!("{}doc /* {} */\n", inner_indent, doc));
        }

        // View-specific: expose clauses
        for expose in &opts.exposes {
            out.push_str(&format!("{}expose {};\n", inner_indent, expose));
        }

        // View-specific: filter
        if let Some(ref f) = opts.filter {
            out.push_str(&format!("{}filter @type istype {};\n", inner_indent, f));
        }

        // Members — for enum defs, render as `enum <name>;`
        let is_enum = opts.kind == DefKind::Enum;
        for member in &opts.members {
            out.push_str(&inner_indent);

            if is_enum {
                // Enum members: just `enum <name>;`
                out.push_str(&format!("enum {};\n", member.name));
            } else {
                // Direction
                if let Some(ref dir) = member.direction {
                    out.push_str(&format!("{} ", dir));
                }

                // Usage kind and name
                out.push_str(&format!("{} {}", member.usage_kind, member.name));

                // Type reference
                if let Some(ref t) = member.type_ref {
                    out.push_str(&format!(" : {}", t));
                }

                // Multiplicity
                if let Some(ref m) = member.multiplicity {
                    out.push_str(&format!(" [{}]", m));
                }

                out.push_str(";\n");
            }
        }

        out.push_str(&format!("{}}}\n", indent));
    } else {
        out.push_str(";\n");
    }

    out
}

/// Parse generated SysML text through tree-sitter and return any syntax errors.
pub fn validate_generated(text: &str) -> Result<(), Vec<String>> {
    let model = crate::parser::parse_file("<generated>", text);
    if model.syntax_errors.is_empty() {
        Ok(())
    } else {
        Err(model
            .syntax_errors
            .iter()
            .map(|e| format!("line {}: {} ({})", e.span.start_row, e.message, e.context))
            .collect())
    }
}

/// Parse a member spec string like "part engine:Engine" or "in port fuelIn:FuelPort[2]".
///
/// Supported formats:
///   - `part engine:Engine`
///   - `in port fuelIn:FuelPort`
///   - `part wheels:Wheel[4]`
///   - `attribute name:String[0..1]`
pub fn parse_member_spec(s: &str) -> Option<MemberSpec> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut idx = 0;
    let direction = match parts.get(idx) {
        Some(&"in") | Some(&"out") | Some(&"inout") => {
            let d = parts[idx].to_string();
            idx += 1;
            Some(d)
        }
        _ => None,
    };

    let usage_kind = parts.get(idx)?.to_string();
    idx += 1;

    let name_and_type = parts.get(idx)?.to_string();
    let (name, type_ref, multiplicity) = parse_name_type_mult(&name_and_type);

    Some(MemberSpec {
        usage_kind,
        name,
        type_ref,
        direction,
        multiplicity,
    })
}

/// Parse `name[:Type[mult]]` into (name, Option<type>, Option<multiplicity>).
fn parse_name_type_mult(s: &str) -> (String, Option<String>, Option<String>) {
    if let Some((name, rest)) = s.split_once(':') {
        // rest could be "Type[mult]" or "Type"
        if let Some((type_ref, mult)) = rest.split_once('[') {
            let mult = mult.trim_end_matches(']');
            (name.to_string(), Some(type_ref.to_string()), Some(mult.to_string()))
        } else {
            (name.to_string(), Some(rest.to_string()), None)
        }
    } else if let Some((name, mult)) = s.split_once('[') {
        // name[mult] without type
        let mult = mult.trim_end_matches(']');
        (name.to_string(), None, Some(mult.to_string()))
    } else {
        (s.to_string(), None, None)
    }
}

/// Generate a connection usage with connect binding.
///
/// Returns text like:
/// ```text
/// connection tempConn : SensorConnection
///     connect tempSensor.dataOut to controller.tempIn;
/// ```
pub fn generate_connection_usage(
    name: &str,
    type_ref: Option<&str>,
    connect_endpoints: &str,
    indent: usize,
) -> String {
    let ind = " ".repeat(indent);
    let inner = " ".repeat(indent + 4);
    let type_part = type_ref
        .map(|t| format!(" : {}", t))
        .unwrap_or_default();
    format!(
        "{}connection {}{}\n{}connect {};\n",
        ind, name, type_part, inner, connect_endpoints,
    )
}

/// Generate a satisfy or verify relationship statement.
///
/// Returns text like:
/// ```text
/// satisfy requirement TemperatureAccuracy by WeatherStationUnit;
/// ```
pub fn generate_relationship(
    rel_kind: &str,
    requirement: &str,
    by: &str,
    indent: usize,
) -> String {
    let ind = " ".repeat(indent);
    format!("{}{} requirement {} by {};\n", ind, rel_kind, requirement, by)
}

/// Generate an import statement.
///
/// Returns text like:
/// ```text
/// import WeatherStation::*;
/// ```
pub fn generate_import(import_path: &str, indent: usize) -> String {
    let ind = " ".repeat(indent);
    format!("{}import {};\n", ind, import_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_simple_part_def() {
        let opts = TemplateOptions {
            kind: DefKind::Part,
            name: "Vehicle".to_string(),
            super_type: None,
            is_abstract: false,
            short_name: None,
            doc: None,
            members: Vec::new(),
            exposes: Vec::new(),
            filter: None,
            indent: 0,
        };
        let result = generate_template(&opts);
        assert_eq!(result.trim(), "part def Vehicle;");
    }

    #[test]
    fn generate_with_supertype_and_doc() {
        let opts = TemplateOptions {
            kind: DefKind::Part,
            name: "Vehicle".to_string(),
            super_type: Some("Base".to_string()),
            is_abstract: false,
            short_name: None,
            doc: Some("A vehicle definition".to_string()),
            members: Vec::new(),
            exposes: Vec::new(),
            filter: None,
            indent: 0,
        };
        let result = generate_template(&opts);
        assert!(result.contains("part def Vehicle :> Base {"));
        assert!(result.contains("doc /* A vehicle definition */"));
    }

    #[test]
    fn generate_abstract_with_members() {
        let opts = TemplateOptions {
            kind: DefKind::Part,
            name: "Vehicle".to_string(),
            super_type: None,
            is_abstract: true,
            short_name: Some("V".to_string()),
            doc: None,
            members: vec![
                MemberSpec {
                    usage_kind: "part".to_string(),
                    name: "engine".to_string(),
                    type_ref: Some("Engine".to_string()),
                    direction: None,
                    multiplicity: None,
                },
            ],
            exposes: Vec::new(),
            filter: None,
            indent: 0,
        };
        let result = generate_template(&opts);
        assert!(result.contains("abstract part def <V> Vehicle {"));
        assert!(result.contains("part engine : Engine;"));
    }

    #[test]
    fn generate_port_def_with_direction() {
        let opts = TemplateOptions {
            kind: DefKind::Port,
            name: "FuelPort".to_string(),
            super_type: None,
            is_abstract: false,
            short_name: None,
            doc: None,
            members: vec![
                MemberSpec {
                    usage_kind: "item".to_string(),
                    name: "fuel".to_string(),
                    type_ref: Some("FuelType".to_string()),
                    direction: Some("in".to_string()),
                    multiplicity: None,
                },
            ],
            exposes: Vec::new(),
            filter: None,
            indent: 0,
        };
        let result = generate_template(&opts);
        assert!(result.contains("port def FuelPort {"));
        assert!(result.contains("in item fuel : FuelType;"));
    }

    #[test]
    fn generate_indented() {
        let opts = TemplateOptions {
            kind: DefKind::Part,
            name: "Engine".to_string(),
            super_type: None,
            is_abstract: false,
            short_name: None,
            doc: None,
            members: Vec::new(),
            exposes: Vec::new(),
            filter: None,
            indent: 4,
        };
        let result = generate_template(&opts);
        assert!(result.starts_with("    part def Engine;"));
    }

    #[test]
    fn parse_template_kind_variants() {
        assert_eq!(parse_template_kind("part-def"), Some(DefKind::Part));
        assert_eq!(parse_template_kind("port def"), Some(DefKind::Port));
        assert_eq!(parse_template_kind("action-def"), Some(DefKind::Action));
        assert_eq!(parse_template_kind("state"), Some(DefKind::State));
        assert_eq!(parse_template_kind("requirement"), Some(DefKind::Requirement));
        assert_eq!(parse_template_kind("package"), Some(DefKind::Package));
        assert_eq!(parse_template_kind("nonsense"), None);
    }

    #[test]
    fn parse_member_spec_variants() {
        let m = parse_member_spec("part engine:Engine").unwrap();
        assert_eq!(m.usage_kind, "part");
        assert_eq!(m.name, "engine");
        assert_eq!(m.type_ref.as_deref(), Some("Engine"));
        assert!(m.direction.is_none());
        assert!(m.multiplicity.is_none());

        let m2 = parse_member_spec("in port fuelIn:FuelPort").unwrap();
        assert_eq!(m2.direction.as_deref(), Some("in"));
        assert_eq!(m2.usage_kind, "port");
        assert_eq!(m2.name, "fuelIn");
    }

    #[test]
    fn parse_member_spec_with_multiplicity() {
        let m = parse_member_spec("part wheels:Wheel[4]").unwrap();
        assert_eq!(m.name, "wheels");
        assert_eq!(m.type_ref.as_deref(), Some("Wheel"));
        assert_eq!(m.multiplicity.as_deref(), Some("4"));

        let m2 = parse_member_spec("attribute sensors:Sensor[1..*]").unwrap();
        assert_eq!(m2.name, "sensors");
        assert_eq!(m2.type_ref.as_deref(), Some("Sensor"));
        assert_eq!(m2.multiplicity.as_deref(), Some("1..*"));

        let m3 = parse_member_spec("part items[0..1]").unwrap();
        assert_eq!(m3.name, "items");
        assert!(m3.type_ref.is_none());
        assert_eq!(m3.multiplicity.as_deref(), Some("0..1"));
    }

    #[test]
    fn generate_member_with_multiplicity() {
        let opts = TemplateOptions {
            kind: DefKind::Part,
            name: "Vehicle".to_string(),
            super_type: None,
            is_abstract: false,
            short_name: None,
            doc: None,
            members: vec![MemberSpec {
                usage_kind: "part".to_string(),
                name: "wheels".to_string(),
                type_ref: Some("Wheel".to_string()),
                direction: None,
                multiplicity: Some("4".to_string()),
            }],
            exposes: Vec::new(),
            filter: None,
            indent: 0,
        };
        let result = generate_template(&opts);
        assert!(result.contains("part wheels : Wheel [4];"));
    }

    #[test]
    fn generate_enum_with_members() {
        let opts = TemplateOptions {
            kind: DefKind::Enum,
            name: "Color".to_string(),
            super_type: None,
            is_abstract: false,
            short_name: None,
            doc: None,
            members: vec![
                MemberSpec {
                    usage_kind: "enum".to_string(),
                    name: "red".to_string(),
                    type_ref: None,
                    direction: None,
                    multiplicity: None,
                },
                MemberSpec {
                    usage_kind: "enum".to_string(),
                    name: "green".to_string(),
                    type_ref: None,
                    direction: None,
                    multiplicity: None,
                },
                MemberSpec {
                    usage_kind: "enum".to_string(),
                    name: "blue".to_string(),
                    type_ref: None,
                    direction: None,
                    multiplicity: None,
                },
            ],
            exposes: Vec::new(),
            filter: None,
            indent: 0,
        };
        let result = generate_template(&opts);
        assert!(result.contains("enum def Color {"));
        assert!(result.contains("enum red;"));
        assert!(result.contains("enum green;"));
        assert!(result.contains("enum blue;"));
        // Should NOT contain usage-style formatting
        assert!(!result.contains("enum enum"));
    }

    #[test]
    fn generate_connection_usage_with_type() {
        let result = generate_connection_usage(
            "tempConn",
            Some("SensorConnection"),
            "tempSensor.dataOut to controller.tempIn",
            0,
        );
        assert!(result.contains("connection tempConn : SensorConnection"));
        assert!(result.contains("connect tempSensor.dataOut to controller.tempIn;"));
    }

    #[test]
    fn generate_connection_usage_without_type() {
        let result = generate_connection_usage(
            "displayConn",
            None,
            "controller.displayOut to display.dataIn",
            0,
        );
        assert!(result.contains("connection displayConn\n"));
        assert!(result.contains("connect controller.displayOut to display.dataIn;"));
        assert!(!result.contains(":"));
    }

    #[test]
    fn generate_connection_usage_indented() {
        let result = generate_connection_usage(
            "conn1",
            Some("C"),
            "a.x to b.y",
            8,
        );
        assert!(result.starts_with("        connection conn1 : C\n"));
        assert!(result.contains("            connect a.x to b.y;"));
    }

    #[test]
    fn generate_satisfy_relationship() {
        let result = generate_relationship("satisfy", "TempAccuracy", "WeatherStation", 0);
        assert_eq!(result, "satisfy requirement TempAccuracy by WeatherStation;\n");
    }

    #[test]
    fn generate_verify_relationship() {
        let result = generate_relationship("verify", "TempAccuracy", "TestTempAccuracy", 4);
        assert_eq!(result, "    verify requirement TempAccuracy by TestTempAccuracy;\n");
    }

    #[test]
    fn generate_import_statement() {
        let result = generate_import("WeatherStation::*", 0);
        assert_eq!(result, "import WeatherStation::*;\n");
    }

    #[test]
    fn generate_import_statement_indented() {
        let result = generate_import("Vehicles::Engine", 4);
        assert_eq!(result, "    import Vehicles::Engine;\n");
    }

    #[test]
    fn validate_generated_valid() {
        let valid = "part def Vehicle;\n";
        assert!(validate_generated(valid).is_ok());
    }

    #[test]
    fn validate_generated_with_body() {
        let valid = "part def Vehicle {\n    part engine : Engine;\n}\n";
        assert!(validate_generated(valid).is_ok());
    }

    #[test]
    fn generate_view_def_with_expose_and_filter() {
        let opts = TemplateOptions {
            kind: DefKind::View,
            name: "PartsView".to_string(),
            super_type: None,
            is_abstract: false,
            short_name: None,
            doc: None,
            members: Vec::new(),
            exposes: vec!["Vehicle::*".to_string()],
            filter: Some("PartDef".to_string()),
            indent: 0,
        };
        let result = generate_template(&opts);
        assert!(result.contains("view def PartsView {"));
        assert!(result.contains("expose Vehicle::*;"));
        assert!(result.contains("filter @type istype PartDef;"));
    }
}

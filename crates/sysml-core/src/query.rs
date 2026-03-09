/// Model query functions for filtering and inspecting SysML v2 elements.
///
/// These functions provide the logic behind CLI commands like `list`, `show`,
/// `trace`, and `interfaces`.

use crate::model::*;

// ========================================================================
// Element enumeration — a unified view over definitions and usages
// ========================================================================

/// A unified view of any model element (definition or usage).
#[derive(Debug, Clone)]
pub enum Element<'a> {
    Def(&'a Definition),
    Usage(&'a Usage),
}

impl<'a> Element<'a> {
    pub fn name(&self) -> &str {
        match self {
            Element::Def(d) => &d.name,
            Element::Usage(u) => &u.name,
        }
    }

    pub fn kind_label(&self) -> &str {
        match self {
            Element::Def(d) => d.kind.label(),
            Element::Usage(u) => &u.kind,
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            Element::Def(d) => &d.span,
            Element::Usage(u) => &u.span,
        }
    }

    pub fn parent_def(&self) -> Option<&str> {
        match self {
            Element::Def(d) => d.parent_def.as_deref(),
            Element::Usage(u) => u.parent_def.as_deref(),
        }
    }

    pub fn type_ref(&self) -> Option<&str> {
        match self {
            Element::Def(d) => d.super_type.as_deref(),
            Element::Usage(u) => u.type_ref.as_deref(),
        }
    }

    pub fn short_name(&self) -> Option<&str> {
        match self {
            Element::Def(d) => d.short_name.as_deref(),
            Element::Usage(_) => None,
        }
    }

    pub fn doc(&self) -> Option<&str> {
        match self {
            Element::Def(d) => d.doc.as_deref(),
            Element::Usage(_) => None,
        }
    }
}

// ========================================================================
// Filter types
// ========================================================================

/// Kind filter for the `list` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindFilter {
    /// All definitions.
    Definitions,
    /// All usages.
    Usages,
    /// Specific definition kind.
    DefKind(DefKind),
    /// Specific usage kind string (e.g., "part", "port").
    UsageKind(String),
    /// Everything.
    All,
}

/// Filter criteria for listing model elements.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    /// Filter by element kind.
    pub kind: Option<KindFilter>,
    /// Filter by name pattern (substring match).
    pub name_pattern: Option<String>,
    /// Filter by parent definition name.
    pub parent: Option<String>,
    /// Only show unused definitions.
    pub unused_only: bool,
    /// Only show abstract definitions.
    pub abstract_only: bool,
    /// Filter by visibility.
    pub visibility: Option<Visibility>,
}

/// List model elements matching the given filter.
pub fn list_elements<'a>(model: &'a Model, filter: &ListFilter) -> Vec<Element<'a>> {
    let mut results = Vec::new();

    let include_defs = match &filter.kind {
        None | Some(KindFilter::All) | Some(KindFilter::Definitions) => true,
        Some(KindFilter::DefKind(_)) => true,
        Some(KindFilter::UsageKind(_)) | Some(KindFilter::Usages) => false,
    };

    let include_usages = match &filter.kind {
        None | Some(KindFilter::All) | Some(KindFilter::Usages) => true,
        Some(KindFilter::UsageKind(_)) => true,
        Some(KindFilter::DefKind(_)) | Some(KindFilter::Definitions) => false,
    };

    let referenced = if filter.unused_only {
        Some(model.referenced_names())
    } else {
        None
    };

    if include_defs {
        for def in &model.definitions {
            if let Some(KindFilter::DefKind(k)) = &filter.kind {
                if def.kind != *k {
                    continue;
                }
            }
            if let Some(pat) = &filter.name_pattern {
                if !def.name.contains(pat.as_str()) {
                    continue;
                }
            }
            if let Some(parent) = &filter.parent {
                if def.parent_def.as_deref() != Some(parent.as_str()) {
                    continue;
                }
            }
            if filter.abstract_only && !def.is_abstract {
                continue;
            }
            if let Some(vis) = &filter.visibility {
                if def.visibility.as_ref() != Some(vis) {
                    continue;
                }
            }
            if let Some(ref refs) = referenced {
                if refs.contains(def.name.as_str()) {
                    continue;
                }
            }
            results.push(Element::Def(def));
        }
    }

    if include_usages {
        for usage in &model.usages {
            if let Some(KindFilter::UsageKind(ref k)) = filter.kind {
                if usage.kind != *k {
                    continue;
                }
            }
            if let Some(pat) = &filter.name_pattern {
                if !usage.name.contains(pat.as_str()) {
                    continue;
                }
            }
            if let Some(parent) = &filter.parent {
                if usage.parent_def.as_deref() != Some(parent.as_str()) {
                    continue;
                }
            }
            results.push(Element::Usage(usage));
        }
    }

    results
}

// ========================================================================
// Requirements traceability
// ========================================================================

/// A row in the requirements traceability matrix.
#[derive(Debug, Clone)]
pub struct TraceRow {
    pub requirement: String,
    pub satisfied_by: Vec<String>,
    pub verified_by: Vec<String>,
}

/// Generate a requirements traceability matrix.
pub fn trace_requirements(model: &Model) -> Vec<TraceRow> {
    let req_defs: Vec<&Definition> = model
        .definitions
        .iter()
        .filter(|d| d.kind == DefKind::Requirement)
        .collect();

    let mut rows = Vec::new();
    for req in &req_defs {
        let satisfied_by: Vec<String> = model
            .satisfactions
            .iter()
            .filter(|s| simple_name(&s.requirement) == req.name)
            .map(|s| s.by.clone().unwrap_or_else(|| "(implicit)".to_string()))
            .collect();

        let verified_by: Vec<String> = model
            .verifications
            .iter()
            .filter(|v| simple_name(&v.requirement) == req.name)
            .map(|v| v.by.clone())
            .collect();

        rows.push(TraceRow {
            requirement: req.name.clone(),
            satisfied_by,
            verified_by,
        });
    }
    rows
}

/// Trace coverage statistics.
#[derive(Debug, Clone)]
pub struct TraceCoverage {
    pub total_requirements: usize,
    pub satisfied_count: usize,
    pub verified_count: usize,
    pub fully_traced_count: usize,
}

/// Compute requirements trace coverage.
pub fn trace_coverage(rows: &[TraceRow]) -> TraceCoverage {
    let total = rows.len();
    let satisfied = rows.iter().filter(|r| !r.satisfied_by.is_empty()).count();
    let verified = rows.iter().filter(|r| !r.verified_by.is_empty()).count();
    let fully_traced = rows
        .iter()
        .filter(|r| !r.satisfied_by.is_empty() && !r.verified_by.is_empty())
        .count();

    TraceCoverage {
        total_requirements: total,
        satisfied_count: satisfied,
        verified_count: verified,
        fully_traced_count: fully_traced,
    }
}

// ========================================================================
// Interface / port analysis
// ========================================================================

/// Information about a port within a definition.
#[derive(Debug, Clone)]
pub struct PortInfo {
    pub name: String,
    pub type_ref: Option<String>,
    pub direction: Option<Direction>,
    pub is_conjugated: bool,
    pub owner: String,
}

/// List all ports in the model with their owners.
pub fn list_ports(model: &Model) -> Vec<PortInfo> {
    model
        .usages
        .iter()
        .filter(|u| u.kind == "port")
        .map(|u| PortInfo {
            name: u.name.clone(),
            type_ref: u.type_ref.clone(),
            direction: u.direction,
            is_conjugated: u.is_conjugated,
            owner: u.parent_def.clone().unwrap_or_default(),
        })
        .collect()
}

/// Find ports that are not referenced by any connection.
pub fn unconnected_ports(model: &Model) -> Vec<PortInfo> {
    let connected_names: std::collections::HashSet<&str> = model
        .connections
        .iter()
        .flat_map(|c| vec![simple_name(&c.source), simple_name(&c.target)])
        .collect();

    list_ports(model)
        .into_iter()
        .filter(|p| !connected_names.contains(p.name.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_file;

    #[test]
    fn list_all_definitions() {
        let model = parse_file(
            "test.sysml",
            r#"
            part def Vehicle;
            part def Engine;
            port def DataPort;
        "#,
        );
        let results = list_elements(&model, &ListFilter::default());
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn list_filter_by_kind() {
        let model = parse_file(
            "test.sysml",
            r#"
            part def Vehicle;
            port def DataPort;
            part def Engine;
        "#,
        );
        let filter = ListFilter {
            kind: Some(KindFilter::DefKind(DefKind::Part)),
            ..Default::default()
        };
        let results = list_elements(&model, &filter);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.kind_label() == "part def"));
    }

    #[test]
    fn list_filter_by_name() {
        let model = parse_file(
            "test.sysml",
            r#"
            part def Vehicle;
            part def VehicleConfig;
            part def Engine;
        "#,
        );
        let filter = ListFilter {
            name_pattern: Some("Vehicle".to_string()),
            ..Default::default()
        };
        let results = list_elements(&model, &filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn list_filter_by_parent() {
        let model = parse_file(
            "test.sysml",
            r#"
            part def Vehicle {
                part engine : Engine;
                part wheels : Wheel;
            }
            part def Standalone;
        "#,
        );
        let filter = ListFilter {
            kind: Some(KindFilter::Usages),
            parent: Some("Vehicle".to_string()),
            ..Default::default()
        };
        let results = list_elements(&model, &filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn list_unused_only() {
        let model = parse_file(
            "test.sysml",
            r#"
            part def Engine;
            part def Vehicle {
                part engine : Engine;
            }
            part def Orphan;
        "#,
        );
        let filter = ListFilter {
            unused_only: true,
            ..Default::default()
        };
        let results = list_elements(&model, &filter);
        let names: Vec<&str> = results.iter().map(|e| e.name()).collect();
        // Vehicle is used (Engine types to it... actually Engine is used by Vehicle, so Orphan + Vehicle are unused)
        assert!(
            names.contains(&"Orphan"),
            "Orphan should be unused, got: {:?}",
            names
        );
        assert!(
            !names.contains(&"Engine"),
            "Engine should not be unused, got: {:?}",
            names
        );
    }

    #[test]
    fn list_abstract_only() {
        let model = parse_file(
            "test.sysml",
            r#"
            abstract part def Base;
            part def Concrete;
        "#,
        );
        let filter = ListFilter {
            abstract_only: true,
            ..Default::default()
        };
        let results = list_elements(&model, &filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name(), "Base");
    }

    #[test]
    fn trace_requirements_empty() {
        let model = parse_file("test.sysml", "part def Vehicle;");
        let rows = trace_requirements(&model);
        assert!(rows.is_empty());
    }

    #[test]
    fn trace_requirements_satisfied() {
        let model = parse_file(
            "test.sysml",
            r#"
            requirement def MassReq {
                doc /* mass < 2000 */
            }
            part def Vehicle {
                satisfy MassReq;
            }
        "#,
        );
        let rows = trace_requirements(&model);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].requirement, "MassReq");
        assert!(!rows[0].satisfied_by.is_empty());
    }

    #[test]
    fn trace_coverage_metrics() {
        let rows = vec![
            TraceRow {
                requirement: "R1".into(),
                satisfied_by: vec!["A".into()],
                verified_by: vec!["T1".into()],
            },
            TraceRow {
                requirement: "R2".into(),
                satisfied_by: vec!["B".into()],
                verified_by: vec![],
            },
            TraceRow {
                requirement: "R3".into(),
                satisfied_by: vec![],
                verified_by: vec![],
            },
        ];
        let cov = trace_coverage(&rows);
        assert_eq!(cov.total_requirements, 3);
        assert_eq!(cov.satisfied_count, 2);
        assert_eq!(cov.verified_count, 1);
        assert_eq!(cov.fully_traced_count, 1);
    }

    #[test]
    fn unconnected_ports_found() {
        let model = parse_file(
            "test.sysml",
            r#"
            port def FuelPort;
            part def Vehicle {
                port fuelIn : FuelPort;
                port dataOut : DataPort;
            }
        "#,
        );
        let unconn = unconnected_ports(&model);
        assert_eq!(unconn.len(), 2, "Both ports are unconnected");
    }

    #[test]
    fn list_ports_with_owner() {
        let model = parse_file(
            "test.sysml",
            r#"
            part def Vehicle {
                port fuelIn : FuelPort;
            }
            part def Station {
                port fuelOut : FuelPort;
            }
        "#,
        );
        let ports = list_ports(&model);
        assert_eq!(ports.len(), 2);
        assert!(ports.iter().any(|p| p.name == "fuelIn" && p.owner == "Vehicle"));
        assert!(ports.iter().any(|p| p.name == "fuelOut" && p.owner == "Station"));
    }
}

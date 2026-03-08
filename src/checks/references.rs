/// Checks for unused definitions and unresolved type references.

use crate::checks::Check;
use crate::diagnostic::{codes, Diagnostic};
use crate::model::{simple_name, DefKind, Model};

/// Standard library types that are always available (not defined in user files).
const BUILTIN_TYPES: &[&str] = &[
    // ScalarValues
    "Boolean", "String", "Integer", "Real", "Natural", "Complex", "Rational",
    "ScalarValues",
    // ISQ common quantities
    "MassValue", "LengthValue", "TimeValue", "PowerValue", "ForceValue",
    "PressureValue", "TemperatureValue", "VelocityValue", "AccelerationValue",
    "AreaValue", "VolumeValue", "DensityValue", "EnergyValue", "TorqueValue",
    "AngularVelocityValue", "FrequencyValue", "ElectricCurrentValue",
    "VoltageValue", "ResistanceValue", "CapacitanceValue",
    "ISQ",
    // SI units
    "SI", "kg", "m", "s", "A", "K", "mol", "cd", "N", "Pa", "J", "W", "Hz",
    // Common base types
    "Anything", "Nothing", "Object", "Occurrence", "Item", "Part",
    "Connection", "Interface", "Port", "Flow", "Action", "State",
    "Constraint", "Requirement", "Calculation", "Analysis",
    "Verification", "UseCase", "View", "Viewpoint", "Rendering",
    "Attribute", "Enum", "Package", "Feature", "Classifier", "Type",
    // Trade studies
    "TradeStudy", "MaximizeObjective", "MinimizeObjective",
    // Verification
    "VerdictKind", "PassFail",
    // Sampling
    "SampledFunction",
];

pub struct UnusedDefCheck;

impl Check for UnusedDefCheck {
    fn name(&self) -> &'static str {
        "unused"
    }

    fn run(&self, model: &Model) -> Vec<Diagnostic> {
        let referenced = model.referenced_names();
        let mut diagnostics = Vec::new();

        for def in &model.definitions {
            // Packages are structural, not referenced by name
            if def.kind == DefKind::Package {
                continue;
            }

            let name = def.name.as_str();
            if !referenced.contains(name) {
                diagnostics.push(Diagnostic::note(
                    &model.file,
                    def.span.clone(),
                    codes::UNUSED_DEF,
                    format!(
                        "{} `{}` is defined but never referenced",
                        def.kind.label(),
                        def.name,
                    ),
                ));
            }
        }

        diagnostics
    }
}

pub struct UnresolvedTypeCheck;

impl Check for UnresolvedTypeCheck {
    fn name(&self) -> &'static str {
        "unresolved"
    }

    fn run(&self, model: &Model) -> Vec<Diagnostic> {
        let defined = model.defined_names();

        // Also collect usage names (parts, ports, etc. are valid targets)
        let mut known: std::collections::HashSet<&str> = defined;
        for u in &model.usages {
            known.insert(u.name.as_str());
        }
        // Add builtins
        for &b in BUILTIN_TYPES {
            known.insert(b);
        }

        let mut diagnostics = Vec::new();

        // Check type references
        for tr in &model.type_references {
            let name = simple_name(&tr.name);
            if !known.contains(name) && !is_qualified_stdlib(&tr.name) {
                diagnostics.push(Diagnostic::warning(
                    &model.file,
                    tr.span.clone(),
                    codes::UNRESOLVED_TYPE,
                    format!("type `{}` is not defined in this file", tr.name),
                ));
            }
        }

        // Check connection targets
        for conn in &model.connections {
            check_ref_exists(&conn.source, &conn.span, &known, &model.file, &mut diagnostics);
            check_ref_exists(&conn.target, &conn.span, &known, &model.file, &mut diagnostics);
        }

        // Check allocation targets
        for alloc in &model.allocations {
            check_ref_exists(
                &alloc.source,
                &alloc.span,
                &known,
                &model.file,
                &mut diagnostics,
            );
            check_ref_exists(
                &alloc.target,
                &alloc.span,
                &known,
                &model.file,
                &mut diagnostics,
            );
        }

        diagnostics
    }
}

fn check_ref_exists(
    name: &str,
    span: &crate::model::Span,
    known: &std::collections::HashSet<&str>,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let simple = simple_name(name);
    if !known.contains(simple) && !is_qualified_stdlib(name) {
        diagnostics.push(Diagnostic::warning(
            file,
            span.clone(),
            codes::UNRESOLVED_TARGET,
            format!("reference `{}` does not resolve to any definition or usage", name),
        ));
    }
}

fn is_qualified_stdlib(name: &str) -> bool {
    name.starts_with("ISQ::")
        || name.starts_with("SI::")
        || name.starts_with("ScalarValues::")
        || name.starts_with("Quantities::")
        || name.starts_with("MeasurementReferences::")
        || name.starts_with("SampledFunctions::")
        || name.starts_with("VerificationCases::")
        || name.starts_with("TradeStudies::")
        || name.starts_with("BaseFunctions::")
        || name.starts_with("ControlFunctions::")
        || name.starts_with("DataFunctions::")
        || name.starts_with("NumericalFunctions::")
        || name.starts_with("SequenceFunctions::")
        || name.starts_with("TrigFunctions::")
}

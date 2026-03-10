/// Corrective and Preventive Action (CAPA) — formal action programs.
///
/// A CAPA is distinct from an NCR: the NCR documents what went wrong,
/// while the CAPA defines and tracks the corrective/preventive actions
/// taken to address root causes and prevent recurrence.
///
/// CAPAs may originate from NCRs, audit findings, customer complaints,
/// or proactive process improvement initiatives.

use std::collections::BTreeMap;
use serde::Serialize;
use sysml_core::record::{generate_record_id, now_iso8601, RecordEnvelope, RecordMeta, RecordValue};

use crate::enums::{CapaStatus, CorrectiveActionType};

/// Source that triggered a CAPA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapaSource {
    /// Triggered by one or more NCRs.
    Ncr,
    /// Triggered by an audit finding.
    AuditFinding,
    /// Triggered by a customer complaint.
    CustomerComplaint,
    /// Proactive process improvement.
    ProcessImprovement,
    /// Triggered by a regulatory observation.
    RegulatoryObservation,
    /// Triggered by management review.
    ManagementReview,
}

impl CapaSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ncr => "NCR",
            Self::AuditFinding => "Audit Finding",
            Self::CustomerComplaint => "Customer Complaint",
            Self::ProcessImprovement => "Process Improvement",
            Self::RegulatoryObservation => "Regulatory Observation",
            Self::ManagementReview => "Management Review",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Ncr, Self::AuditFinding, Self::CustomerComplaint,
            Self::ProcessImprovement, Self::RegulatoryObservation,
            Self::ManagementReview,
        ]
    }
}

/// Whether the CAPA is corrective (fix existing problem) or
/// preventive (prevent potential problem).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapaType {
    Corrective,
    Preventive,
}

impl CapaType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Corrective => "Corrective",
            Self::Preventive => "Preventive",
        }
    }
}

/// A corrective or preventive action program.
#[derive(Debug, Clone, Serialize)]
pub struct Capa {
    pub id: String,
    pub title: String,
    pub description: String,
    pub capa_type: CapaType,
    pub source: CapaSource,
    /// IDs of source items (NCR IDs, audit finding refs, etc.).
    pub source_refs: Vec<String>,
    pub root_cause: Option<String>,
    pub actions: Vec<CapaAction>,
    pub status: CapaStatus,
    pub owner: String,
    pub created: String,
}

/// An individual action within a CAPA program.
#[derive(Debug, Clone, Serialize)]
pub struct CapaAction {
    pub id: String,
    pub action_type: CorrectiveActionType,
    pub description: String,
    pub owner: String,
    pub due_date: String,
    pub completed: bool,
    pub verification_ref: Option<String>,
}

/// Create a new CAPA with generated ID and `Initiated` status.
pub fn create_capa(
    title: &str,
    description: &str,
    capa_type: CapaType,
    source: CapaSource,
    source_refs: Vec<String>,
    owner: &str,
) -> Capa {
    let id = generate_record_id("quality", "capa", owner);
    Capa {
        id,
        title: title.to_string(),
        description: description.to_string(),
        capa_type,
        source,
        source_refs,
        root_cause: None,
        actions: Vec::new(),
        status: CapaStatus::Initiated,
        owner: owner.to_string(),
        created: now_iso8601(),
    }
}

/// Add an action to a CAPA.
pub fn add_action(capa: &mut Capa, action: CapaAction) {
    capa.actions.push(action);
    if capa.status == CapaStatus::PlanningActions || capa.status == CapaStatus::RootCauseAnalysis {
        capa.status = CapaStatus::Implementing;
    }
}

/// Set the root cause on a CAPA and advance to `PlanningActions`.
pub fn set_root_cause(capa: &mut Capa, root_cause: &str) {
    capa.root_cause = Some(root_cause.to_string());
    if capa.status == CapaStatus::RootCauseAnalysis || capa.status == CapaStatus::Initiated {
        capa.status = CapaStatus::PlanningActions;
    }
}

/// Build wizard steps for adding a corrective/preventive action to a CAPA.
pub fn build_action_wizard_steps() -> Vec<sysml_core::interactive::WizardStep> {
    use sysml_core::interactive::WizardStep;

    vec![
        WizardStep::choice("action_type", "Action type", vec![
            ("design_change", "Design Change"),
            ("process_change", "Process Change"),
            ("supplier_change", "Supplier Change"),
            ("tooling_change", "Tooling Change"),
            ("training", "Training/Retraining"),
            ("procedure_update", "Procedure Update"),
            ("inspection", "Inspection Enhancement"),
            ("containment", "Containment (immediate)"),
            ("no_action", "No Action Required"),
        ]).with_explanation(
            "Select the type of action that addresses the root cause. \
             Containment actions are immediate; others are systemic fixes."
        ),
        WizardStep::string("description", "Action description")
            .with_explanation("What specifically will be done?"),
        WizardStep::string("owner", "Action owner (responsible person)")
            .with_default("engineer"),
        WizardStep::string("due_date", "Due date (YYYY-MM-DD)")
            .with_explanation("When must this action be completed?"),
        WizardStep::string("verification_ref", "Verification reference (Enter to skip)")
            .with_explanation(
                "If this action needs to be verified, enter the verification case \
                 or test plan reference."
            )
            .optional(),
    ]
}

/// Interpret a completed action wizard result into a [`CapaAction`].
pub fn interpret_action_result(
    result: &sysml_core::interactive::WizardResult,
    action_id: &str,
) -> CapaAction {
    let action_type = match result.get_string("action_type").unwrap_or("procedure_update") {
        "design_change" => CorrectiveActionType::DesignChange,
        "process_change" => CorrectiveActionType::ProcessChange,
        "supplier_change" => CorrectiveActionType::SupplierChange,
        "tooling_change" => CorrectiveActionType::ToolingChange,
        "training" => CorrectiveActionType::TrainingRetraining,
        "procedure_update" => CorrectiveActionType::ProcedureUpdate,
        "inspection" => CorrectiveActionType::InspectionEnhancement,
        "containment" => CorrectiveActionType::Containment,
        "no_action" => CorrectiveActionType::NoActionRequired,
        _ => CorrectiveActionType::ProcedureUpdate,
    };

    let description = result.get_string("description")
        .unwrap_or("")
        .to_string();
    let owner = result.get_string("owner")
        .unwrap_or("engineer")
        .to_string();
    let due_date = result.get_string("due_date")
        .unwrap_or("")
        .to_string();
    let verification_ref = result.get_string("verification_ref")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    CapaAction {
        id: action_id.to_string(),
        action_type,
        description,
        owner,
        due_date,
        completed: false,
        verification_ref,
    }
}

/// Create a [`RecordEnvelope`] for a CAPA.
pub fn create_capa_record(capa: &Capa, author: &str) -> RecordEnvelope {
    let id = generate_record_id("quality", "capa", author);

    let mut refs = BTreeMap::new();
    refs.insert("capa".to_string(), vec![capa.id.clone()]);
    if !capa.source_refs.is_empty() {
        refs.insert("source".to_string(), capa.source_refs.clone());
    }

    let mut data = BTreeMap::new();
    data.insert("title".into(), RecordValue::String(capa.title.clone()));
    data.insert("description".into(), RecordValue::String(capa.description.clone()));
    data.insert("capa_type".into(), RecordValue::String(capa.capa_type.label().to_string()));
    data.insert("source".into(), RecordValue::String(capa.source.label().to_string()));
    data.insert("status".into(), RecordValue::String(capa.status.label().to_string()));
    data.insert("owner".into(), RecordValue::String(capa.owner.clone()));
    if let Some(rc) = &capa.root_cause {
        data.insert("root_cause".into(), RecordValue::String(rc.clone()));
    }
    data.insert(
        "action_count".into(),
        RecordValue::String(capa.actions.len().to_string()),
    );

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "quality".into(),
            record_type: "capa".into(),
            created: now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::CorrectiveActionType;

    #[test]
    fn create_capa_generates_id_and_initiated_status() {
        let capa = create_capa(
            "Fix brake rotor tolerance",
            "Address recurring dimensional failures in brake rotors",
            CapaType::Corrective,
            CapaSource::Ncr,
            vec!["NCR-001".into()],
            "alice",
        );
        assert!(capa.id.starts_with("quality-capa-"));
        assert_eq!(capa.status, CapaStatus::Initiated);
        assert!(capa.root_cause.is_none());
        assert!(capa.actions.is_empty());
    }

    #[test]
    fn set_root_cause_advances_status() {
        let mut capa = create_capa(
            "Fix issue", "desc",
            CapaType::Corrective, CapaSource::Ncr, vec![], "bob",
        );
        set_root_cause(&mut capa, "Missing SOP for tool offset checks");
        assert_eq!(capa.root_cause.as_deref(), Some("Missing SOP for tool offset checks"));
        assert_eq!(capa.status, CapaStatus::PlanningActions);
    }

    #[test]
    fn add_action_advances_status() {
        let mut capa = create_capa(
            "Fix issue", "desc",
            CapaType::Corrective, CapaSource::Ncr, vec![], "bob",
        );
        set_root_cause(&mut capa, "Root cause");

        let action = CapaAction {
            id: "CA-001".into(),
            action_type: CorrectiveActionType::ProcedureUpdate,
            description: "Update turning SOP".into(),
            owner: "bob".into(),
            due_date: "2026-04-01".into(),
            completed: false,
            verification_ref: None,
        };
        add_action(&mut capa, action);
        assert_eq!(capa.actions.len(), 1);
        assert_eq!(capa.status, CapaStatus::Implementing);
    }

    #[test]
    fn capa_source_labels() {
        assert_eq!(CapaSource::Ncr.label(), "NCR");
        assert_eq!(CapaSource::AuditFinding.label(), "Audit Finding");
        assert_eq!(CapaSource::CustomerComplaint.label(), "Customer Complaint");
    }

    #[test]
    fn capa_type_labels() {
        assert_eq!(CapaType::Corrective.label(), "Corrective");
        assert_eq!(CapaType::Preventive.label(), "Preventive");
    }

    #[test]
    fn capa_record_structure() {
        let capa = create_capa(
            "Fix rotor", "Fix dimensional issue",
            CapaType::Corrective, CapaSource::Ncr,
            vec!["NCR-001".into()], "alice",
        );
        let rec = create_capa_record(&capa, "alice");
        assert_eq!(rec.meta.tool, "quality");
        assert_eq!(rec.meta.record_type, "capa");
        assert!(rec.refs.contains_key("capa"));
        assert!(rec.refs.contains_key("source"));
        assert_eq!(
            rec.data.get("capa_type"),
            Some(&RecordValue::String("Corrective".into()))
        );
        assert_eq!(
            rec.data.get("source"),
            Some(&RecordValue::String("NCR".into()))
        );
    }

    #[test]
    fn capa_record_round_trips_toml() {
        let capa = create_capa(
            "Fix rotor", "Fix dimensional issue",
            CapaType::Corrective, CapaSource::Ncr, vec![], "alice",
        );
        let rec = create_capa_record(&capa, "alice");
        let toml = rec.to_toml_string();
        let parsed = RecordEnvelope::from_toml_str(&toml).unwrap();
        assert_eq!(parsed.meta.tool, "quality");
        assert_eq!(parsed.meta.record_type, "capa");
    }

    #[test]
    fn capa_serializes() {
        let capa = create_capa(
            "Test CAPA", "description",
            CapaType::Preventive, CapaSource::ProcessImprovement, vec![], "dave",
        );
        let json = serde_json::to_string(&capa).unwrap();
        assert!(json.contains("\"preventive\""));
        assert!(json.contains("\"process_improvement\""));
    }

    #[test]
    fn action_wizard_step_count() {
        let steps = build_action_wizard_steps();
        assert_eq!(steps.len(), 5);
    }

    #[test]
    fn action_wizard_step_ids() {
        let steps = build_action_wizard_steps();
        let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["action_type", "description", "owner", "due_date", "verification_ref"]);
    }

    #[test]
    fn action_wizard_verification_ref_optional() {
        let steps = build_action_wizard_steps();
        let vr = steps.iter().find(|s| s.id == "verification_ref").unwrap();
        assert!(!vr.required);
    }

    #[test]
    fn interpret_action_result_basic() {
        use sysml_core::interactive::{WizardResult, WizardAnswer};

        let mut result = WizardResult::new();
        result.set("action_type", WizardAnswer::String("procedure_update".into()));
        result.set("description", WizardAnswer::String("Update turning SOP".into()));
        result.set("owner", WizardAnswer::String("bob".into()));
        result.set("due_date", WizardAnswer::String("2026-04-01".into()));
        result.set("verification_ref", WizardAnswer::Skipped);

        let action = interpret_action_result(&result, "CA-001");
        assert_eq!(action.id, "CA-001");
        assert_eq!(action.action_type, CorrectiveActionType::ProcedureUpdate);
        assert_eq!(action.description, "Update turning SOP");
        assert_eq!(action.owner, "bob");
        assert_eq!(action.due_date, "2026-04-01");
        assert!(!action.completed);
        assert!(action.verification_ref.is_none());
    }

    #[test]
    fn interpret_action_result_with_verification() {
        use sysml_core::interactive::{WizardResult, WizardAnswer};

        let mut result = WizardResult::new();
        result.set("action_type", WizardAnswer::String("design_change".into()));
        result.set("description", WizardAnswer::String("Redesign bracket".into()));
        result.set("owner", WizardAnswer::String("alice".into()));
        result.set("due_date", WizardAnswer::String("2026-05-15".into()));
        result.set("verification_ref", WizardAnswer::String("VC-042".into()));

        let action = interpret_action_result(&result, "CA-002");
        assert_eq!(action.action_type, CorrectiveActionType::DesignChange);
        assert_eq!(action.verification_ref.as_deref(), Some("VC-042"));
    }

    #[test]
    fn interpret_action_all_types() {
        use sysml_core::interactive::{WizardResult, WizardAnswer};

        let type_map = vec![
            ("design_change", CorrectiveActionType::DesignChange),
            ("process_change", CorrectiveActionType::ProcessChange),
            ("supplier_change", CorrectiveActionType::SupplierChange),
            ("tooling_change", CorrectiveActionType::ToolingChange),
            ("training", CorrectiveActionType::TrainingRetraining),
            ("inspection", CorrectiveActionType::InspectionEnhancement),
            ("containment", CorrectiveActionType::Containment),
            ("no_action", CorrectiveActionType::NoActionRequired),
        ];

        for (input, expected) in type_map {
            let mut result = WizardResult::new();
            result.set("action_type", WizardAnswer::String(input.into()));
            result.set("description", WizardAnswer::String("desc".into()));
            result.set("owner", WizardAnswer::String("x".into()));
            result.set("due_date", WizardAnswer::String("2026-01-01".into()));

            let action = interpret_action_result(&result, "test");
            assert_eq!(action.action_type, expected, "failed for input '{input}'");
        }
    }
}

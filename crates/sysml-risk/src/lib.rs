/// Risk management domain for SysML v2 models.
///
/// Implements hazard analysis (MIL-STD-882E / ISO 14971) and FMEA
/// (AIAG/VDA, SAE J1739, IEC 60812) methodology.  Integrates with
/// `sysml-core`'s record envelope system for audit-trail persistence
/// and the interactive wizard framework for guided risk entry.
///
/// ## Scoring
///
/// All three scoring dimensions use a 1–5 integer scale:
///
/// | Score | Severity (S)     | Occurrence (O)   | Detection (D)          |
/// |-------|------------------|------------------|------------------------|
/// | 1     | Negligible       | Improbable       | Almost Certain         |
/// | 2     | Marginal         | Remote           | High                   |
/// | 3     | Moderate         | Occasional       | Moderate               |
/// | 4     | Critical         | Probable         | Low                    |
/// | 5     | Catastrophic     | Frequent         | Almost Impossible      |
///
/// RPN = S × O × D  (range 1–125).
///
/// ## Risk Acceptance (ISO 14971 / MIL-STD-882E)
///
/// The 5×5 severity–occurrence matrix is partitioned into four zones:
///
/// - **Unacceptable**: S×O ≥ 15, or S=5 with O≥3, or O=5 with S≥3
/// - **Undesirable**: S×O ≥ 8 (not already unacceptable)
/// - **Acceptable with review**: S×O ≥ 4 (not already higher)
/// - **Broadly acceptable**: S×O < 4

use std::collections::BTreeMap;

use serde::Serialize;
use sysml_core::interactive::{PromptKind, WizardStep};
use sysml_core::model::{DefKind, Model};
use sysml_core::record::{generate_record_id, RecordEnvelope, RecordMeta, RecordValue};

// =========================================================================
// Score label lookups
// =========================================================================

/// Human-readable label for a severity score (1–5).
pub fn severity_label(score: u32) -> &'static str {
    match score {
        1 => "Negligible",
        2 => "Marginal",
        3 => "Moderate",
        4 => "Critical",
        5 => "Catastrophic",
        _ => "?",
    }
}

/// Human-readable label for an occurrence score (1–5).
pub fn occurrence_label(score: u32) -> &'static str {
    match score {
        1 => "Improbable",
        2 => "Remote",
        3 => "Occasional",
        4 => "Probable",
        5 => "Frequent",
        _ => "?",
    }
}

/// Human-readable label for a detection score (1–5).
pub fn detection_label(score: u32) -> &'static str {
    match score {
        1 => "Almost Certain",
        2 => "High",
        3 => "Moderate",
        4 => "Low",
        5 => "Almost Impossible",
        _ => "?",
    }
}

// =========================================================================
// Enums (domain categories, statuses, strategies)
// =========================================================================

/// Domain category for a risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Technical,
    Schedule,
    Cost,
    Safety,
    Regulatory,
    SupplyChain,
    Environmental,
}

impl RiskCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Technical => "Technical",
            Self::Schedule => "Schedule",
            Self::Cost => "Cost",
            Self::Safety => "Safety",
            Self::Regulatory => "Regulatory",
            Self::SupplyChain => "Supply Chain",
            Self::Environmental => "Environmental",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Technical,
            Self::Schedule,
            Self::Cost,
            Self::Safety,
            Self::Regulatory,
            Self::SupplyChain,
            Self::Environmental,
        ]
    }

    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "technical" => Some(Self::Technical),
            "schedule" => Some(Self::Schedule),
            "cost" => Some(Self::Cost),
            "safety" => Some(Self::Safety),
            "regulatory" => Some(Self::Regulatory),
            "supply_chain" => Some(Self::SupplyChain),
            "environmental" => Some(Self::Environmental),
            _ => None,
        }
    }

    fn id(&self) -> &'static str {
        match self {
            Self::Technical => "technical",
            Self::Schedule => "schedule",
            Self::Cost => "cost",
            Self::Safety => "safety",
            Self::Regulatory => "regulatory",
            Self::SupplyChain => "supply_chain",
            Self::Environmental => "environmental",
        }
    }
}

/// Lifecycle status of a risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskStatus {
    Identified,
    Analyzing,
    Mitigating,
    Monitoring,
    Closed,
    Accepted,
}

impl RiskStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Identified => "Identified",
            Self::Analyzing => "Analyzing",
            Self::Mitigating => "Mitigating",
            Self::Monitoring => "Monitoring",
            Self::Closed => "Closed",
            Self::Accepted => "Accepted",
        }
    }

    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "identified" => Some(Self::Identified),
            "analyzing" => Some(Self::Analyzing),
            "mitigating" => Some(Self::Mitigating),
            "monitoring" => Some(Self::Monitoring),
            "closed" => Some(Self::Closed),
            "accepted" => Some(Self::Accepted),
            _ => None,
        }
    }
}

/// Strategy for addressing a risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MitigationStrategy {
    Avoid,
    Transfer,
    Reduce,
    Accept,
    Contingency,
}

impl MitigationStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Avoid => "Avoid",
            Self::Transfer => "Transfer",
            Self::Reduce => "Reduce",
            Self::Accept => "Accept",
            Self::Contingency => "Contingency",
        }
    }

    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "avoid" => Some(Self::Avoid),
            "transfer" => Some(Self::Transfer),
            "reduce" => Some(Self::Reduce),
            "accept" => Some(Self::Accept),
            "contingency" => Some(Self::Contingency),
            _ => None,
        }
    }
}

/// Risk acceptance level per ISO 14971 / MIL-STD-882E matrix zones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskAcceptance {
    /// S×O ≥ 15, or high-severity/high-occurrence corner
    Unacceptable,
    /// S×O ≥ 8 (not already unacceptable)
    Undesirable,
    /// S×O ≥ 4 (not already higher)
    AcceptableWithReview,
    /// S×O < 4
    BroadlyAcceptable,
}

impl RiskAcceptance {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unacceptable => "UNACCEPTABLE",
            Self::Undesirable => "UNDESIRABLE",
            Self::AcceptableWithReview => "Review",
            Self::BroadlyAcceptable => "Acceptable",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            Self::Unacceptable => "!!",
            Self::Undesirable => "! ",
            Self::AcceptableWithReview => "? ",
            Self::BroadlyAcceptable => "  ",
        }
    }
}

/// Determine risk acceptance from severity and occurrence scores.
pub fn risk_acceptance(severity: u32, occurrence: u32) -> RiskAcceptance {
    let product = severity * occurrence;
    // Unacceptable: product ≥ 15, or catastrophic+occasional+, or frequent+moderate+
    if product >= 15 || (severity == 5 && occurrence >= 3) || (occurrence == 5 && severity >= 3) {
        RiskAcceptance::Unacceptable
    } else if product >= 8 {
        RiskAcceptance::Undesirable
    } else if product >= 4 {
        RiskAcceptance::AcceptableWithReview
    } else {
        RiskAcceptance::BroadlyAcceptable
    }
}

// =========================================================================
// Domain structs
// =========================================================================

/// A risk/hazard definition extracted from or created for a SysML model.
///
/// Follows FMEA structure (AIAG/VDA, SAE J1739) and hazard analysis
/// (MIL-STD-882E, ISO 14971).  All scores are 1–5 integers.
#[derive(Debug, Clone, Serialize)]
pub struct RiskDef {
    /// Unique identifier (from SysML definition name).
    pub id: String,
    /// Item or function this risk applies to.
    pub item: String,
    /// Specific failure mode.
    pub failure_mode: String,
    /// Potential effect(s) of the failure.
    pub failure_effect: String,
    /// Root cause or mechanism of the failure.
    pub failure_cause: String,
    /// Current prevention controls.
    pub current_controls: String,
    /// Risk domain category.
    pub category: Option<RiskCategory>,
    /// Lifecycle status.
    pub status: Option<RiskStatus>,
    /// Severity score (1–5).
    pub severity: Option<u32>,
    /// Occurrence score (1–5).  FMEA "O" / MIL-STD-882E probability.
    pub occurrence: Option<u32>,
    /// Detection score (1–5).  Higher = harder to detect (FMEA convention).
    pub detection: Option<u32>,
    /// Risk Priority Number (S × O × D); `None` if any factor is missing.
    pub rpn: Option<u32>,
    /// Risk acceptance level derived from S×O matrix position.
    pub acceptance: Option<RiskAcceptance>,
    /// Person or team responsible.
    pub owner: Option<String>,
    /// Additional notes, context, or references.
    pub notes: Option<String>,
    /// Recommended action to reduce risk.
    pub recommended_action: Option<String>,
    /// Parent definition this risk is nested inside (the entity it's assigned to).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
}

impl RiskDef {
    /// Create a minimal RiskDef with only an ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            item: String::new(),
            failure_mode: String::new(),
            failure_effect: String::new(),
            failure_cause: String::new(),
            current_controls: String::new(),
            category: None,
            status: None,
            severity: None,
            occurrence: None,
            detection: None,
            rpn: None,
            acceptance: None,
            owner: None,
            notes: None,
            recommended_action: None,
            assigned_to: None,
        }
    }

    /// Recompute RPN and acceptance from current scores.
    pub fn recompute(&mut self) {
        self.rpn = match (self.severity, self.occurrence, self.detection) {
            (Some(s), Some(o), Some(d)) => Some(s * o * d),
            _ => None,
        };
        self.acceptance = match (self.severity, self.occurrence) {
            (Some(s), Some(o)) => Some(risk_acceptance(s, o)),
            _ => None,
        };
    }
}

/// A point-in-time assessment of a risk (append-only history).
#[derive(Debug, Clone, Serialize)]
pub struct RiskAssessment {
    pub risk_id: String,
    pub severity: u32,
    pub occurrence: u32,
    pub detection: u32,
    pub rpn: u32,
    pub notes: Option<String>,
    pub assessed_by: String,
}

/// A mitigation/recommended action linked to a risk.
#[derive(Debug, Clone, Serialize)]
pub struct Mitigation {
    pub id: String,
    pub risk_id: String,
    pub description: String,
    pub strategy: MitigationStrategy,
    pub owner: Option<String>,
    pub due_date: Option<String>,
    pub status: Option<RiskStatus>,
}

// =========================================================================
// RPN computation
// =========================================================================

/// Compute RPN: Severity × Occurrence × Detection.  Range: 1–125.
pub fn compute_rpn(severity: u32, occurrence: u32, detection: u32) -> u32 {
    severity * occurrence * detection
}

// =========================================================================
// Model extraction
// =========================================================================

/// Scan a parsed SysML model for part definitions that represent risks.
///
/// A definition is considered a risk if it specializes `RiskDef` (via
/// `super_type`) or its name contains the substring `"Risk"`.
///
/// Scores are extracted from attribute usages named `severity`,
/// `occurrence` (or `likelihood` for backward compatibility), and
/// `detection` (or `detectability`), where the `value_expr` is a 1–5
/// integer.  FMEA fields (`failureMode`, `failureEffect`, `failureCause`,
/// `currentControls`, `recommendedAction`) are extracted from string
/// attribute usages.
pub fn extract_risks(model: &Model) -> Vec<RiskDef> {
    let mut risks = Vec::new();

    for def in &model.definitions {
        if def.kind != DefKind::Part {
            continue;
        }

        let is_risk_specialization = def
            .super_type
            .as_deref()
            .map(|s| s == "RiskDef" || s.ends_with("::RiskDef"))
            .unwrap_or(false);

        let name_contains_risk = def.name.contains("Risk")
            || def.name.contains("risk")
            || def.name.contains("Hazard")
            || def.name.contains("hazard");

        if !is_risk_specialization && !name_contains_risk {
            continue;
        }

        let usages = model.usages_in_def(&def.name);

        let severity = extract_level(&usages, "severity");
        // Accept both "occurrence" (FMEA) and "likelihood" (legacy/MIL-STD)
        let occurrence = extract_level(&usages, "occurrence")
            .or_else(|| extract_level(&usages, "likelihood"));
        // Accept both "detection" (FMEA) and "detectability" (legacy)
        let detection = extract_level(&usages, "detection")
            .or_else(|| extract_level(&usages, "detectability"));

        let rpn = match (severity, occurrence, detection) {
            (Some(s), Some(o), Some(d)) => Some(compute_rpn(s, o, d)),
            _ => None,
        };
        let acceptance = match (severity, occurrence) {
            (Some(s), Some(o)) => Some(risk_acceptance(s, o)),
            _ => None,
        };

        let owner = extract_string_attr(&usages, "owner");
        let notes = extract_string_attr(&usages, "notes");
        let category_str = extract_string_attr(&usages, "category");
        let status_str = extract_string_attr(&usages, "status");

        let failure_mode = extract_string_attr(&usages, "failureMode")
            .unwrap_or_default();
        let failure_effect = extract_string_attr(&usages, "failureEffect")
            .unwrap_or_default();
        let failure_cause = extract_string_attr(&usages, "failureCause")
            .unwrap_or_default();
        let current_controls = extract_string_attr(&usages, "currentControls")
            .unwrap_or_default();
        let recommended_action = extract_string_attr(&usages, "recommendedAction");

        // Item defaults to the doc comment or the parent definition
        let item = def.doc.clone()
            .or_else(|| def.parent_def.clone())
            .unwrap_or_default();

        risks.push(RiskDef {
            id: def.name.clone(),
            item,
            failure_mode,
            failure_effect,
            failure_cause,
            current_controls,
            category: category_str.and_then(|s| RiskCategory::from_str_value(&s)),
            status: status_str.and_then(|s| RiskStatus::from_str_value(&s)),
            severity,
            occurrence,
            detection,
            rpn,
            acceptance,
            owner,
            notes,
            recommended_action,
            assigned_to: def.parent_def.clone(),
        });
    }

    risks
}

/// Try to extract a numeric level (1–5) from a named attribute usage.
fn extract_level(usages: &[&sysml_core::model::Usage], attr_name: &str) -> Option<u32> {
    for u in usages {
        if u.name == attr_name {
            if let Some(expr) = &u.value_expr {
                if let Ok(v) = expr.trim().parse::<u32>() {
                    if (1..=5).contains(&v) {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}

/// Try to extract a string value from a named attribute usage.
fn extract_string_attr(usages: &[&sysml_core::model::Usage], attr_name: &str) -> Option<String> {
    for u in usages {
        if u.name == attr_name {
            if let Some(expr) = &u.value_expr {
                let trimmed = expr.trim().trim_matches('"');
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

// =========================================================================
// Record creation
// =========================================================================

/// Create a [`RecordEnvelope`] for a newly identified risk.
pub fn create_risk_record(risk: &RiskDef, author: &str) -> RecordEnvelope {
    let id = generate_record_id("risk", "entity", author);

    let mut refs = BTreeMap::new();
    refs.insert("risk".to_string(), vec![risk.id.clone()]);

    let mut data = BTreeMap::new();
    data.insert("item".into(), RecordValue::String(risk.item.clone()));
    data.insert("failure_mode".into(), RecordValue::String(risk.failure_mode.clone()));
    if !risk.failure_effect.is_empty() {
        data.insert("failure_effect".into(), RecordValue::String(risk.failure_effect.clone()));
    }
    if !risk.failure_cause.is_empty() {
        data.insert("failure_cause".into(), RecordValue::String(risk.failure_cause.clone()));
    }
    if !risk.current_controls.is_empty() {
        data.insert("current_controls".into(), RecordValue::String(risk.current_controls.clone()));
    }
    if let Some(cat) = &risk.category {
        data.insert("category".into(), RecordValue::String(cat.label().to_string()));
    }
    if let Some(st) = &risk.status {
        data.insert("status".into(), RecordValue::String(st.label().to_string()));
    }
    if let Some(s) = risk.severity {
        data.insert("severity".into(), RecordValue::Integer(s as i64));
        data.insert("severity_label".into(), RecordValue::String(severity_label(s).to_string()));
    }
    if let Some(o) = risk.occurrence {
        data.insert("occurrence".into(), RecordValue::Integer(o as i64));
        data.insert("occurrence_label".into(), RecordValue::String(occurrence_label(o).to_string()));
    }
    if let Some(d) = risk.detection {
        data.insert("detection".into(), RecordValue::Integer(d as i64));
        data.insert("detection_label".into(), RecordValue::String(detection_label(d).to_string()));
    }
    if let Some(rpn) = risk.rpn {
        data.insert("rpn".into(), RecordValue::Integer(rpn as i64));
    }
    if let Some(acc) = &risk.acceptance {
        data.insert("acceptance".into(), RecordValue::String(acc.label().to_string()));
    }
    if let Some(owner) = &risk.owner {
        data.insert("owner".into(), RecordValue::String(owner.clone()));
    }
    if let Some(notes) = &risk.notes {
        data.insert("notes".into(), RecordValue::String(notes.clone()));
    }
    if let Some(action) = &risk.recommended_action {
        data.insert("recommended_action".into(), RecordValue::String(action.clone()));
    }

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "risk".into(),
            record_type: "entity".into(),
            created: sysml_core::record::now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

/// Create a [`RecordEnvelope`] for a risk assessment (append-only history).
pub fn create_assessment_record(assessment: &RiskAssessment, author: &str) -> RecordEnvelope {
    let id = generate_record_id("risk", "assessment", author);

    let mut refs = BTreeMap::new();
    refs.insert("risk".to_string(), vec![assessment.risk_id.clone()]);

    let mut data = BTreeMap::new();
    data.insert("severity".into(), RecordValue::Integer(assessment.severity as i64));
    data.insert("severity_label".into(), RecordValue::String(severity_label(assessment.severity).to_string()));
    data.insert("occurrence".into(), RecordValue::Integer(assessment.occurrence as i64));
    data.insert("occurrence_label".into(), RecordValue::String(occurrence_label(assessment.occurrence).to_string()));
    data.insert("detection".into(), RecordValue::Integer(assessment.detection as i64));
    data.insert("detection_label".into(), RecordValue::String(detection_label(assessment.detection).to_string()));
    data.insert("rpn".into(), RecordValue::Integer(assessment.rpn as i64));
    if let Some(notes) = &assessment.notes {
        data.insert("notes".into(), RecordValue::String(notes.clone()));
    }
    data.insert("assessed_by".into(), RecordValue::String(assessment.assessed_by.clone()));

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "risk".into(),
            record_type: "assessment".into(),
            created: sysml_core::record::now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

/// Create a [`RecordEnvelope`] for a mitigation action.
pub fn create_mitigation_record(mitigation: &Mitigation, author: &str) -> RecordEnvelope {
    let id = generate_record_id("risk", "mitigation", author);

    let mut refs = BTreeMap::new();
    refs.insert("risk".to_string(), vec![mitigation.risk_id.clone()]);
    refs.insert("mitigation".to_string(), vec![mitigation.id.clone()]);

    let mut data = BTreeMap::new();
    data.insert("description".into(), RecordValue::String(mitigation.description.clone()));
    data.insert("strategy".into(), RecordValue::String(mitigation.strategy.label().to_string()));
    if let Some(owner) = &mitigation.owner {
        data.insert("owner".into(), RecordValue::String(owner.clone()));
    }
    if let Some(due) = &mitigation.due_date {
        data.insert("due_date".into(), RecordValue::String(due.clone()));
    }
    if let Some(st) = &mitigation.status {
        data.insert("status".into(), RecordValue::String(st.label().to_string()));
    }

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "risk".into(),
            record_type: "mitigation".into(),
            created: sysml_core::record::now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

// =========================================================================
// Interactive wizards
// =========================================================================

/// Build the full risk creation wizard (7 steps).
///
/// Prompts for: failure mode, failure effect, failure cause, severity (1–5),
/// occurrence (1–5), detection (1–5), recommended action.
pub fn build_risk_wizard() -> Vec<WizardStep> {
    vec![
        WizardStep::string("failure_mode", "Failure mode")
            .with_explanation("What could go wrong? Describe the specific failure mode."),
        WizardStep::string("failure_effect", "Failure effect")
            .with_explanation("What would the consequence be if this failure occurs?"),
        WizardStep::string("failure_cause", "Failure cause")
            .with_explanation("What is the root cause or mechanism that triggers this failure?"),
        WizardStep::number("severity", "Severity (1-5)")
            .with_explanation(
                "Rate the severity of the failure effect:\n\
                 1 = Negligible  2 = Marginal  3 = Moderate\n\
                 4 = Critical    5 = Catastrophic",
            )
            .with_bounds(Some(1.0), Some(5.0)),
        WizardStep::number("occurrence", "Occurrence (1-5)")
            .with_explanation(
                "Rate the probability of this failure occurring:\n\
                 1 = Improbable  2 = Remote    3 = Occasional\n\
                 4 = Probable    5 = Frequent",
            )
            .with_bounds(Some(1.0), Some(5.0)),
        WizardStep::number("detection", "Detection (1-5)")
            .with_explanation(
                "Rate how easily current controls can detect this failure\n\
                 (higher = harder to detect):\n\
                 1 = Almost Certain  2 = High      3 = Moderate\n\
                 4 = Low             5 = Almost Impossible",
            )
            .with_bounds(Some(1.0), Some(5.0)),
        WizardStep::string("recommended_action", "Recommended action")
            .with_explanation("What action should be taken to reduce or eliminate this risk?")
            .optional(),
    ]
}

/// Build wizard for re-assessing an existing risk.
pub fn build_assessment_wizard(risk: &RiskDef) -> Vec<WizardStep> {
    let context = format!(
        "Assessing: {} (current RPN: {})",
        risk.failure_mode,
        risk.rpn.map_or("N/A".to_string(), |r| r.to_string()),
    );

    let mut sev_step = WizardStep::number("severity", &format!("{context}\nSeverity (1-5)"))
        .with_explanation(
            "Re-evaluate severity. Has the potential impact changed since last review?\n\
             1 = Negligible  2 = Marginal  3 = Moderate  4 = Critical  5 = Catastrophic",
        )
        .with_bounds(Some(1.0), Some(5.0));
    if let Some(s) = risk.severity {
        sev_step.default = Some(s.to_string());
    }

    let mut occ_step = WizardStep::number("occurrence", "Occurrence (1-5)")
        .with_explanation(
            "Has the probability changed due to new information or mitigations?\n\
             1 = Improbable  2 = Remote  3 = Occasional  4 = Probable  5 = Frequent",
        )
        .with_bounds(Some(1.0), Some(5.0));
    if let Some(o) = risk.occurrence {
        occ_step.default = Some(o.to_string());
    }

    let mut det_step = WizardStep::number("detection", "Detection (1-5)")
        .with_explanation(
            "Have new tests or monitoring improved detection capability?\n\
             1 = Almost Certain  2 = High  3 = Moderate  4 = Low  5 = Almost Impossible",
        )
        .with_bounds(Some(1.0), Some(5.0));
    if let Some(d) = risk.detection {
        det_step.default = Some(d.to_string());
    }

    vec![
        sev_step,
        occ_step,
        det_step,
        WizardStep::string("notes", "Assessment notes")
            .with_explanation("Record rationale for any changes in scores.")
            .optional(),
    ]
}

// =========================================================================
// Risk matrix
// =========================================================================

/// A 5×5 risk matrix indexed by severity (rows) and occurrence (columns).
///
/// Each cell contains risk IDs and its acceptance level.
#[derive(Debug, Clone, Serialize)]
pub struct RiskMatrix {
    /// `cells[severity_index][occurrence_index]` where both indices are 0–4.
    pub cells: [[Vec<String>; 5]; 5],
}

impl RiskMatrix {
    pub fn new() -> Self {
        Self {
            cells: Default::default(),
        }
    }

    /// Render the matrix as ASCII art with acceptance zone markers.
    pub fn to_text(&self) -> String {
        let mut out = String::new();

        out.push_str("                  | Improb. | Remote  | Occasnl | Probabl | Frequnt |\n");
        out.push_str("------------------+---------+---------+---------+---------+---------+\n");

        let row_labels = [
            "Negligible",
            "Marginal  ",
            "Moderate  ",
            "Critical  ",
            "Catastroph",
        ];
        for sev_idx in (0..5).rev() {
            out.push_str(&format!("{:18}|", row_labels[sev_idx]));
            for occ_idx in 0..5 {
                let ids = &self.cells[sev_idx][occ_idx];
                let acc = risk_acceptance(sev_idx as u32 + 1, occ_idx as u32 + 1);
                let zone = acc.short();
                let cell_text = if ids.is_empty() {
                    format!("{} -   ", zone)
                } else if ids.len() == 1 {
                    format!("{}{}", zone, truncate_id(&ids[0], 5))
                } else {
                    format!("{}({})", zone, ids.len())
                };
                out.push_str(&format!(" {:>7} |", cell_text));
            }
            out.push('\n');
        }

        out.push_str("\nZones: !! = Unacceptable  ! = Undesirable  ? = Review    (blank) = Acceptable\n");

        out
    }
}

impl Default for RiskMatrix {
    fn default() -> Self {
        Self::new()
    }
}

fn truncate_id(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 2 {
        format!("{}..", &s[..max_len - 2])
    } else {
        s[..max_len].to_string()
    }
}

/// Organize risks into a 5×5 severity × occurrence grid.
pub fn generate_risk_matrix(risks: &[RiskDef]) -> RiskMatrix {
    let mut matrix = RiskMatrix::new();

    for risk in risks {
        if let (Some(sev), Some(occ)) = (risk.severity, risk.occurrence) {
            if sev >= 1 && sev <= 5 && occ >= 1 && occ <= 5 {
                let sev_idx = (sev - 1) as usize;
                let occ_idx = (occ - 1) as usize;
                matrix.cells[sev_idx][occ_idx].push(risk.id.clone());
            }
        }
    }

    matrix
}

// =========================================================================
// FMEA table
// =========================================================================

/// A single row in an FMEA worksheet (AIAG/VDA format).
#[derive(Debug, Clone, Serialize)]
pub struct FmeaRow {
    pub item: String,
    pub failure_mode: String,
    pub failure_effect: String,
    pub failure_cause: String,
    pub current_controls: String,
    pub severity: u32,
    pub occurrence: u32,
    pub detection: u32,
    pub rpn: u32,
    pub acceptance: String,
    pub recommended_action: String,
    pub status: String,
    pub assigned_to: Option<String>,
}

/// Generate FMEA worksheet data from risk definitions.
pub fn generate_fmea_table(risks: &[RiskDef]) -> Vec<FmeaRow> {
    risks
        .iter()
        .map(|risk| {
            let sev = risk.severity.unwrap_or(0);
            let occ = risk.occurrence.unwrap_or(0);
            let det = risk.detection.unwrap_or(0);
            let rpn = risk.rpn.unwrap_or(0);
            let acceptance = risk.acceptance
                .map(|a| a.label().to_string())
                .unwrap_or_else(|| "-".to_string());

            FmeaRow {
                item: risk.item.clone(),
                failure_mode: if risk.failure_mode.is_empty() {
                    risk.id.clone()
                } else {
                    risk.failure_mode.clone()
                },
                failure_effect: risk.failure_effect.clone(),
                failure_cause: risk.failure_cause.clone(),
                current_controls: risk.current_controls.clone(),
                severity: sev,
                occurrence: occ,
                detection: det,
                rpn,
                acceptance,
                recommended_action: risk.recommended_action.clone().unwrap_or_default(),
                status: risk
                    .status
                    .map(|s| s.label().to_string())
                    .unwrap_or_else(|| "-".to_string()),
                assigned_to: risk.assigned_to.clone(),
            }
        })
        .collect()
}

// =========================================================================
// Risk trend
// =========================================================================

/// Extract (date, RPN) pairs from a series of assessments for trend charting.
pub fn risk_trend(assessments: &[RiskAssessment]) -> Vec<(String, u32)> {
    assessments
        .iter()
        .enumerate()
        .map(|(i, a)| (format!("#{i}"), a.rpn))
        .collect()
}

// =========================================================================
// SysML generation
// =========================================================================

/// Generate SysML text for a risk definition using numeric scores.
pub fn risk_to_sysml(
    name: &str,
    failure_mode: &str,
    severity: u32,
    occurrence: u32,
    detection: u32,
    failure_effect: Option<&str>,
    failure_cause: Option<&str>,
    recommended_action: Option<&str>,
) -> String {
    let mut out = format!("part {} : RiskDef {{\n", name);
    out.push_str(&format!("    doc /* {} */\n", failure_mode));
    out.push_str(&format!("    attribute severity = {};\n", severity));
    out.push_str(&format!("    attribute occurrence = {};\n", occurrence));
    out.push_str(&format!("    attribute detection = {};\n", detection));
    if let Some(effect) = failure_effect {
        if !effect.is_empty() {
            out.push_str(&format!("    attribute failureEffect = \"{}\";\n", effect));
        }
    }
    if let Some(cause) = failure_cause {
        if !cause.is_empty() {
            out.push_str(&format!("    attribute failureCause = \"{}\";\n", cause));
        }
    }
    if let Some(action) = recommended_action {
        if !action.is_empty() {
            out.push_str(&format!("    attribute recommendedAction = \"{}\";\n", action));
        }
    }
    out.push_str("}\n");
    out
}

/// Build wizard steps for interactive risk creation (`risk add`).
/// If a model is provided, enum choices are extracted from it.
pub fn build_risk_add_wizard(_model: Option<&sysml_core::model::Model>) -> Vec<sysml_core::interactive::WizardStep> {
    vec![
        WizardStep::string("failure_mode", "Failure mode")
            .with_explanation("What could go wrong? A brief description of the failure."),
        WizardStep::string("failure_effect", "Failure effect")
            .with_explanation("What would happen if this failure occurs?")
            .optional(),
        WizardStep::string("failure_cause", "Failure cause")
            .with_explanation("Root cause or mechanism of the failure.")
            .optional(),
        WizardStep::number("severity", "Severity (1-5)")
            .with_explanation(
                "1 = Negligible  2 = Marginal  3 = Moderate  4 = Critical  5 = Catastrophic",
            )
            .with_bounds(Some(1.0), Some(5.0)),
        WizardStep::number("occurrence", "Occurrence (1-5)")
            .with_explanation(
                "1 = Improbable  2 = Remote  3 = Occasional  4 = Probable  5 = Frequent",
            )
            .with_bounds(Some(1.0), Some(5.0)),
        WizardStep::number("detection", "Detection (1-5)")
            .with_explanation(
                "Higher = harder to detect.\n\
                 1 = Almost Certain  2 = High  3 = Moderate  4 = Low  5 = Almost Impossible",
            )
            .with_bounds(Some(1.0), Some(5.0)),
        WizardStep::string("recommended_action", "Recommended action")
            .with_explanation("What action should be taken to reduce this risk?")
            .optional(),
    ]
}

/// Interpret wizard results into (element_name, sysml_text).
pub fn interpret_risk_add_wizard(result: &sysml_core::interactive::WizardResult) -> Option<(String, String)> {
    let failure_mode = result.get_string("failure_mode")?;
    let severity = result.get_number("severity")? as u32;
    let occurrence = result.get_number("occurrence")? as u32;
    let detection = result.get_number("detection")? as u32;
    let failure_effect = result.get_string("failure_effect");
    let failure_cause = result.get_string("failure_cause");
    let recommended_action = result.get_string("recommended_action");

    let name = format!("risk{}", title_to_identifier(failure_mode));

    let rpn = severity * occurrence * detection;
    let acc = risk_acceptance(severity, occurrence);

    let sysml = risk_to_sysml(
        &name,
        failure_mode,
        severity,
        occurrence,
        detection,
        failure_effect.map(|s| s.as_ref()),
        failure_cause.map(|s| s.as_ref()),
        recommended_action.map(|s| s.as_ref()),
    );

    let preview = format!(
        "{}RPN: {} ({} × {} × {})  Risk: {}",
        sysml, rpn, severity, occurrence, detection, acc.label()
    );

    Some((name, preview))
}

fn title_to_identifier(title: &str) -> String {
    title
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => {
                    let rest: String = chars.collect();
                    format!("{}{}", c.to_uppercase(), rest.to_lowercase())
                }
                None => String::new(),
            }
        })
        .collect()
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use sysml_core::model::{Definition, Model, Span, Usage};

    // -- Score label tests ---------------------------------------------------

    #[test]
    fn severity_labels() {
        assert_eq!(severity_label(1), "Negligible");
        assert_eq!(severity_label(5), "Catastrophic");
        assert_eq!(severity_label(0), "?");
        assert_eq!(severity_label(6), "?");
    }

    #[test]
    fn occurrence_labels() {
        assert_eq!(occurrence_label(1), "Improbable");
        assert_eq!(occurrence_label(5), "Frequent");
    }

    #[test]
    fn detection_labels() {
        assert_eq!(detection_label(1), "Almost Certain");
        assert_eq!(detection_label(5), "Almost Impossible");
    }

    // -- Risk acceptance tests -----------------------------------------------

    #[test]
    fn acceptance_broadly_acceptable() {
        assert_eq!(risk_acceptance(1, 1), RiskAcceptance::BroadlyAcceptable);
        assert_eq!(risk_acceptance(1, 3), RiskAcceptance::BroadlyAcceptable);
        assert_eq!(risk_acceptance(3, 1), RiskAcceptance::BroadlyAcceptable);
    }

    #[test]
    fn acceptance_review() {
        assert_eq!(risk_acceptance(2, 2), RiskAcceptance::AcceptableWithReview);
        assert_eq!(risk_acceptance(1, 4), RiskAcceptance::AcceptableWithReview);
    }

    #[test]
    fn acceptance_undesirable() {
        assert_eq!(risk_acceptance(2, 4), RiskAcceptance::Undesirable);
        assert_eq!(risk_acceptance(4, 2), RiskAcceptance::Undesirable);
    }

    #[test]
    fn acceptance_unacceptable() {
        assert_eq!(risk_acceptance(5, 3), RiskAcceptance::Unacceptable);
        assert_eq!(risk_acceptance(3, 5), RiskAcceptance::Unacceptable);
        assert_eq!(risk_acceptance(5, 5), RiskAcceptance::Unacceptable);
        assert_eq!(risk_acceptance(4, 4), RiskAcceptance::Unacceptable);
    }

    // -- Enum tests ---------------------------------------------------------

    #[test]
    fn risk_category_all_and_labels() {
        assert_eq!(RiskCategory::all().len(), 7);
        assert_eq!(RiskCategory::Technical.label(), "Technical");
        assert_eq!(RiskCategory::SupplyChain.label(), "Supply Chain");
    }

    #[test]
    fn risk_category_from_str_roundtrip() {
        assert_eq!(RiskCategory::from_str_value("technical"), Some(RiskCategory::Technical));
        assert_eq!(RiskCategory::from_str_value("supply_chain"), Some(RiskCategory::SupplyChain));
        assert_eq!(RiskCategory::from_str_value("unknown"), None);
    }

    #[test]
    fn risk_status_labels() {
        assert_eq!(RiskStatus::Identified.label(), "Identified");
        assert_eq!(RiskStatus::Mitigating.label(), "Mitigating");
        assert_eq!(RiskStatus::Closed.label(), "Closed");
    }

    #[test]
    fn risk_status_from_str_roundtrip() {
        assert_eq!(RiskStatus::from_str_value("analyzing"), Some(RiskStatus::Analyzing));
        assert_eq!(RiskStatus::from_str_value("accepted"), Some(RiskStatus::Accepted));
        assert_eq!(RiskStatus::from_str_value("bogus"), None);
    }

    #[test]
    fn mitigation_strategy_labels() {
        assert_eq!(MitigationStrategy::Avoid.label(), "Avoid");
        assert_eq!(MitigationStrategy::Contingency.label(), "Contingency");
    }

    #[test]
    fn mitigation_strategy_from_str_roundtrip() {
        assert_eq!(MitigationStrategy::from_str_value("reduce"), Some(MitigationStrategy::Reduce));
        assert_eq!(MitigationStrategy::from_str_value("transfer"), Some(MitigationStrategy::Transfer));
        assert_eq!(MitigationStrategy::from_str_value("nope"), None);
    }

    // -- RPN computation ----------------------------------------------------

    #[test]
    fn compute_rpn_min() {
        assert_eq!(compute_rpn(1, 1, 1), 1);
    }

    #[test]
    fn compute_rpn_max() {
        assert_eq!(compute_rpn(5, 5, 5), 125);
    }

    #[test]
    fn compute_rpn_mid() {
        assert_eq!(compute_rpn(3, 3, 3), 27);
    }

    #[test]
    fn compute_rpn_asymmetric() {
        assert_eq!(compute_rpn(5, 1, 1), 5);
    }

    // -- RiskDef recompute --------------------------------------------------

    #[test]
    fn risk_def_recompute_complete() {
        let mut risk = sample_risk();
        risk.recompute();
        assert_eq!(risk.rpn, Some(27));
        assert_eq!(risk.acceptance, Some(RiskAcceptance::Undesirable));
    }

    #[test]
    fn risk_def_recompute_incomplete() {
        let mut risk = sample_risk();
        risk.detection = None;
        risk.recompute();
        assert_eq!(risk.rpn, None);
        // Acceptance still computed from S×O
        assert_eq!(risk.acceptance, Some(RiskAcceptance::Undesirable));
    }

    // -- Model extraction ---------------------------------------------------

    #[test]
    fn extract_risks_from_model() {
        let model = build_test_model();
        let risks = extract_risks(&model);
        assert_eq!(risks.len(), 2);

        let thermal = risks.iter().find(|r| r.id == "ThermalRisk").unwrap();
        assert_eq!(thermal.severity, Some(4));
        assert_eq!(thermal.occurrence, Some(3));
        assert_eq!(thermal.rpn, Some(4 * 3 * 2));

        let supply = risks.iter().find(|r| r.id == "SupplyChainRisk").unwrap();
        assert!(supply.severity.is_none());
        assert!(supply.rpn.is_none());
    }

    #[test]
    fn extract_risks_ignores_non_risk_defs() {
        let mut model = Model::new("test.sysml".into());
        model.definitions.push(make_def("Vehicle", None));
        assert!(extract_risks(&model).is_empty());
    }

    #[test]
    fn extract_risks_by_supertype() {
        let mut model = Model::new("test.sysml".into());
        model.definitions.push(make_def("MyIssue", Some("RiskDef")));
        let risks = extract_risks(&model);
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].id, "MyIssue");
    }

    #[test]
    fn extract_risks_accepts_occurrence_and_likelihood() {
        // "occurrence" attribute name
        let mut model = Model::new("test.sysml".into());
        model.definitions.push(make_def("HazardA", None));
        model.usages.push(make_usage("HazardA", "severity", "3"));
        model.usages.push(make_usage("HazardA", "occurrence", "4"));
        model.usages.push(make_usage("HazardA", "detection", "2"));
        let risks = extract_risks(&model);
        assert_eq!(risks[0].occurrence, Some(4));
        assert_eq!(risks[0].rpn, Some(24));

        // "likelihood" attribute name (backward compat)
        let mut model2 = Model::new("test.sysml".into());
        model2.definitions.push(make_def("HazardB", None));
        model2.usages.push(make_usage("HazardB", "severity", "3"));
        model2.usages.push(make_usage("HazardB", "likelihood", "2"));
        model2.usages.push(make_usage("HazardB", "detectability", "1"));
        let risks2 = extract_risks(&model2);
        assert_eq!(risks2[0].occurrence, Some(2));
        assert_eq!(risks2[0].detection, Some(1));
    }

    // -- Record creation ----------------------------------------------------

    #[test]
    fn create_risk_record_structure() {
        let risk = sample_risk();
        let envelope = create_risk_record(&risk, "alice");

        assert_eq!(envelope.meta.tool, "risk");
        assert_eq!(envelope.meta.record_type, "entity");
        assert_eq!(envelope.meta.author, "alice");
        assert!(envelope.refs.contains_key("risk"));
        assert_eq!(envelope.data.get("severity"), Some(&RecordValue::Integer(3)));
        assert_eq!(envelope.data.get("rpn"), Some(&RecordValue::Integer(27)));
        assert!(envelope.data.contains_key("occurrence"));
        assert!(envelope.data.contains_key("detection"));
    }

    #[test]
    fn create_risk_record_minimal() {
        let risk = RiskDef::new("R1");
        let envelope = create_risk_record(&risk, "bob");
        assert_eq!(envelope.meta.tool, "risk");
        assert!(!envelope.data.contains_key("severity"));
        assert!(!envelope.data.contains_key("rpn"));
    }

    #[test]
    fn create_assessment_record_structure() {
        let assessment = sample_assessment();
        let envelope = create_assessment_record(&assessment, "charlie");

        assert_eq!(envelope.meta.tool, "risk");
        assert_eq!(envelope.meta.record_type, "assessment");
        assert_eq!(envelope.data.get("rpn"), Some(&RecordValue::Integer(60)));
        assert_eq!(
            envelope.data.get("assessed_by"),
            Some(&RecordValue::String("charlie_assessor".into()))
        );
        assert!(envelope.data.contains_key("occurrence"));
    }

    #[test]
    fn create_mitigation_record_structure() {
        let mitigation = sample_mitigation();
        let envelope = create_mitigation_record(&mitigation, "dave");

        assert_eq!(envelope.meta.tool, "risk");
        assert_eq!(envelope.meta.record_type, "mitigation");
        assert!(envelope.refs.contains_key("risk"));
        assert!(envelope.refs.contains_key("mitigation"));
        assert_eq!(
            envelope.data.get("strategy"),
            Some(&RecordValue::String("Reduce".into()))
        );
        assert_eq!(
            envelope.data.get("due_date"),
            Some(&RecordValue::String("2026-06-01".into()))
        );
    }

    #[test]
    fn record_toml_round_trip() {
        let risk = sample_risk();
        let envelope = create_risk_record(&risk, "tester");
        let toml = envelope.to_toml_string();
        let parsed = RecordEnvelope::from_toml_str(&toml).unwrap();
        assert_eq!(parsed.meta.tool, "risk");
        assert_eq!(parsed.meta.record_type, "entity");
        assert_eq!(parsed.data.get("rpn"), envelope.data.get("rpn"));
    }

    // -- Wizard tests -------------------------------------------------------

    #[test]
    fn risk_wizard_has_expected_steps() {
        let steps = build_risk_wizard();
        assert_eq!(steps.len(), 7);

        let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            ["failure_mode", "failure_effect", "failure_cause",
             "severity", "occurrence", "detection", "recommended_action"]
        );

        // First three are required, last is optional
        assert!(steps[0].required);
        assert!(!steps[6].required);

        // All steps have explanations
        for step in &steps {
            assert!(step.explanation.is_some(), "step '{}' should have an explanation", step.id);
        }
    }

    #[test]
    fn risk_wizard_severity_is_numeric() {
        let steps = build_risk_wizard();
        let sev_step = &steps[3]; // severity
        assert!(matches!(sev_step.kind, PromptKind::Number { min: Some(m), max: Some(x) } if m == 1.0 && x == 5.0));
    }

    #[test]
    fn assessment_wizard_has_defaults() {
        let risk = sample_risk();
        let steps = build_assessment_wizard(&risk);
        assert_eq!(steps.len(), 4);

        assert_eq!(steps[0].default.as_deref(), Some("3"));
        assert_eq!(steps[1].default.as_deref(), Some("3"));
        assert_eq!(steps[2].default.as_deref(), Some("3"));
    }

    #[test]
    fn assessment_wizard_no_defaults_when_none() {
        let risk = RiskDef::new("R1");
        let steps = build_assessment_wizard(&risk);
        assert!(steps[0].default.is_none());
        assert!(steps[1].default.is_none());
        assert!(steps[2].default.is_none());
    }

    // -- Risk matrix --------------------------------------------------------

    #[test]
    fn generate_risk_matrix_empty() {
        let matrix = generate_risk_matrix(&[]);
        for row in &matrix.cells {
            for cell in row {
                assert!(cell.is_empty());
            }
        }
    }

    #[test]
    fn generate_risk_matrix_placement() {
        let mut r1 = RiskDef::new("R1");
        r1.severity = Some(4);  // Critical -> idx 3
        r1.occurrence = Some(3); // Occasional -> idx 2

        let mut r2 = RiskDef::new("R2");
        r2.severity = Some(4);
        r2.occurrence = Some(3);

        let matrix = generate_risk_matrix(&[r1, r2]);
        assert_eq!(matrix.cells[3][2].len(), 2);
        assert!(matrix.cells[3][2].contains(&"R1".to_string()));
        assert!(matrix.cells[3][2].contains(&"R2".to_string()));
        assert!(matrix.cells[0][0].is_empty());
        assert!(matrix.cells[4][4].is_empty());
    }

    #[test]
    fn generate_risk_matrix_skips_incomplete() {
        let mut r = RiskDef::new("R1");
        r.severity = Some(4);
        // occurrence missing

        let matrix = generate_risk_matrix(&[r]);
        for row in &matrix.cells {
            for cell in row {
                assert!(cell.is_empty());
            }
        }
    }

    #[test]
    fn risk_matrix_to_text_contains_headers() {
        let matrix = generate_risk_matrix(&[]);
        let text = matrix.to_text();
        assert!(text.contains("Improb."));
        assert!(text.contains("Frequnt"));
        assert!(text.contains("Catastroph"));
        assert!(text.contains("Negligible"));
    }

    #[test]
    fn risk_matrix_to_text_contains_zones() {
        let text = generate_risk_matrix(&[]).to_text();
        assert!(text.contains("Unacceptable"));
        assert!(text.contains("Acceptable"));
    }

    #[test]
    fn risk_matrix_to_text_shows_risk_id() {
        let mut r = RiskDef::new("R1");
        r.severity = Some(1);
        r.occurrence = Some(1);
        let text = generate_risk_matrix(&[r]).to_text();
        assert!(text.contains("R1"));
    }

    #[test]
    fn risk_matrix_to_text_shows_count_for_multiple() {
        let mut a = RiskDef::new("A");
        a.severity = Some(4);
        a.occurrence = Some(5);

        let mut b = RiskDef::new("B");
        b.severity = Some(4);
        b.occurrence = Some(5);

        let text = generate_risk_matrix(&[a, b]).to_text();
        assert!(text.contains("(2)"));
    }

    // -- FMEA table ---------------------------------------------------------

    #[test]
    fn generate_fmea_table_complete() {
        let risk = sample_risk();
        let rows = generate_fmea_table(&[risk]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].severity, 3);
        assert_eq!(rows[0].occurrence, 3);
        assert_eq!(rows[0].detection, 3);
        assert_eq!(rows[0].rpn, 27);
        assert!(!rows[0].acceptance.is_empty());
    }

    #[test]
    fn generate_fmea_table_incomplete() {
        let mut r = RiskDef::new("R1");
        r.severity = Some(4);
        let rows = generate_fmea_table(&[r]);
        assert_eq!(rows[0].severity, 4);
        assert_eq!(rows[0].occurrence, 0);
        assert_eq!(rows[0].detection, 0);
        assert_eq!(rows[0].rpn, 0);
    }

    #[test]
    fn generate_fmea_table_empty() {
        assert!(generate_fmea_table(&[]).is_empty());
    }

    #[test]
    fn generate_fmea_table_preserves_status() {
        let mut r = RiskDef::new("R1");
        r.status = Some(RiskStatus::Mitigating);
        let rows = generate_fmea_table(&[r]);
        assert_eq!(rows[0].status, "Mitigating");
    }

    #[test]
    fn generate_fmea_table_shows_failure_fields() {
        let mut r = RiskDef::new("R1");
        r.failure_mode = "Seal leak".into();
        r.failure_effect = "Water ingress".into();
        r.failure_cause = "Tool wear".into();
        r.current_controls = "Visual inspection".into();
        r.recommended_action = Some("Add pressure test".into());
        r.severity = Some(4);
        r.occurrence = Some(3);
        r.detection = Some(3);
        r.rpn = Some(36);
        r.acceptance = Some(RiskAcceptance::Undesirable);

        let rows = generate_fmea_table(&[r]);
        assert_eq!(rows[0].failure_mode, "Seal leak");
        assert_eq!(rows[0].failure_effect, "Water ingress");
        assert_eq!(rows[0].failure_cause, "Tool wear");
        assert_eq!(rows[0].current_controls, "Visual inspection");
        assert_eq!(rows[0].recommended_action, "Add pressure test");
    }

    // -- Risk trend ---------------------------------------------------------

    #[test]
    fn risk_trend_empty() {
        assert!(risk_trend(&[]).is_empty());
    }

    #[test]
    fn risk_trend_ordering() {
        let assessments = vec![
            RiskAssessment {
                risk_id: "R1".into(),
                severity: 4,
                occurrence: 5,
                detection: 4,
                rpn: 80,
                notes: None,
                assessed_by: "alice".into(),
            },
            RiskAssessment {
                risk_id: "R1".into(),
                severity: 3,
                occurrence: 3,
                detection: 3,
                rpn: 27,
                notes: None,
                assessed_by: "alice".into(),
            },
        ];
        let trend = risk_trend(&assessments);
        assert_eq!(trend.len(), 2);
        assert_eq!(trend[0], ("#0".to_string(), 80));
        assert_eq!(trend[1], ("#1".to_string(), 27));
    }

    #[test]
    fn risk_trend_single_assessment() {
        let assessments = vec![sample_assessment()];
        let trend = risk_trend(&assessments);
        assert_eq!(trend.len(), 1);
        assert_eq!(trend[0].1, 60);
    }

    // -- Helper: truncate_id ------------------------------------------------

    #[test]
    fn truncate_id_short() {
        assert_eq!(truncate_id("R1", 7), "R1");
    }

    #[test]
    fn truncate_id_exact() {
        assert_eq!(truncate_id("R123456", 7), "R123456");
    }

    #[test]
    fn truncate_id_long() {
        assert_eq!(truncate_id("ReallyLongRiskId", 7), "Reall..");
    }

    // -- Serialization sanity -----------------------------------------------

    #[test]
    fn risk_def_serializes_to_json() {
        let risk = sample_risk();
        let json = serde_json::to_string(&risk).unwrap();
        assert!(json.contains("\"severity\""));
        assert!(json.contains("\"failure_mode\""));
    }

    #[test]
    fn fmea_row_serializes_to_json() {
        let risk = sample_risk();
        let rows = generate_fmea_table(&[risk]);
        let json = serde_json::to_string(&rows[0]).unwrap();
        assert!(json.contains("\"rpn\""));
        assert!(json.contains("27"));
    }

    #[test]
    fn risk_matrix_serializes_to_json() {
        let matrix = generate_risk_matrix(&[]);
        let json = serde_json::to_string(&matrix).unwrap();
        assert!(json.contains("\"cells\""));
    }

    // -- SysML generator tests ----------------------------------------------

    #[test]
    fn risk_to_sysml_basic() {
        let sysml = risk_to_sysml(
            "riskBrakeFail", "Brake system failure",
            4, 2, 3, None, None, None,
        );
        assert!(sysml.contains("part riskBrakeFail : RiskDef {"));
        assert!(sysml.contains("doc /* Brake system failure */"));
        assert!(sysml.contains("attribute severity = 4;"));
        assert!(sysml.contains("attribute occurrence = 2;"));
        assert!(sysml.contains("attribute detection = 3;"));
    }

    #[test]
    fn risk_to_sysml_with_fmea_fields() {
        let sysml = risk_to_sysml(
            "riskOverheat", "Overheating", 3, 3, 3,
            Some("Thermal damage"), Some("Blocked airflow"),
            Some("Add thermal cutoff"),
        );
        assert!(sysml.contains("attribute failureEffect = \"Thermal damage\""));
        assert!(sysml.contains("attribute failureCause = \"Blocked airflow\""));
        assert!(sysml.contains("attribute recommendedAction = \"Add thermal cutoff\""));
    }

    #[test]
    fn build_risk_wizard_defaults() {
        let steps = build_risk_add_wizard(None);
        assert_eq!(steps.len(), 7);
        assert_eq!(steps[0].id, "failure_mode");
        assert_eq!(steps[3].id, "severity");
        assert_eq!(steps[4].id, "occurrence");
        assert_eq!(steps[5].id, "detection");
    }

    #[test]
    fn interpret_risk_wizard() {
        use sysml_core::interactive::*;
        let mut result = WizardResult::new();
        result.set("failure_mode", WizardAnswer::String("Brake leak".into()));
        result.set("severity", WizardAnswer::Number(4.0));
        result.set("occurrence", WizardAnswer::Number(2.0));
        result.set("detection", WizardAnswer::Number(3.0));
        let (name, preview) = interpret_risk_add_wizard(&result).unwrap();
        assert_eq!(name, "riskBrakeLeak");
        assert!(preview.contains("attribute severity = 4;"));
        assert!(preview.contains("attribute occurrence = 2;"));
        assert!(preview.contains("RPN: 24"));
    }

    #[test]
    fn title_to_identifier_converts() {
        assert_eq!(title_to_identifier("brake fluid leak"), "BrakeFluidLeak");
        assert_eq!(title_to_identifier("Overheating"), "Overheating");
    }

    // -- Test helpers -------------------------------------------------------

    fn sample_risk() -> RiskDef {
        let mut r = RiskDef::new("ThermalRisk");
        r.item = "Battery subsystem".into();
        r.failure_mode = "Thermal runaway".into();
        r.failure_effect = "Battery fire".into();
        r.failure_cause = "Insufficient cooling".into();
        r.current_controls = "Temperature monitoring".into();
        r.category = Some(RiskCategory::Technical);
        r.status = Some(RiskStatus::Identified);
        r.severity = Some(3);
        r.occurrence = Some(3);
        r.detection = Some(3);
        r.rpn = Some(27);
        r.acceptance = Some(risk_acceptance(3, 3));
        r.owner = Some("thermal_team".into());
        r.notes = Some("Monitor battery temps".into());
        r.recommended_action = Some("Add thermal cutoff circuit".into());
        r
    }

    fn sample_assessment() -> RiskAssessment {
        RiskAssessment {
            risk_id: "ThermalRisk".into(),
            severity: 4,
            occurrence: 3,
            detection: 5,
            rpn: 60,
            notes: Some("Increased after field reports".into()),
            assessed_by: "charlie_assessor".into(),
        }
    }

    fn sample_mitigation() -> Mitigation {
        Mitigation {
            id: "MIT-001".into(),
            risk_id: "ThermalRisk".into(),
            description: "Add thermal cutoff circuit".into(),
            strategy: MitigationStrategy::Reduce,
            owner: Some("hw_team".into()),
            due_date: Some("2026-06-01".into()),
            status: Some(RiskStatus::Mitigating),
        }
    }

    fn make_def(name: &str, super_type: Option<&str>) -> Definition {
        Definition {
            kind: DefKind::Part,
            name: name.into(),
            super_type: super_type.map(|s| s.into()),
            span: Span::default(),
            has_body: true,
            param_count: 0,
            has_constraint_expr: false,
            has_return: false,
            visibility: None,
            short_name: None,
            doc: None,
            is_abstract: false,
            enum_members: Vec::new(),
            parent_def: None,
            body_start_byte: None,
            body_end_byte: None,
            qualified_name: None,
        }
    }

    fn make_usage(parent: &str, name: &str, value: &str) -> Usage {
        Usage {
            kind: "attribute".into(),
            name: name.into(),
            type_ref: None,
            span: Span::default(),
            direction: None,
            is_conjugated: false,
            parent_def: Some(parent.into()),
            multiplicity: None,
            value_expr: Some(value.into()),
            short_name: None,
            redefinition: None,
            subsets: None,
            qualified_name: None,
        }
    }

    fn build_test_model() -> Model {
        let mut model = Model::new("risks.sysml".into());

        model.definitions.push(make_def("ThermalRisk", None));
        model.usages.push(make_usage("ThermalRisk", "severity", "4"));
        model.usages.push(make_usage("ThermalRisk", "likelihood", "3"));
        model.usages.push(make_usage("ThermalRisk", "detectability", "2"));

        model.definitions.push(make_def("SupplyChainRisk", None));

        // Non-risk def (should be ignored)
        model.definitions.push(make_def("Vehicle", None));

        model
    }
}

/// Nonconformance and corrective action (CAPA) domain for SysML v2 models.
///
/// Provides types and functions for creating nonconformance reports (NCRs),
/// performing root cause analysis, tracking corrective/preventive actions,
/// and detecting trends or escalation conditions.  Integrates with
/// `sysml-core`'s record envelope system for audit-trail persistence and
/// the interactive wizard framework for guided root cause analysis.

use std::collections::BTreeMap;

use serde::Serialize;
use sysml_core::interactive::WizardStep;
use sysml_core::record::{generate_record_id, now_iso8601, RecordEnvelope, RecordMeta, RecordValue};

// =========================================================================
// Enums
// =========================================================================

/// Classification of a nonconformance by defect type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NonconformanceCategory {
    Dimensional,
    Material,
    Cosmetic,
    Functional,
    Workmanship,
    Documentation,
    Labeling,
    Packaging,
    Contamination,
    Software,
}

impl NonconformanceCategory {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Dimensional => "Dimensional",
            Self::Material => "Material",
            Self::Cosmetic => "Cosmetic",
            Self::Functional => "Functional",
            Self::Workmanship => "Workmanship",
            Self::Documentation => "Documentation",
            Self::Labeling => "Labeling",
            Self::Packaging => "Packaging",
            Self::Contamination => "Contamination",
            Self::Software => "Software",
        }
    }

    /// Lowercase identifier for serialization and grouping.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Dimensional => "dimensional",
            Self::Material => "material",
            Self::Cosmetic => "cosmetic",
            Self::Functional => "functional",
            Self::Workmanship => "workmanship",
            Self::Documentation => "documentation",
            Self::Labeling => "labeling",
            Self::Packaging => "packaging",
            Self::Contamination => "contamination",
            Self::Software => "software",
        }
    }

    /// All variants.
    pub fn all() -> &'static [Self] {
        &[
            Self::Dimensional,
            Self::Material,
            Self::Cosmetic,
            Self::Functional,
            Self::Workmanship,
            Self::Documentation,
            Self::Labeling,
            Self::Packaging,
            Self::Contamination,
            Self::Software,
        ]
    }
}

/// Severity classification of a nonconformance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityClass {
    Critical,
    Major,
    Minor,
    Observation,
}

impl SeverityClass {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Observation => "Observation",
        }
    }

    /// Lowercase identifier for serialization and grouping.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Observation => "observation",
        }
    }

    /// All variants in descending severity order.
    pub fn all() -> &'static [Self] {
        &[Self::Critical, Self::Major, Self::Minor, Self::Observation]
    }
}

/// Material review board disposition for a nonconforming item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    UseAsIs,
    Rework,
    Repair,
    Scrap,
    ReturnToVendor,
    SortAndScreen,
    Deviate,
}

impl Disposition {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::UseAsIs => "Use As Is",
            Self::Rework => "Rework",
            Self::Repair => "Repair",
            Self::Scrap => "Scrap",
            Self::ReturnToVendor => "Return to Vendor",
            Self::SortAndScreen => "Sort and Screen",
            Self::Deviate => "Deviate",
        }
    }
}

/// Type of corrective or preventive action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectiveActionType {
    DesignChange,
    ProcessChange,
    SupplierChange,
    ToolingChange,
    TrainingRetraining,
    ProcedureUpdate,
    InspectionEnhancement,
    Containment,
    NoActionRequired,
}

impl CorrectiveActionType {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::DesignChange => "Design Change",
            Self::ProcessChange => "Process Change",
            Self::SupplierChange => "Supplier Change",
            Self::ToolingChange => "Tooling Change",
            Self::TrainingRetraining => "Training/Retraining",
            Self::ProcedureUpdate => "Procedure Update",
            Self::InspectionEnhancement => "Inspection Enhancement",
            Self::Containment => "Containment",
            Self::NoActionRequired => "No Action Required",
        }
    }
}

/// Root cause analysis methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RootCauseMethod {
    FiveWhy,
    Fishbone,
    FaultTreeAnalysis,
    EightD,
    KepnerTregoe,
    IsIsNot,
    ParetoAnalysis,
}

impl RootCauseMethod {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::FiveWhy => "5 Why",
            Self::Fishbone => "Fishbone (Ishikawa)",
            Self::FaultTreeAnalysis => "Fault Tree Analysis",
            Self::EightD => "8D",
            Self::KepnerTregoe => "Kepner-Tregoe",
            Self::IsIsNot => "IS/IS NOT",
            Self::ParetoAnalysis => "Pareto Analysis",
        }
    }
}

/// Lifecycle status of a CAPA item (NCR or corrective action).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapaStatus {
    Initiated,
    Investigating,
    RootCauseIdentified,
    ActionPlanned,
    ActionImplemented,
    EffectivenessVerified,
    Closed,
    ClosedIneffective,
    Reopened,
}

impl CapaStatus {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Initiated => "Initiated",
            Self::Investigating => "Investigating",
            Self::RootCauseIdentified => "Root Cause Identified",
            Self::ActionPlanned => "Action Planned",
            Self::ActionImplemented => "Action Implemented",
            Self::EffectivenessVerified => "Effectiveness Verified",
            Self::Closed => "Closed",
            Self::ClosedIneffective => "Closed (Ineffective)",
            Self::Reopened => "Reopened",
        }
    }
}

// =========================================================================
// Domain structs
// =========================================================================

/// A nonconformance report (NCR).
#[derive(Debug, Clone, Serialize)]
pub struct Ncr {
    pub id: String,
    pub part_name: String,
    pub lot_id: Option<String>,
    pub supplier: Option<String>,
    pub category: NonconformanceCategory,
    pub severity_class: SeverityClass,
    pub description: String,
    pub disposition: Option<Disposition>,
    pub status: CapaStatus,
    pub created: String,
    pub owner: String,
}

/// Root cause analysis linked to an NCR.
#[derive(Debug, Clone, Serialize)]
pub struct RootCauseAnalysis {
    pub ncr_id: String,
    pub method: RootCauseMethod,
    pub findings: Vec<String>,
    pub root_cause: String,
}

/// A corrective or preventive action linked to an NCR.
#[derive(Debug, Clone, Serialize)]
pub struct CorrectiveAction {
    pub id: String,
    pub ncr_id: String,
    pub action_type: CorrectiveActionType,
    pub description: String,
    pub owner: String,
    pub due_date: String,
    pub status: CapaStatus,
    pub verification_ref: Option<String>,
}

/// A single data point for trend analysis.
#[derive(Debug, Clone, Serialize)]
pub struct TrendItem {
    pub category: NonconformanceCategory,
    pub severity_class: SeverityClass,
    pub count: usize,
    pub period: String,
}

// =========================================================================
// NCR creation
// =========================================================================

/// Create a new nonconformance report with a generated ID and `Initiated`
/// status.
///
/// The ID is generated via `sysml-core`'s record ID generator, providing a
/// unique, timestamp-based identifier suitable for audit trails.
pub fn create_ncr(
    part_name: &str,
    category: NonconformanceCategory,
    severity: SeverityClass,
    description: &str,
    owner: &str,
) -> Ncr {
    let id = generate_record_id("capa", "ncr", owner);
    Ncr {
        id,
        part_name: part_name.to_string(),
        lot_id: None,
        supplier: None,
        category,
        severity_class: severity,
        description: description.to_string(),
        disposition: None,
        status: CapaStatus::Initiated,
        created: now_iso8601(),
        owner: owner.to_string(),
    }
}

// =========================================================================
// Record creation (audit-trail persistence)
// =========================================================================

/// Create a [`RecordEnvelope`] for an NCR.
pub fn create_ncr_record(ncr: &Ncr, author: &str) -> RecordEnvelope {
    let id = generate_record_id("capa", "ncr", author);

    let mut refs = BTreeMap::new();
    refs.insert("ncr".to_string(), vec![ncr.id.clone()]);
    refs.insert("part".to_string(), vec![ncr.part_name.clone()]);

    let mut data = BTreeMap::new();
    data.insert(
        "part_name".into(),
        RecordValue::String(ncr.part_name.clone()),
    );
    data.insert(
        "category".into(),
        RecordValue::String(ncr.category.label().to_string()),
    );
    data.insert(
        "severity_class".into(),
        RecordValue::String(ncr.severity_class.label().to_string()),
    );
    data.insert(
        "description".into(),
        RecordValue::String(ncr.description.clone()),
    );
    data.insert(
        "status".into(),
        RecordValue::String(ncr.status.label().to_string()),
    );
    data.insert(
        "owner".into(),
        RecordValue::String(ncr.owner.clone()),
    );
    if let Some(lot) = &ncr.lot_id {
        data.insert("lot_id".into(), RecordValue::String(lot.clone()));
    }
    if let Some(supplier) = &ncr.supplier {
        data.insert("supplier".into(), RecordValue::String(supplier.clone()));
    }
    if let Some(disp) = &ncr.disposition {
        data.insert(
            "disposition".into(),
            RecordValue::String(disp.label().to_string()),
        );
    }

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "capa".into(),
            record_type: "ncr".into(),
            created: now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

/// Create a [`RecordEnvelope`] for a root cause analysis.
pub fn create_rca_record(rca: &RootCauseAnalysis, author: &str) -> RecordEnvelope {
    let id = generate_record_id("capa", "rca", author);

    let mut refs = BTreeMap::new();
    refs.insert("ncr".to_string(), vec![rca.ncr_id.clone()]);

    let mut data = BTreeMap::new();
    data.insert(
        "method".into(),
        RecordValue::String(rca.method.label().to_string()),
    );
    data.insert(
        "root_cause".into(),
        RecordValue::String(rca.root_cause.clone()),
    );
    data.insert(
        "findings".into(),
        RecordValue::Array(
            rca.findings
                .iter()
                .map(|f| RecordValue::String(f.clone()))
                .collect(),
        ),
    );

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "capa".into(),
            record_type: "rca".into(),
            created: now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

/// Create a [`RecordEnvelope`] for a corrective action.
pub fn create_action_record(action: &CorrectiveAction, author: &str) -> RecordEnvelope {
    let id = generate_record_id("capa", "action", author);

    let mut refs = BTreeMap::new();
    refs.insert("ncr".to_string(), vec![action.ncr_id.clone()]);
    refs.insert("action".to_string(), vec![action.id.clone()]);

    let mut data = BTreeMap::new();
    data.insert(
        "action_type".into(),
        RecordValue::String(action.action_type.label().to_string()),
    );
    data.insert(
        "description".into(),
        RecordValue::String(action.description.clone()),
    );
    data.insert(
        "owner".into(),
        RecordValue::String(action.owner.clone()),
    );
    data.insert(
        "due_date".into(),
        RecordValue::String(action.due_date.clone()),
    );
    data.insert(
        "status".into(),
        RecordValue::String(action.status.label().to_string()),
    );
    if let Some(vref) = &action.verification_ref {
        data.insert(
            "verification_ref".into(),
            RecordValue::String(vref.clone()),
        );
    }

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "capa".into(),
            record_type: "action".into(),
            created: now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

// =========================================================================
// Interactive wizards for root cause analysis
// =========================================================================

/// Build a 5 Why analysis wizard.
///
/// Returns six wizard steps: five successive "Why?" prompts that guide the
/// user from the observable symptom down to deeper causes, plus a final
/// step to summarize the identified root cause.
pub fn build_five_why_steps() -> Vec<WizardStep> {
    let mut steps = Vec::with_capacity(6);

    for i in 1..=5 {
        let prompt = match i {
            1 => "Why did the nonconformance occur? (1st Why)".to_string(),
            2 => "Why did that happen? (2nd Why)".to_string(),
            3 => "Why did that happen? (3rd Why)".to_string(),
            4 => "Why did that happen? (4th Why)".to_string(),
            5 => "Why did that happen? (5th Why)".to_string(),
            _ => unreachable!(),
        };

        let explanation = match i {
            1 => Some(
                "Start with the immediate, observable failure. Describe what \
                 went wrong in concrete terms."
                    .to_string(),
            ),
            2 => Some(
                "Look one level deeper. What condition or event allowed the \
                 first answer to occur?"
                    .to_string(),
            ),
            3 => Some(
                "Continue drilling down. Avoid jumping to solutions -- focus \
                 on understanding the causal chain."
                    .to_string(),
            ),
            4 => Some(
                "You should be approaching systemic or process-level causes \
                 at this point."
                    .to_string(),
            ),
            5 => Some(
                "The fifth why typically reveals the root cause: a management \
                 system, process, or cultural gap."
                    .to_string(),
            ),
            _ => unreachable!(),
        };

        let mut step = WizardStep::string(format!("why_{i}"), prompt);
        if let Some(text) = explanation {
            step = step.with_explanation(&text);
        }
        // All five whys are required for a complete analysis.
        steps.push(step);
    }

    // Summary step for the identified root cause.
    steps.push(
        WizardStep::string("root_cause", "Based on the 5 Whys, what is the root cause?")
            .with_explanation(
                "Synthesize the causal chain into a single, actionable root cause \
                 statement. This should describe the fundamental systemic issue, \
                 not just a symptom.",
            ),
    );

    steps
}

/// Build a Fishbone (Ishikawa) analysis wizard.
///
/// Returns seven wizard steps: one for each of the six standard Ishikawa
/// categories (Man, Machine, Method, Material, Measurement, Environment)
/// plus a final root cause summary.  Each category step is optional because
/// not every category contributes to every nonconformance.
pub fn build_fishbone_steps() -> Vec<WizardStep> {
    vec![
        WizardStep::string(
            "man",
            "Man (People): What personnel factors may have contributed?",
        )
        .with_explanation(
            "Consider training gaps, fatigue, skill level, staffing, \
             communication breakdowns, or procedural non-compliance.",
        )
        .optional(),
        WizardStep::string(
            "machine",
            "Machine (Equipment): What equipment factors may have contributed?",
        )
        .with_explanation(
            "Consider equipment age, maintenance history, calibration status, \
             capability, or environmental conditions affecting machinery.",
        )
        .optional(),
        WizardStep::string(
            "method",
            "Method (Process): What process or procedure factors may have contributed?",
        )
        .with_explanation(
            "Consider work instructions, standard operating procedures, process \
             validation status, or sequence of operations.",
        )
        .optional(),
        WizardStep::string(
            "material",
            "Material: What material or component factors may have contributed?",
        )
        .with_explanation(
            "Consider raw material quality, incoming inspection results, \
             supplier changes, storage conditions, or shelf life.",
        )
        .optional(),
        WizardStep::string(
            "measurement",
            "Measurement: What measurement or inspection factors may have contributed?",
        )
        .with_explanation(
            "Consider gage R&R, measurement uncertainty, inspection criteria, \
             sampling plans, or test method validation.",
        )
        .optional(),
        WizardStep::string(
            "environment",
            "Environment: What environmental factors may have contributed?",
        )
        .with_explanation(
            "Consider temperature, humidity, cleanliness, lighting, vibration, \
             or other ambient conditions in the work area.",
        )
        .optional(),
        WizardStep::string(
            "root_cause",
            "Based on the Fishbone analysis, what is the root cause?",
        )
        .with_explanation(
            "Synthesize findings from the contributing categories into a single, \
             actionable root cause statement.",
        ),
    ]
}

// =========================================================================
// Trend analysis
// =========================================================================

/// Group NCRs and produce trend counts.
///
/// The `group_by` parameter selects the grouping dimension:
/// - `"category"` — group by [`NonconformanceCategory`]
/// - `"severity"` — group by [`SeverityClass`]
/// - `"part"` — group by `part_name`
/// - `"supplier"` — group by `supplier` (NCRs without a supplier are skipped)
///
/// Each resulting [`TrendItem`] carries the category and severity from its
/// source NCR, a count of matching NCRs, and a `period` string equal to
/// the `group_by` key value.
pub fn trend_analysis(ncrs: &[Ncr], group_by: &str) -> Vec<TrendItem> {
    // We accumulate into a BTreeMap keyed by group value for deterministic
    // ordering. Each entry tracks (category, severity, count).
    let mut groups: BTreeMap<String, (NonconformanceCategory, SeverityClass, usize)> =
        BTreeMap::new();

    for ncr in ncrs {
        let key = match group_by {
            "category" => ncr.category.id().to_string(),
            "severity" => ncr.severity_class.id().to_string(),
            "part" => ncr.part_name.clone(),
            "supplier" => match &ncr.supplier {
                Some(s) => s.clone(),
                None => continue,
            },
            _ => continue,
        };

        let entry = groups
            .entry(key)
            .or_insert((ncr.category, ncr.severity_class, 0));
        entry.2 += 1;
    }

    groups
        .into_iter()
        .map(|(period, (category, severity_class, count))| TrendItem {
            category,
            severity_class,
            count,
            period,
        })
        .collect()
}

// =========================================================================
// Escalation detection
// =========================================================================

/// Check whether any failure pattern in the NCR set exceeds the given
/// threshold, indicating a systemic issue that requires escalation.
///
/// Two NCRs are considered "same failure" if they share both `part_name`
/// and `category`.  The `time_window_days` parameter is compared against
/// the NCR `created` timestamps (ISO 8601 date prefixes).
///
/// Returns a list of human-readable warning messages, one per escalation
/// trigger.
pub fn check_escalation(
    ncrs: &[Ncr],
    same_failure_threshold: usize,
    time_window_days: u32,
) -> Vec<String> {
    if ncrs.is_empty() || same_failure_threshold == 0 {
        return Vec::new();
    }

    // Group by (part_name, category).
    let mut groups: BTreeMap<(String, String), Vec<&Ncr>> = BTreeMap::new();
    for ncr in ncrs {
        let key = (ncr.part_name.clone(), ncr.category.id().to_string());
        groups.entry(key).or_default().push(ncr);
    }

    let mut warnings = Vec::new();

    for ((part, cat), group_ncrs) in &groups {
        // Extract dates from ISO 8601 timestamps (first 10 chars: YYYY-MM-DD).
        let mut dates: Vec<&str> = group_ncrs
            .iter()
            .filter_map(|n| {
                if n.created.len() >= 10 {
                    Some(&n.created[..10])
                } else {
                    None
                }
            })
            .collect();
        dates.sort();

        if dates.is_empty() {
            continue;
        }

        // Sliding window: count how many NCRs fall within `time_window_days`
        // of each starting date.
        let window_secs = time_window_days as i64 * 86400;

        for (i, &start_date) in dates.iter().enumerate() {
            let start_epoch = match date_to_epoch(start_date) {
                Some(e) => e,
                None => continue,
            };

            let mut count = 0usize;
            for &d in &dates[i..] {
                let epoch = match date_to_epoch(d) {
                    Some(e) => e,
                    None => continue,
                };
                if epoch - start_epoch <= window_secs {
                    count += 1;
                } else {
                    break; // dates are sorted, no need to check further
                }
            }

            if count >= same_failure_threshold {
                warnings.push(format!(
                    "ESCALATION: {count} NCRs for part '{part}' category '{cat}' \
                     within {time_window_days} days (threshold: {same_failure_threshold})",
                ));
                // Only report once per group.
                break;
            }
        }
    }

    warnings
}

/// Parse a `YYYY-MM-DD` date string into seconds since Unix epoch.
///
/// Returns `None` for malformed dates.  Uses a simplified calculation
/// sufficient for day-granularity comparisons.
fn date_to_epoch(date: &str) -> Option<i64> {
    if date.len() < 10 {
        return None;
    }
    let year: i64 = date[0..4].parse().ok()?;
    let month: i64 = date[5..7].parse().ok()?;
    let day: i64 = date[8..10].parse().ok()?;

    // Simplified days-since-epoch using the same algorithm as sysml-core.
    // We compute days from a reference and multiply by 86400.
    let m = if month <= 2 { month + 9 } else { month - 3 };
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y / 400 } else { (y - 399) / 400 };
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    Some(days * 86400)
}

// =========================================================================
// Disposition
// =========================================================================

/// Set the disposition on an NCR and advance its status to `Investigating`.
///
/// This reflects the material review board (MRB) decision on what to do
/// with the nonconforming material.
pub fn disposition_ncr(ncr: &mut Ncr, disposition: Disposition) {
    ncr.disposition = Some(disposition);
    ncr.status = CapaStatus::Investigating;
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper -----------------------------------------------------------

    fn sample_ncr() -> Ncr {
        Ncr {
            id: "NCR-001".into(),
            part_name: "BrakeRotor".into(),
            lot_id: Some("LOT-2026-03".into()),
            supplier: Some("AcmeCasting".into()),
            category: NonconformanceCategory::Dimensional,
            severity_class: SeverityClass::Major,
            description: "OD out of tolerance by 0.5mm".into(),
            disposition: None,
            status: CapaStatus::Initiated,
            created: "2026-03-01T10:00:00Z".into(),
            owner: "alice".into(),
        }
    }

    fn sample_rca() -> RootCauseAnalysis {
        RootCauseAnalysis {
            ncr_id: "NCR-001".into(),
            method: RootCauseMethod::FiveWhy,
            findings: vec![
                "Rotor OD measured 250.5mm vs 250.0mm spec".into(),
                "Lathe tool offset drifted during shift".into(),
                "No mid-shift tool offset verification".into(),
                "SOP does not require periodic re-check".into(),
                "Process validation gap in turning operations".into(),
            ],
            root_cause: "Missing periodic tool offset verification in turning SOP".into(),
        }
    }

    fn sample_action() -> CorrectiveAction {
        CorrectiveAction {
            id: "CA-001".into(),
            ncr_id: "NCR-001".into(),
            action_type: CorrectiveActionType::ProcedureUpdate,
            description: "Add mid-shift tool offset verification to turning SOP".into(),
            owner: "bob".into(),
            due_date: "2026-04-01".into(),
            status: CapaStatus::ActionPlanned,
            verification_ref: Some("Vehicle::Tests::RotorDimCheck".into()),
        }
    }

    // -- Enum label tests -------------------------------------------------

    #[test]
    fn nonconformance_category_labels() {
        assert_eq!(NonconformanceCategory::Dimensional.label(), "Dimensional");
        assert_eq!(NonconformanceCategory::Software.label(), "Software");
        assert_eq!(NonconformanceCategory::Contamination.id(), "contamination");
    }

    #[test]
    fn nonconformance_category_all_count() {
        assert_eq!(NonconformanceCategory::all().len(), 10);
    }

    #[test]
    fn severity_class_labels() {
        assert_eq!(SeverityClass::Critical.label(), "Critical");
        assert_eq!(SeverityClass::Observation.label(), "Observation");
        assert_eq!(SeverityClass::Minor.id(), "minor");
    }

    #[test]
    fn severity_class_all_count() {
        assert_eq!(SeverityClass::all().len(), 4);
    }

    #[test]
    fn disposition_labels() {
        assert_eq!(Disposition::UseAsIs.label(), "Use As Is");
        assert_eq!(Disposition::ReturnToVendor.label(), "Return to Vendor");
        assert_eq!(Disposition::SortAndScreen.label(), "Sort and Screen");
    }

    #[test]
    fn corrective_action_type_labels() {
        assert_eq!(CorrectiveActionType::DesignChange.label(), "Design Change");
        assert_eq!(
            CorrectiveActionType::TrainingRetraining.label(),
            "Training/Retraining"
        );
        assert_eq!(
            CorrectiveActionType::NoActionRequired.label(),
            "No Action Required"
        );
    }

    #[test]
    fn root_cause_method_labels() {
        assert_eq!(RootCauseMethod::FiveWhy.label(), "5 Why");
        assert_eq!(RootCauseMethod::Fishbone.label(), "Fishbone (Ishikawa)");
        assert_eq!(RootCauseMethod::EightD.label(), "8D");
        assert_eq!(RootCauseMethod::IsIsNot.label(), "IS/IS NOT");
    }

    #[test]
    fn capa_status_labels() {
        assert_eq!(CapaStatus::Initiated.label(), "Initiated");
        assert_eq!(CapaStatus::RootCauseIdentified.label(), "Root Cause Identified");
        assert_eq!(CapaStatus::ClosedIneffective.label(), "Closed (Ineffective)");
        assert_eq!(CapaStatus::Reopened.label(), "Reopened");
    }

    // -- create_ncr -------------------------------------------------------

    #[test]
    fn create_ncr_generates_id() {
        let ncr = create_ncr(
            "Widget",
            NonconformanceCategory::Functional,
            SeverityClass::Critical,
            "Widget fails under load",
            "charlie",
        );
        assert!(ncr.id.starts_with("capa-ncr-"));
        assert!(ncr.id.contains("charlie"));
    }

    #[test]
    fn create_ncr_initial_status() {
        let ncr = create_ncr(
            "Bracket",
            NonconformanceCategory::Material,
            SeverityClass::Minor,
            "Wrong alloy",
            "dave",
        );
        assert_eq!(ncr.status, CapaStatus::Initiated);
        assert_eq!(ncr.part_name, "Bracket");
        assert_eq!(ncr.category, NonconformanceCategory::Material);
        assert_eq!(ncr.severity_class, SeverityClass::Minor);
        assert!(ncr.disposition.is_none());
        assert!(ncr.lot_id.is_none());
        assert!(ncr.supplier.is_none());
    }

    // -- Record creation --------------------------------------------------

    #[test]
    fn ncr_record_structure() {
        let ncr = sample_ncr();
        let rec = create_ncr_record(&ncr, "alice");

        assert_eq!(rec.meta.tool, "capa");
        assert_eq!(rec.meta.record_type, "ncr");
        assert_eq!(rec.meta.author, "alice");
        assert!(rec.meta.id.starts_with("capa-ncr-"));

        assert!(rec.refs.contains_key("ncr"));
        assert!(rec.refs.contains_key("part"));
        assert_eq!(rec.refs["part"], vec!["BrakeRotor".to_string()]);

        assert_eq!(
            rec.data.get("category"),
            Some(&RecordValue::String("Dimensional".into()))
        );
        assert_eq!(
            rec.data.get("severity_class"),
            Some(&RecordValue::String("Major".into()))
        );
        assert!(rec.data.contains_key("description"));
        assert!(rec.data.contains_key("status"));
        assert!(rec.data.contains_key("owner"));
    }

    #[test]
    fn ncr_record_includes_optional_fields() {
        let ncr = sample_ncr();
        let rec = create_ncr_record(&ncr, "alice");

        assert_eq!(
            rec.data.get("lot_id"),
            Some(&RecordValue::String("LOT-2026-03".into()))
        );
        assert_eq!(
            rec.data.get("supplier"),
            Some(&RecordValue::String("AcmeCasting".into()))
        );
    }

    #[test]
    fn ncr_record_omits_none_fields() {
        let mut ncr = sample_ncr();
        ncr.lot_id = None;
        ncr.supplier = None;
        ncr.disposition = None;

        let rec = create_ncr_record(&ncr, "alice");
        assert!(!rec.data.contains_key("lot_id"));
        assert!(!rec.data.contains_key("supplier"));
        assert!(!rec.data.contains_key("disposition"));
    }

    #[test]
    fn rca_record_structure() {
        let rca = sample_rca();
        let rec = create_rca_record(&rca, "alice");

        assert_eq!(rec.meta.tool, "capa");
        assert_eq!(rec.meta.record_type, "rca");
        assert!(rec.refs.contains_key("ncr"));
        assert_eq!(rec.refs["ncr"], vec!["NCR-001".to_string()]);

        assert_eq!(
            rec.data.get("method"),
            Some(&RecordValue::String("5 Why".into()))
        );
        assert!(rec.data.contains_key("root_cause"));

        // Findings should be an array.
        match rec.data.get("findings") {
            Some(RecordValue::Array(arr)) => assert_eq!(arr.len(), 5),
            other => panic!("expected findings array, got {other:?}"),
        }
    }

    #[test]
    fn action_record_structure() {
        let action = sample_action();
        let rec = create_action_record(&action, "bob");

        assert_eq!(rec.meta.tool, "capa");
        assert_eq!(rec.meta.record_type, "action");
        assert!(rec.refs.contains_key("ncr"));
        assert!(rec.refs.contains_key("action"));

        assert_eq!(
            rec.data.get("action_type"),
            Some(&RecordValue::String("Procedure Update".into()))
        );
        assert_eq!(
            rec.data.get("due_date"),
            Some(&RecordValue::String("2026-04-01".into()))
        );
        assert_eq!(
            rec.data.get("verification_ref"),
            Some(&RecordValue::String(
                "Vehicle::Tests::RotorDimCheck".into()
            ))
        );
    }

    #[test]
    fn action_record_omits_none_verification() {
        let mut action = sample_action();
        action.verification_ref = None;
        let rec = create_action_record(&action, "bob");
        assert!(!rec.data.contains_key("verification_ref"));
    }

    // -- Wizard steps -----------------------------------------------------

    #[test]
    fn five_why_step_count() {
        let steps = build_five_why_steps();
        assert_eq!(steps.len(), 6); // 5 whys + root cause summary
    }

    #[test]
    fn five_why_step_ids() {
        let steps = build_five_why_steps();
        let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["why_1", "why_2", "why_3", "why_4", "why_5", "root_cause"]
        );
    }

    #[test]
    fn five_why_all_required() {
        let steps = build_five_why_steps();
        assert!(steps.iter().all(|s| s.required));
    }

    #[test]
    fn five_why_has_explanations() {
        let steps = build_five_why_steps();
        assert!(steps.iter().all(|s| s.explanation.is_some()));
    }

    #[test]
    fn fishbone_step_count() {
        let steps = build_fishbone_steps();
        assert_eq!(steps.len(), 7); // 6 Ishikawa categories + root cause
    }

    #[test]
    fn fishbone_step_ids() {
        let steps = build_fishbone_steps();
        let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "man",
                "machine",
                "method",
                "material",
                "measurement",
                "environment",
                "root_cause",
            ]
        );
    }

    #[test]
    fn fishbone_categories_optional_root_cause_required() {
        let steps = build_fishbone_steps();
        // First 6 are optional (not every Ishikawa category applies).
        for step in &steps[..6] {
            assert!(!step.required, "category step '{}' should be optional", step.id);
        }
        // Root cause summary is required.
        assert!(steps[6].required);
    }

    #[test]
    fn fishbone_has_explanations() {
        let steps = build_fishbone_steps();
        assert!(steps.iter().all(|s| s.explanation.is_some()));
    }

    // -- Trend analysis ---------------------------------------------------

    #[test]
    fn trend_by_category() {
        let ncrs = vec![
            sample_ncr(),
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n
            },
            {
                let mut n = sample_ncr();
                n.id = "NCR-003".into();
                n.category = NonconformanceCategory::Functional;
                n
            },
        ];

        let trends = trend_analysis(&ncrs, "category");
        assert_eq!(trends.len(), 2);

        let dim = trends.iter().find(|t| t.period == "dimensional").unwrap();
        assert_eq!(dim.count, 2);
        assert_eq!(dim.category, NonconformanceCategory::Dimensional);

        let func = trends.iter().find(|t| t.period == "functional").unwrap();
        assert_eq!(func.count, 1);
    }

    #[test]
    fn trend_by_severity() {
        let ncrs = vec![
            sample_ncr(),
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n.severity_class = SeverityClass::Critical;
                n
            },
        ];

        let trends = trend_analysis(&ncrs, "severity");
        assert_eq!(trends.len(), 2);

        let major = trends.iter().find(|t| t.period == "major").unwrap();
        assert_eq!(major.count, 1);

        let critical = trends.iter().find(|t| t.period == "critical").unwrap();
        assert_eq!(critical.count, 1);
    }

    #[test]
    fn trend_by_part() {
        let ncrs = vec![
            sample_ncr(),
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n.part_name = "CalliperHousing".into();
                n
            },
        ];

        let trends = trend_analysis(&ncrs, "part");
        assert_eq!(trends.len(), 2);

        let rotor = trends.iter().find(|t| t.period == "BrakeRotor").unwrap();
        assert_eq!(rotor.count, 1);
    }

    #[test]
    fn trend_by_supplier_skips_none() {
        let ncrs = vec![
            sample_ncr(),
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n.supplier = None;
                n
            },
        ];

        let trends = trend_analysis(&ncrs, "supplier");
        assert_eq!(trends.len(), 1);
        assert_eq!(trends[0].period, "AcmeCasting");
        assert_eq!(trends[0].count, 1);
    }

    #[test]
    fn trend_empty_input() {
        let trends = trend_analysis(&[], "category");
        assert!(trends.is_empty());
    }

    #[test]
    fn trend_unknown_group_by() {
        let ncrs = vec![sample_ncr()];
        let trends = trend_analysis(&ncrs, "nonexistent");
        assert!(trends.is_empty());
    }

    // -- Escalation detection ---------------------------------------------

    #[test]
    fn escalation_triggers_on_threshold() {
        let ncrs = vec![
            {
                let mut n = sample_ncr();
                n.created = "2026-03-01T10:00:00Z".into();
                n
            },
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n.created = "2026-03-05T10:00:00Z".into();
                n
            },
            {
                let mut n = sample_ncr();
                n.id = "NCR-003".into();
                n.created = "2026-03-10T10:00:00Z".into();
                n
            },
        ];

        let warnings = check_escalation(&ncrs, 3, 30);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("ESCALATION"));
        assert!(warnings[0].contains("3 NCRs"));
        assert!(warnings[0].contains("BrakeRotor"));
    }

    #[test]
    fn escalation_no_trigger_below_threshold() {
        let ncrs = vec![
            sample_ncr(),
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n.created = "2026-03-05T10:00:00Z".into();
                n
            },
        ];

        let warnings = check_escalation(&ncrs, 3, 30);
        assert!(warnings.is_empty());
    }

    #[test]
    fn escalation_respects_time_window() {
        let ncrs = vec![
            {
                let mut n = sample_ncr();
                n.created = "2026-01-01T10:00:00Z".into();
                n
            },
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n.created = "2026-06-01T10:00:00Z".into();
                n
            },
            {
                let mut n = sample_ncr();
                n.id = "NCR-003".into();
                n.created = "2026-12-01T10:00:00Z".into();
                n
            },
        ];

        // 3 NCRs but spread over 11 months, well outside a 30-day window.
        let warnings = check_escalation(&ncrs, 3, 30);
        assert!(warnings.is_empty());
    }

    #[test]
    fn escalation_empty_input() {
        let warnings = check_escalation(&[], 3, 30);
        assert!(warnings.is_empty());
    }

    #[test]
    fn escalation_zero_threshold() {
        let ncrs = vec![sample_ncr()];
        let warnings = check_escalation(&ncrs, 0, 30);
        assert!(warnings.is_empty());
    }

    #[test]
    fn escalation_different_parts_no_trigger() {
        let ncrs = vec![
            {
                let mut n = sample_ncr();
                n.created = "2026-03-01T10:00:00Z".into();
                n
            },
            {
                let mut n = sample_ncr();
                n.id = "NCR-002".into();
                n.part_name = "OtherPart".into();
                n.created = "2026-03-02T10:00:00Z".into();
                n
            },
        ];

        let warnings = check_escalation(&ncrs, 2, 30);
        assert_eq!(warnings.len(), 0);
    }

    // -- Disposition ------------------------------------------------------

    #[test]
    fn disposition_sets_value_and_status() {
        let mut ncr = sample_ncr();
        assert!(ncr.disposition.is_none());
        assert_eq!(ncr.status, CapaStatus::Initiated);

        disposition_ncr(&mut ncr, Disposition::Rework);

        assert_eq!(ncr.disposition, Some(Disposition::Rework));
        assert_eq!(ncr.status, CapaStatus::Investigating);
    }

    #[test]
    fn disposition_overwrite() {
        let mut ncr = sample_ncr();
        disposition_ncr(&mut ncr, Disposition::Scrap);
        assert_eq!(ncr.disposition, Some(Disposition::Scrap));

        disposition_ncr(&mut ncr, Disposition::UseAsIs);
        assert_eq!(ncr.disposition, Some(Disposition::UseAsIs));
        assert_eq!(ncr.status, CapaStatus::Investigating);
    }

    // -- date_to_epoch helper ---------------------------------------------

    #[test]
    fn date_epoch_known_dates() {
        // 1970-01-01 should be 0.
        assert_eq!(date_to_epoch("1970-01-01"), Some(0));
        // 2000-01-01 = day 10957, so 10957 * 86400.
        assert_eq!(date_to_epoch("2000-01-01"), Some(10957 * 86400));
    }

    #[test]
    fn date_epoch_malformed() {
        assert_eq!(date_to_epoch("bad"), None);
        assert_eq!(date_to_epoch(""), None);
    }

    // -- Serde compatibility ----------------------------------------------

    #[test]
    fn ncr_serializes() {
        let ncr = sample_ncr();
        let json = serde_json::to_string(&ncr).unwrap();
        assert!(json.contains("\"part_name\""));
        assert!(json.contains("\"dimensional\""));
        assert!(json.contains("\"major\""));
    }

    #[test]
    fn rca_serializes() {
        let rca = sample_rca();
        let json = serde_json::to_string(&rca).unwrap();
        assert!(json.contains("\"five_why\""));
        assert!(json.contains("\"root_cause\""));
    }

    #[test]
    fn action_serializes() {
        let action = sample_action();
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"procedure_update\""));
        assert!(json.contains("\"action_planned\""));
    }

    #[test]
    fn trend_item_serializes() {
        let item = TrendItem {
            category: NonconformanceCategory::Material,
            severity_class: SeverityClass::Minor,
            count: 5,
            period: "2026-Q1".into(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"material\""));
        assert!(json.contains("\"minor\""));
        assert!(json.contains("\"count\":5"));
    }

    // -- Record round-trip ------------------------------------------------

    #[test]
    fn ncr_record_round_trips_toml() {
        let ncr = sample_ncr();
        let rec = create_ncr_record(&ncr, "alice");
        let toml = rec.to_toml_string();
        let parsed = RecordEnvelope::from_toml_str(&toml).unwrap();
        assert_eq!(parsed.meta.tool, "capa");
        assert_eq!(parsed.meta.record_type, "ncr");
        assert_eq!(parsed.data, rec.data);
    }

    #[test]
    fn rca_record_round_trips_toml() {
        let rca = sample_rca();
        let rec = create_rca_record(&rca, "alice");
        let toml = rec.to_toml_string();
        let parsed = RecordEnvelope::from_toml_str(&toml).unwrap();
        assert_eq!(parsed.meta.record_type, "rca");
        assert_eq!(parsed.data, rec.data);
    }
}

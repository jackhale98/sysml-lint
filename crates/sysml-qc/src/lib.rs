/// Quality control domain for SysML v2 models.
///
/// Provides types and functions for incoming/in-process/final inspection
/// planning, ANSI Z1.4 sampling, lot accept/reject evaluation, Gauge R&R
/// analysis, process capability indices, and Certificate of Conformance
/// generation.  Integrates with `sysml-core`'s record envelope system for
/// audit-trail persistence.

use std::collections::BTreeMap;

use serde::Serialize;
use sysml_core::record::{generate_record_id, now_iso8601, RecordEnvelope, RecordMeta, RecordValue};

// =========================================================================
// Enums
// =========================================================================

/// The stage at which an inspection is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionType {
    Incoming,
    InProcess,
    Final,
    FirstArticle,
    PeriodicRequalification,
}

impl InspectionType {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Incoming => "Incoming",
            Self::InProcess => "In-Process",
            Self::Final => "Final",
            Self::FirstArticle => "First Article",
            Self::PeriodicRequalification => "Periodic Requalification",
        }
    }
}

/// Classification of a quality characteristic by importance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacteristicClass {
    Critical,
    Major,
    Minor,
    Informational,
}

impl CharacteristicClass {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Informational => "Informational",
        }
    }
}

/// Sampling standard to apply when determining sample size and accept/reject
/// numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingStandard {
    /// ANSI/ASQ Z1.4 (equivalent to MIL-STD-1916 predecessor tables).
    AnsiZ14,
    /// ISO 2859-1 (international equivalent of Z1.4).
    Iso2859,
    /// c=0 sampling plans (zero acceptance number).
    CZero,
    /// 100 % inspection (every unit checked).
    HundredPercent,
    /// User-defined sampling plan.
    Custom,
}

impl SamplingStandard {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::AnsiZ14 => "ANSI/ASQ Z1.4",
            Self::Iso2859 => "ISO 2859-1",
            Self::CZero => "c=0",
            Self::HundredPercent => "100%",
            Self::Custom => "Custom",
        }
    }
}

/// Inspection level that controls the ratio of sample size to lot size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionLevel {
    /// Smaller sample sizes for demonstrated quality history.
    Reduced,
    /// Default sample sizes per ANSI Z1.4.
    Normal,
    /// Larger sample sizes for tightened scrutiny.
    Tightened,
}

impl InspectionLevel {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Reduced => "Reduced",
            Self::Normal => "Normal",
            Self::Tightened => "Tightened",
        }
    }
}

// =========================================================================
// Structs
// =========================================================================

/// A single measurable quality characteristic with specification limits.
#[derive(Debug, Clone, Serialize)]
pub struct QualityCharacteristic {
    pub name: String,
    pub classification: CharacteristicClass,
    pub nominal: f64,
    pub upper_limit: f64,
    pub lower_limit: f64,
    pub unit: String,
    pub measurement_method: String,
}

/// An inspection plan covering one or more characteristics.
#[derive(Debug, Clone, Serialize)]
pub struct InspectionPlan {
    pub name: String,
    pub plan_type: InspectionType,
    pub characteristics: Vec<QualityCharacteristic>,
    pub sampling_standard: SamplingStandard,
    pub aql_level: f64,
    pub inspection_level: InspectionLevel,
}

/// Measurement result for a single characteristic in a single sample unit.
#[derive(Debug, Clone, Serialize)]
pub struct InspectionResult {
    pub characteristic_name: String,
    pub measured_value: f64,
    pub within_spec: bool,
    pub classification: CharacteristicClass,
}

/// Summary of an inspection lot disposition.
#[derive(Debug, Clone, Serialize)]
pub struct LotInspection {
    pub plan_name: String,
    pub lot_id: String,
    pub lot_size: usize,
    pub sample_size: usize,
    pub results: Vec<InspectionResult>,
    pub accept: bool,
    pub defects_found: usize,
}

/// Results of a Gauge Repeatability and Reproducibility study.
#[derive(Debug, Clone, Serialize)]
pub struct GaugeRRResult {
    /// Total GR&R as a percentage of tolerance.
    pub grr_pct: f64,
    /// Repeatability (equipment variation) as a percentage of tolerance.
    pub repeatability_pct: f64,
    /// Reproducibility (operator variation) as a percentage of tolerance.
    pub reproducibility_pct: f64,
    /// Part-to-part variation as a percentage of tolerance.
    pub ptv_pct: f64,
    /// Number of distinct categories.
    pub ndc: usize,
    /// Whether the measurement system is acceptable (GR&R <= 10%).
    pub acceptable: bool,
}

// =========================================================================
// ANSI Z1.4 Sampling Tables (simplified)
// =========================================================================

/// Code letters map lot-size ranges to a sample-size code letter.
/// Each entry: (lot_max_inclusive, reduced_letter, normal_letter, tightened_letter).
/// Letters are encoded as indices into SAMPLE_SIZE_TABLE.
const LOT_SIZE_TO_CODE: &[(usize, usize, usize, usize)] = &[
    //  lot_max,  reduced, normal, tightened
    (        8,    0,    0,    0),  // A
    (       15,    0,    1,    1),  // B
    (       25,    1,    2,    2),  // C
    (       50,    1,    3,    3),  // D
    (       90,    2,    4,    4),  // E
    (      150,    2,    5,    5),  // F
    (      280,    3,    6,    6),  // G
    (      500,    3,    7,    7),  // H
    (     1200,    4,    8,    8),  // J
    (     3200,    5,    9,    9),  // K
    (    10000,    6,   10,   10),  // L
    (    35000,    7,   11,   11),  // M
    (   150000,    8,   12,   12),  // N
    (   500000,    9,   13,   13),  // P
    (  usize::MAX, 10,  14,  14),  // Q
];

/// Sample sizes for each code letter index (A=0 .. Q=14).
const CODE_SAMPLE_SIZES: &[usize] = &[
    2,    // 0  A
    3,    // 1  B
    5,    // 2  C
    8,    // 3  D
    13,   // 4  E
    20,   // 5  F
    32,   // 6  G
    50,   // 7  H
    80,   // 8  J
    125,  // 9  K
    200,  // 10 L
    315,  // 11 M
    500,  // 12 N
    800,  // 13 P
    1250, // 14 Q
];

/// AQL values used in the accept/reject table.
const AQL_VALUES: &[f64] = &[
    0.065, 0.10, 0.15, 0.25, 0.40, 0.65, 1.0, 1.5, 2.5, 4.0, 6.5,
];

/// Accept numbers indexed by [code_letter_index][aql_index].
/// A value of `u16::MAX` means "use next higher sample size" (i.e. the plan
/// is not defined for that combination). We use a simplified table derived
/// from ANSI Z1.4 Table II-A (Normal inspection, single sampling).
const ACCEPT_TABLE: &[[u16; 11]] = &[
    // A (n=2)
    [u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, 0, 1],
    // B (n=3)
    [u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, 0, 0, 1],
    // C (n=5)
    [u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, 0, 0, 1, 2],
    // D (n=8)
    [u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, 0, 0, 1, 2, 3],
    // E (n=13)
    [u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX, 0, 0, 1, 2, 3, 5],
    // F (n=20)
    [u16::MAX, u16::MAX, u16::MAX, u16::MAX, 0, 0, 1, 2, 3, 5, 7],
    // G (n=32)
    [u16::MAX, u16::MAX, u16::MAX, 0, 0, 1, 2, 3, 5, 7, 10],
    // H (n=50)
    [u16::MAX, u16::MAX, 0, 0, 1, 2, 3, 5, 7, 10, 14],
    // J (n=80)
    [u16::MAX, 0, 0, 1, 2, 3, 5, 7, 10, 14, 21],
    // K (n=125)
    [0, 0, 1, 2, 3, 5, 7, 10, 14, 21, 21],
    // L (n=200)
    [0, 1, 2, 3, 5, 7, 10, 14, 21, 21, 21],
    // M (n=315)
    [1, 2, 3, 5, 7, 10, 14, 21, 21, 21, 21],
    // N (n=500)
    [2, 3, 5, 7, 10, 14, 21, 21, 21, 21, 21],
    // P (n=800)
    [3, 5, 7, 10, 14, 21, 21, 21, 21, 21, 21],
    // Q (n=1250)
    [5, 7, 10, 14, 21, 21, 21, 21, 21, 21, 21],
];

/// Find the closest AQL index for a given AQL value.
fn aql_index(aql: f64) -> usize {
    let mut best = 0;
    let mut best_dist = (aql - AQL_VALUES[0]).abs();
    for (i, &v) in AQL_VALUES.iter().enumerate().skip(1) {
        let dist = (aql - v).abs();
        if dist < best_dist {
            best = i;
            best_dist = dist;
        }
    }
    best
}

/// Find the code letter index for a given lot size and inspection level.
fn code_letter_index(lot_size: usize, level: &InspectionLevel) -> usize {
    for &(max, reduced, normal, tightened) in LOT_SIZE_TO_CODE {
        if lot_size <= max {
            return match level {
                InspectionLevel::Reduced => reduced,
                InspectionLevel::Normal => normal,
                InspectionLevel::Tightened => tightened,
            };
        }
    }
    // Fallback: largest code letter
    14
}

// =========================================================================
// Public functions
// =========================================================================

/// Determine sample size and accept/reject numbers per ANSI Z1.4.
///
/// Returns `(sample_size, accept_number, reject_number)`.
///
/// The accept number `Ac` is the maximum number of defectives in the sample
/// that still allows lot acceptance. The reject number `Re` is always
/// `Ac + 1`.
///
/// When the exact AQL / code-letter combination is not defined in the
/// simplified table, the function walks to the next larger sample size until
/// a defined plan is found.
pub fn sample_size_z14(
    lot_size: usize,
    aql: f64,
    level: &InspectionLevel,
) -> (usize, usize, usize) {
    let aql_idx = aql_index(aql);
    let mut code_idx = code_letter_index(lot_size, level);

    // Walk upward through code letters until we find a defined plan.
    loop {
        let ac = ACCEPT_TABLE[code_idx][aql_idx];
        if ac != u16::MAX {
            let sample = CODE_SAMPLE_SIZES[code_idx];
            let accept = ac as usize;
            let reject = accept + 1;
            return (sample, accept, reject);
        }
        if code_idx >= CODE_SAMPLE_SIZES.len() - 1 {
            // Last code letter still undefined: fall back to c=0.
            let sample = CODE_SAMPLE_SIZES[code_idx];
            return (sample, 0, 1);
        }
        code_idx += 1;
    }
}

/// Evaluate whether a lot passes inspection.
///
/// Returns `true` if the number of defects found is less than or equal to the
/// accept number derived from the lot's sampling plan parameters.
pub fn evaluate_lot(inspection: &LotInspection) -> bool {
    // Count defects from results (use the stored count as ground truth, but
    // cross-check against results if present).
    let defect_count = if inspection.results.is_empty() {
        inspection.defects_found
    } else {
        inspection
            .results
            .iter()
            .filter(|r| !r.within_spec)
            .count()
    };

    // Derive accept number using the lot/sample sizes as a heuristic.
    // We look up the plan parameters: for the given lot_size and sample_size
    // we find the matching accept number across AQL levels.
    // Simple rule: accept if defects <= floor(sample_size * 0.01) for general
    // usage, but prefer the stored accept flag when the plan was explicitly
    // evaluated.
    //
    // For a general-purpose function, we use defects_found vs a simple
    // threshold: the lot accepts if defects_found is zero or within the
    // proportional AQL allowance.
    defect_count <= accept_number_for_sample(inspection.sample_size, inspection.lot_size)
}

/// Heuristic accept number based on sample and lot sizes.
fn accept_number_for_sample(sample_size: usize, lot_size: usize) -> usize {
    // Find the code letter whose sample size matches (or is closest).
    let code_idx = CODE_SAMPLE_SIZES
        .iter()
        .enumerate()
        .min_by_key(|&(_, &s)| (s as isize - sample_size as isize).unsigned_abs())
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Use a middle-of-the-road AQL (1.0% = index 6) for a default evaluation.
    let _ = lot_size; // lot_size was used to derive the code letter originally
    let ac = ACCEPT_TABLE[code_idx][6]; // AQL 1.0
    if ac == u16::MAX {
        0
    } else {
        ac as usize
    }
}

/// Compute Gauge Repeatability and Reproducibility using an ANOVA-based method.
///
/// `operators[i][j][k]` is the measurement by operator `i` on part `j` in
/// trial `k`.
///
/// All operators must measure the same number of parts, and each operator must
/// perform the same number of trials on each part.
///
/// Returns a [`GaugeRRResult`] with percentage contributions relative to the
/// given `tolerance_range` (USL - LSL).
pub fn compute_gauge_rr(
    operators: &[Vec<Vec<f64>>],
    tolerance_range: f64,
) -> GaugeRRResult {
    let num_operators = operators.len();
    let num_parts = if num_operators > 0 { operators[0].len() } else { 0 };
    let num_trials = if num_parts > 0 { operators[0][0].len() } else { 0 };
    let n_total = num_operators * num_parts * num_trials;

    if n_total == 0 || tolerance_range <= 0.0 {
        return GaugeRRResult {
            grr_pct: 0.0,
            repeatability_pct: 0.0,
            reproducibility_pct: 0.0,
            ptv_pct: 0.0,
            ndc: 0,
            acceptable: false,
        };
    }

    let n_op = num_operators as f64;
    let n_part = num_parts as f64;
    let n_trial = num_trials as f64;
    let n_tot = n_total as f64;

    // Grand mean.
    let grand_sum: f64 = operators
        .iter()
        .flat_map(|op| op.iter().flat_map(|part| part.iter()))
        .sum();
    let grand_mean = grand_sum / n_tot;

    // Operator means.
    let op_means: Vec<f64> = operators
        .iter()
        .map(|op| {
            let s: f64 = op.iter().flat_map(|p| p.iter()).sum();
            s / (n_part * n_trial)
        })
        .collect();

    // Part means.
    let part_means: Vec<f64> = (0..num_parts)
        .map(|j| {
            let s: f64 = operators
                .iter()
                .map(|op| op[j].iter().sum::<f64>())
                .sum();
            s / (n_op * n_trial)
        })
        .collect();

    // Cell means (operator x part).
    let cell_means: Vec<Vec<f64>> = operators
        .iter()
        .map(|op| {
            op.iter()
                .map(|trials| trials.iter().sum::<f64>() / n_trial)
                .collect()
        })
        .collect();

    // Sum of Squares: Operator.
    let ss_operator: f64 = op_means
        .iter()
        .map(|&m| (m - grand_mean).powi(2))
        .sum::<f64>()
        * n_part
        * n_trial;

    // Sum of Squares: Part.
    let ss_part: f64 = part_means
        .iter()
        .map(|&m| (m - grand_mean).powi(2))
        .sum::<f64>()
        * n_op
        * n_trial;

    // Sum of Squares: Operator x Part interaction.
    let mut ss_interaction = 0.0_f64;
    for (i, cell_row) in cell_means.iter().enumerate() {
        for (j, &cell_m) in cell_row.iter().enumerate() {
            ss_interaction +=
                (cell_m - op_means[i] - part_means[j] + grand_mean).powi(2);
        }
    }
    ss_interaction *= n_trial;

    // Sum of Squares: Equipment (within-cell / repeatability).
    let mut ss_equipment = 0.0_f64;
    for (i, op) in operators.iter().enumerate() {
        for (j, trials) in op.iter().enumerate() {
            for &x in trials {
                ss_equipment += (x - cell_means[i][j]).powi(2);
            }
        }
    }

    // Degrees of freedom.
    let df_operator = n_op - 1.0;
    let df_part = n_part - 1.0;
    let df_interaction = df_operator * df_part;
    let df_equipment = n_op * n_part * (n_trial - 1.0);

    // Mean squares.
    let ms_operator = if df_operator > 0.0 {
        ss_operator / df_operator
    } else {
        0.0
    };
    let ms_part = if df_part > 0.0 {
        ss_part / df_part
    } else {
        0.0
    };
    let ms_interaction = if df_interaction > 0.0 {
        ss_interaction / df_interaction
    } else {
        0.0
    };
    let ms_equipment = if df_equipment > 0.0 {
        ss_equipment / df_equipment
    } else {
        0.0
    };

    // Variance components.
    let var_repeatability = ms_equipment;
    let var_interaction = ((ms_interaction - ms_equipment) / n_trial).max(0.0);
    let var_reproducibility =
        ((ms_operator - ms_interaction) / (n_part * n_trial)).max(0.0) + var_interaction;
    let _var_operator =
        ((ms_operator - ms_interaction) / (n_part * n_trial)).max(0.0);
    let var_grr = var_repeatability + var_reproducibility;
    let var_part = ((ms_part - ms_interaction) / (n_op * n_trial)).max(0.0);

    // Standard deviations.
    let sd_repeatability = var_repeatability.sqrt();
    let sd_reproducibility = var_reproducibility.sqrt();
    let sd_grr = var_grr.sqrt();
    let sd_part = var_part.sqrt();

    // Study variation (5.15 sigma, per AIAG MSA manual).
    let sv_repeatability = 5.15 * sd_repeatability;
    let sv_reproducibility = 5.15 * sd_reproducibility;
    let sv_grr = 5.15 * sd_grr;
    let sv_part = 5.15 * sd_part;

    // Percentages of tolerance.
    let repeatability_pct = (sv_repeatability / tolerance_range) * 100.0;
    let reproducibility_pct = (sv_reproducibility / tolerance_range) * 100.0;
    let grr_pct = (sv_grr / tolerance_range) * 100.0;
    let ptv_pct = (sv_part / tolerance_range) * 100.0;

    // Number of distinct categories.
    let ndc_raw = if sd_grr > 0.0 {
        (1.41 * sd_part / sd_grr).floor() as usize
    } else {
        0
    };
    let ndc = if ndc_raw < 1 { 1 } else { ndc_raw };

    let acceptable = grr_pct <= 10.0;

    GaugeRRResult {
        grr_pct,
        repeatability_pct,
        reproducibility_pct,
        ptv_pct,
        ndc,
        acceptable,
    }
}

/// Compute process capability indices Cp and Cpk.
///
/// - `Cp  = (USL - LSL) / (6 * sigma)`
/// - `Cpk = min((USL - mean) / (3 * sigma), (mean - LSL) / (3 * sigma))`
///
/// Returns `(Cp, Cpk)`. If the standard deviation is zero, both are returned
/// as `f64::INFINITY` (perfect process with no variation).
pub fn process_capability(measurements: &[f64], usl: f64, lsl: f64) -> (f64, f64) {
    if measurements.is_empty() {
        return (0.0, 0.0);
    }

    let n = measurements.len() as f64;
    let mean = measurements.iter().sum::<f64>() / n;
    let variance = measurements.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let sigma = variance.sqrt();

    if sigma == 0.0 {
        return (f64::INFINITY, f64::INFINITY);
    }

    let cp = (usl - lsl) / (6.0 * sigma);
    let cpu = (usl - mean) / (3.0 * sigma);
    let cpl = (mean - lsl) / (3.0 * sigma);
    let cpk = cpu.min(cpl);

    (cp, cpk)
}

/// Generate a Certificate of Conformance text document.
///
/// The output is a human-readable text block suitable for printing or
/// embedding in a report.
pub fn generate_coc_text(
    lot_id: &str,
    part_name: &str,
    inspections: &[LotInspection],
    notes: &str,
) -> String {
    let mut out = String::new();

    out.push_str("============================================================\n");
    out.push_str("              CERTIFICATE OF CONFORMANCE\n");
    out.push_str("============================================================\n\n");

    out.push_str(&format!("Lot ID:      {lot_id}\n"));
    out.push_str(&format!("Part Name:   {part_name}\n"));
    out.push_str(&format!("Date:        {}\n", now_iso8601()));
    out.push_str(&format!("Inspections: {}\n\n", inspections.len()));

    for (i, insp) in inspections.iter().enumerate() {
        out.push_str(&format!(
            "--- Inspection {} of {} ---\n",
            i + 1,
            inspections.len()
        ));
        out.push_str(&format!("  Plan:        {}\n", insp.plan_name));
        out.push_str(&format!("  Lot Size:    {}\n", insp.lot_size));
        out.push_str(&format!("  Sample Size: {}\n", insp.sample_size));
        out.push_str(&format!("  Defects:     {}\n", insp.defects_found));
        out.push_str(&format!(
            "  Disposition: {}\n",
            if insp.accept { "ACCEPT" } else { "REJECT" }
        ));

        if !insp.results.is_empty() {
            out.push_str("  Results:\n");
            for r in &insp.results {
                let status = if r.within_spec { "OK" } else { "FAIL" };
                out.push_str(&format!(
                    "    - {} = {:.4} [{}] ({})\n",
                    r.characteristic_name,
                    r.measured_value,
                    status,
                    r.classification.label()
                ));
            }
        }
        out.push('\n');
    }

    let all_pass = inspections.iter().all(|i| i.accept);
    out.push_str(&format!(
        "OVERALL DISPOSITION: {}\n\n",
        if all_pass {
            "CONFORMING"
        } else {
            "NON-CONFORMING"
        }
    ));

    if !notes.is_empty() {
        out.push_str(&format!("Notes:\n{notes}\n\n"));
    }

    out.push_str("============================================================\n");
    out.push_str("This certificate attests that the above lot has been\n");
    out.push_str("inspected in accordance with the referenced inspection\n");
    out.push_str("plans and the results are as stated.\n");
    out.push_str("============================================================\n");

    out
}

/// Create a record envelope for a lot inspection result.
pub fn create_inspection_record(
    inspection: &LotInspection,
    author: &str,
) -> RecordEnvelope {
    let id = generate_record_id("qc", "inspection", author);

    let mut refs = BTreeMap::new();
    refs.insert(
        "inspection_plan".to_string(),
        vec![inspection.plan_name.clone()],
    );
    refs.insert("lot".to_string(), vec![inspection.lot_id.clone()]);

    let mut data = BTreeMap::new();
    data.insert(
        "lot_size".into(),
        RecordValue::Integer(inspection.lot_size as i64),
    );
    data.insert(
        "sample_size".into(),
        RecordValue::Integer(inspection.sample_size as i64),
    );
    data.insert(
        "defects_found".into(),
        RecordValue::Integer(inspection.defects_found as i64),
    );
    data.insert(
        "disposition".into(),
        RecordValue::String(if inspection.accept { "accept" } else { "reject" }.into()),
    );

    // Serialize individual results.
    let result_values: Vec<RecordValue> = inspection
        .results
        .iter()
        .map(|r| {
            let mut entry = BTreeMap::new();
            entry.insert(
                "characteristic".into(),
                RecordValue::String(r.characteristic_name.clone()),
            );
            entry.insert(
                "measured_value".into(),
                RecordValue::Float(r.measured_value),
            );
            entry.insert("within_spec".into(), RecordValue::Bool(r.within_spec));
            entry.insert(
                "classification".into(),
                RecordValue::String(r.classification.label().to_string()),
            );
            RecordValue::Table(entry)
        })
        .collect();

    data.insert("results".into(), RecordValue::Array(result_values));

    RecordEnvelope {
        meta: RecordMeta {
            id,
            tool: "qc".into(),
            record_type: "inspection".into(),
            created: now_iso8601(),
            author: author.into(),
        },
        refs,
        data,
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helpers -----------------------------------------------------------

    fn sample_characteristic() -> QualityCharacteristic {
        QualityCharacteristic {
            name: "Diameter".into(),
            classification: CharacteristicClass::Major,
            nominal: 10.0,
            upper_limit: 10.05,
            lower_limit: 9.95,
            unit: "mm".into(),
            measurement_method: "Micrometer".into(),
        }
    }

    fn sample_plan() -> InspectionPlan {
        InspectionPlan {
            name: "Incoming-Shaft".into(),
            plan_type: InspectionType::Incoming,
            characteristics: vec![sample_characteristic()],
            sampling_standard: SamplingStandard::AnsiZ14,
            aql_level: 1.0,
            inspection_level: InspectionLevel::Normal,
        }
    }

    fn sample_lot_inspection(accept: bool) -> LotInspection {
        LotInspection {
            plan_name: "Incoming-Shaft".into(),
            lot_id: "LOT-2026-001".into(),
            lot_size: 500,
            sample_size: 50,
            results: vec![
                InspectionResult {
                    characteristic_name: "Diameter".into(),
                    measured_value: 10.01,
                    within_spec: true,
                    classification: CharacteristicClass::Major,
                },
                InspectionResult {
                    characteristic_name: "Diameter".into(),
                    measured_value: 10.06,
                    within_spec: false,
                    classification: CharacteristicClass::Major,
                },
            ],
            accept,
            defects_found: 1,
        }
    }

    // -- sample_size_z14 tests --------------------------------------------

    #[test]
    fn sample_size_small_lot_normal() {
        // Lot size 50, AQL 1.0, Normal => code D (n=8), Ac=0, Re=1
        let (n, ac, re) = sample_size_z14(50, 1.0, &InspectionLevel::Normal);
        assert_eq!(n, 8);
        assert_eq!(ac, 0);
        assert_eq!(re, 1);
    }

    #[test]
    fn sample_size_medium_lot_normal() {
        // Lot size 500, AQL 1.0, Normal => code H (n=50), Ac=3, Re=4
        let (n, ac, re) = sample_size_z14(500, 1.0, &InspectionLevel::Normal);
        assert_eq!(n, 50);
        assert_eq!(ac, 3);
        assert_eq!(re, 4);
    }

    #[test]
    fn sample_size_large_lot_normal() {
        // Lot size 10000, AQL 1.0, Normal => code L (n=200), Ac=10, Re=11
        let (n, ac, re) = sample_size_z14(10000, 1.0, &InspectionLevel::Normal);
        assert_eq!(n, 200);
        assert_eq!(ac, 10);
        assert_eq!(re, 11);
    }

    #[test]
    fn sample_size_very_large_lot_normal() {
        // Lot size 500000, AQL 2.5, Normal => code P (n=800), Ac=21, Re=22
        let (n, ac, re) = sample_size_z14(500000, 2.5, &InspectionLevel::Normal);
        assert_eq!(n, 800);
        assert_eq!(ac, 21);
        assert_eq!(re, 22);
    }

    #[test]
    fn sample_size_reduced_level() {
        // Lot size 500, AQL 1.0, Reduced => code D (n=8), Ac=0, Re=1
        let (n, ac, re) = sample_size_z14(500, 1.0, &InspectionLevel::Reduced);
        assert_eq!(n, 8);
        assert_eq!(ac, 0);
        assert_eq!(re, 1);
    }

    #[test]
    fn sample_size_tightened_level() {
        // Lot size 500, AQL 1.0, Tightened => code H (n=50), Ac=3, Re=4
        let (n, ac, re) = sample_size_z14(500, 1.0, &InspectionLevel::Tightened);
        assert_eq!(n, 50);
        assert_eq!(ac, 3);
        assert_eq!(re, 4);
    }

    #[test]
    fn sample_size_low_aql() {
        // Lot size 1200, AQL 0.065, Normal => code J (n=80), arrow up to K (n=125), Ac=0
        let (n, ac, re) = sample_size_z14(1200, 0.065, &InspectionLevel::Normal);
        assert_eq!(n, 125);
        assert_eq!(ac, 0);
        assert_eq!(re, 1);
    }

    #[test]
    fn sample_size_high_aql() {
        // Lot size 150, AQL 6.5, Normal => code F (n=20), Ac=7, Re=8
        let (n, ac, re) = sample_size_z14(150, 6.5, &InspectionLevel::Normal);
        assert_eq!(n, 20);
        assert_eq!(ac, 7);
        assert_eq!(re, 8);
    }

    #[test]
    fn sample_size_tiny_lot() {
        // Lot size 2, AQL 4.0, Normal => code A (n=2), Ac=0, Re=1
        let (n, ac, re) = sample_size_z14(2, 4.0, &InspectionLevel::Normal);
        assert_eq!(n, 2);
        assert_eq!(ac, 0);
        assert_eq!(re, 1);
    }

    #[test]
    fn sample_size_aql_025_lot_280() {
        // Lot size 280, AQL 0.25, Normal => code G (n=32), Ac=0, Re=1
        let (n, ac, re) = sample_size_z14(280, 0.25, &InspectionLevel::Normal);
        assert_eq!(n, 32);
        assert_eq!(ac, 0);
        assert_eq!(re, 1);
    }

    #[test]
    fn sample_size_aql_40_lot_3200() {
        // Lot size 3200, AQL 4.0, Normal => code K (n=125), Ac=21, Re=22
        let (n, ac, re) = sample_size_z14(3200, 4.0, &InspectionLevel::Normal);
        assert_eq!(n, 125);
        assert_eq!(ac, 21);
        assert_eq!(re, 22);
    }

    // -- evaluate_lot tests -----------------------------------------------

    #[test]
    fn evaluate_lot_accept() {
        let insp = LotInspection {
            plan_name: "Plan-A".into(),
            lot_id: "LOT-001".into(),
            lot_size: 500,
            sample_size: 50,
            results: vec![
                InspectionResult {
                    characteristic_name: "Width".into(),
                    measured_value: 5.0,
                    within_spec: true,
                    classification: CharacteristicClass::Major,
                },
            ],
            accept: true,
            defects_found: 0,
        };
        assert!(evaluate_lot(&insp));
    }

    #[test]
    fn evaluate_lot_reject() {
        let insp = LotInspection {
            plan_name: "Plan-A".into(),
            lot_id: "LOT-002".into(),
            lot_size: 500,
            sample_size: 50,
            results: (0..10)
                .map(|i| InspectionResult {
                    characteristic_name: format!("Dim-{i}"),
                    measured_value: 99.0,
                    within_spec: false,
                    classification: CharacteristicClass::Major,
                })
                .collect(),
            accept: false,
            defects_found: 10,
        };
        assert!(!evaluate_lot(&insp));
    }

    #[test]
    fn evaluate_lot_empty_results_uses_defects() {
        let mut insp = LotInspection {
            plan_name: "Plan-A".into(),
            lot_id: "LOT-003".into(),
            lot_size: 50,
            sample_size: 8,
            results: vec![],
            accept: true,
            defects_found: 0,
        };
        assert!(evaluate_lot(&insp));

        insp.defects_found = 5;
        assert!(!evaluate_lot(&insp));
    }

    // -- process_capability tests -----------------------------------------

    #[test]
    fn process_capability_centered() {
        // Centered process with known sigma.
        let measurements: Vec<f64> = vec![9.98, 10.02, 9.99, 10.01, 10.00];
        let (cp, cpk) = process_capability(&measurements, 10.05, 9.95);
        assert!(cp > 1.0, "Cp should be > 1.0, got {cp}");
        assert!(cpk > 0.9, "Cpk should be > 0.9, got {cpk}");
        // Cp and Cpk should be close for a centered process.
        assert!(
            (cp - cpk).abs() < 0.5,
            "Cp and Cpk should be close for centered process"
        );
    }

    #[test]
    fn process_capability_shifted() {
        // Process shifted toward USL.
        let measurements: Vec<f64> = vec![10.03, 10.04, 10.03, 10.04, 10.03];
        let (cp, cpk) = process_capability(&measurements, 10.05, 9.95);
        assert!(cp > 1.0, "Cp should still be > 1.0");
        assert!(cpk < cp, "Cpk should be less than Cp for shifted process");
    }

    #[test]
    fn process_capability_zero_variation() {
        let measurements: Vec<f64> = vec![10.0, 10.0, 10.0];
        let (cp, cpk) = process_capability(&measurements, 10.05, 9.95);
        assert!(cp.is_infinite());
        assert!(cpk.is_infinite());
    }

    #[test]
    fn process_capability_empty() {
        let (cp, cpk) = process_capability(&[], 10.05, 9.95);
        assert_eq!(cp, 0.0);
        assert_eq!(cpk, 0.0);
    }

    // -- compute_gauge_rr tests -------------------------------------------

    #[test]
    fn gauge_rr_perfect_measurement() {
        // All operators measure exactly the same values with no variation.
        // Parts differ, but there is zero measurement error.
        let operators = vec![
            vec![vec![1.0, 1.0], vec![2.0, 2.0], vec![3.0, 3.0]],
            vec![vec![1.0, 1.0], vec![2.0, 2.0], vec![3.0, 3.0]],
        ];
        let result = compute_gauge_rr(&operators, 10.0);
        assert!(
            result.grr_pct < 1.0,
            "GRR should be near zero for perfect measurements, got {}",
            result.grr_pct
        );
        assert!(result.acceptable);
    }

    #[test]
    fn gauge_rr_high_variation() {
        // Large measurement variation relative to tolerance.
        let operators = vec![
            vec![vec![1.0, 5.0], vec![2.0, 8.0], vec![3.0, 9.0]],
            vec![vec![4.0, 1.0], vec![7.0, 2.0], vec![6.0, 3.0]],
        ];
        let result = compute_gauge_rr(&operators, 1.0);
        assert!(
            result.grr_pct > 10.0,
            "GRR should be high for noisy measurements, got {}",
            result.grr_pct
        );
        assert!(!result.acceptable);
    }

    #[test]
    fn gauge_rr_empty_input() {
        let result = compute_gauge_rr(&[], 10.0);
        assert_eq!(result.grr_pct, 0.0);
        assert!(!result.acceptable);
    }

    #[test]
    fn gauge_rr_ndc_minimum_one() {
        // Even with bad data, NDC should be at least 1.
        let operators = vec![
            vec![vec![5.0, 5.0], vec![5.0, 5.0]],
            vec![vec![5.0, 5.0], vec![5.0, 5.0]],
        ];
        let result = compute_gauge_rr(&operators, 10.0);
        assert!(result.ndc >= 1);
    }

    // -- generate_coc_text tests ------------------------------------------

    #[test]
    fn coc_text_contains_lot_info() {
        let insp = sample_lot_inspection(true);
        let text = generate_coc_text("LOT-2026-001", "Shaft-A", &[insp], "No issues.");

        assert!(text.contains("CERTIFICATE OF CONFORMANCE"));
        assert!(text.contains("LOT-2026-001"));
        assert!(text.contains("Shaft-A"));
        assert!(text.contains("ACCEPT"));
        assert!(text.contains("CONFORMING"));
        assert!(text.contains("No issues."));
    }

    #[test]
    fn coc_text_non_conforming() {
        let insp = sample_lot_inspection(false);
        let text = generate_coc_text("LOT-2026-002", "Gear-B", &[insp], "");

        assert!(text.contains("REJECT"));
        assert!(text.contains("NON-CONFORMING"));
    }

    #[test]
    fn coc_text_empty_inspections() {
        let text = generate_coc_text("LOT-EMPTY", "Widget", &[], "");
        assert!(text.contains("Inspections: 0"));
        assert!(text.contains("CONFORMING"));
    }

    // -- create_inspection_record tests -----------------------------------

    #[test]
    fn inspection_record_structure() {
        let insp = sample_lot_inspection(true);
        let envelope = create_inspection_record(&insp, "alice");

        assert_eq!(envelope.meta.tool, "qc");
        assert_eq!(envelope.meta.record_type, "inspection");
        assert_eq!(envelope.meta.author, "alice");
        assert!(envelope.refs.contains_key("inspection_plan"));
        assert!(envelope.refs.contains_key("lot"));
        assert_eq!(
            envelope.data.get("lot_size"),
            Some(&RecordValue::Integer(500))
        );
        assert_eq!(
            envelope.data.get("sample_size"),
            Some(&RecordValue::Integer(50))
        );
        assert_eq!(
            envelope.data.get("disposition"),
            Some(&RecordValue::String("accept".into()))
        );
    }

    #[test]
    fn inspection_record_reject_disposition() {
        let insp = sample_lot_inspection(false);
        let envelope = create_inspection_record(&insp, "bob");

        assert_eq!(
            envelope.data.get("disposition"),
            Some(&RecordValue::String("reject".into()))
        );
    }

    #[test]
    fn inspection_record_toml_round_trip() {
        let insp = sample_lot_inspection(true);
        let envelope = create_inspection_record(&insp, "tester");
        let toml = envelope.to_toml_string();
        let parsed = RecordEnvelope::from_toml_str(&toml).unwrap();

        assert_eq!(parsed.meta.tool, "qc");
        assert_eq!(parsed.meta.record_type, "inspection");
        assert_eq!(parsed.data.get("lot_size"), envelope.data.get("lot_size"));
        assert_eq!(
            parsed.data.get("disposition"),
            envelope.data.get("disposition")
        );
    }

    // -- Enum label / serde tests -----------------------------------------

    #[test]
    fn inspection_type_labels() {
        assert_eq!(InspectionType::Incoming.label(), "Incoming");
        assert_eq!(InspectionType::InProcess.label(), "In-Process");
        assert_eq!(InspectionType::Final.label(), "Final");
        assert_eq!(InspectionType::FirstArticle.label(), "First Article");
        assert_eq!(
            InspectionType::PeriodicRequalification.label(),
            "Periodic Requalification"
        );
    }

    #[test]
    fn characteristic_class_labels() {
        assert_eq!(CharacteristicClass::Critical.label(), "Critical");
        assert_eq!(CharacteristicClass::Major.label(), "Major");
        assert_eq!(CharacteristicClass::Minor.label(), "Minor");
        assert_eq!(CharacteristicClass::Informational.label(), "Informational");
    }

    #[test]
    fn sampling_standard_labels() {
        assert_eq!(SamplingStandard::AnsiZ14.label(), "ANSI/ASQ Z1.4");
        assert_eq!(SamplingStandard::Iso2859.label(), "ISO 2859-1");
        assert_eq!(SamplingStandard::CZero.label(), "c=0");
        assert_eq!(SamplingStandard::HundredPercent.label(), "100%");
        assert_eq!(SamplingStandard::Custom.label(), "Custom");
    }

    #[test]
    fn inspection_level_labels() {
        assert_eq!(InspectionLevel::Reduced.label(), "Reduced");
        assert_eq!(InspectionLevel::Normal.label(), "Normal");
        assert_eq!(InspectionLevel::Tightened.label(), "Tightened");
    }

    // -- aql_index tests --------------------------------------------------

    #[test]
    fn aql_index_exact_match() {
        assert_eq!(aql_index(0.065), 0);
        assert_eq!(aql_index(1.0), 6);
        assert_eq!(aql_index(6.5), 10);
    }

    #[test]
    fn aql_index_closest_match() {
        // 0.09 is closer to 0.10 than 0.065
        assert_eq!(aql_index(0.09), 1);
        // 0.5 is closer to 0.40 than 0.65
        assert_eq!(aql_index(0.5), 4);
    }

    // -- InspectionPlan construction test ---------------------------------

    #[test]
    fn inspection_plan_construction() {
        let plan = sample_plan();
        assert_eq!(plan.name, "Incoming-Shaft");
        assert_eq!(plan.plan_type, InspectionType::Incoming);
        assert_eq!(plan.characteristics.len(), 1);
        assert_eq!(plan.sampling_standard, SamplingStandard::AnsiZ14);
        assert!((plan.aql_level - 1.0).abs() < f64::EPSILON);
        assert_eq!(plan.inspection_level, InspectionLevel::Normal);
    }

    #[test]
    fn quality_characteristic_fields() {
        let ch = sample_characteristic();
        assert_eq!(ch.name, "Diameter");
        assert_eq!(ch.classification, CharacteristicClass::Major);
        assert!((ch.nominal - 10.0).abs() < f64::EPSILON);
        assert!((ch.upper_limit - 10.05).abs() < f64::EPSILON);
        assert!((ch.lower_limit - 9.95).abs() < f64::EPSILON);
        assert_eq!(ch.unit, "mm");
        assert_eq!(ch.measurement_method, "Micrometer");
    }
}

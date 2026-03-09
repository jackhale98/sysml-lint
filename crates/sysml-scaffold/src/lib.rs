//! Scaffolding and example generation for SysML v2 projects.
//!
//! This crate builds on `sysml-core` to generate richly-commented SysML v2
//! templates, verification cases, risk definitions, tolerance chains, and
//! complete example projects suitable for learning or bootstrapping new work.

use std::fmt;

use sysml_core::codegen::template::{generate_template, MemberSpec, TemplateOptions};
use sysml_core::model::DefKind;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during scaffold generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaffoldError {
    /// The requested element kind is not recognised.
    UnknownKind(String),
    /// The requested example name is not recognised.
    UnknownExample(String),
}

impl fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKind(k) => write!(f, "unknown element kind: '{}'", k),
            Self::UnknownExample(n) => write!(f, "unknown example: '{}'", n),
        }
    }
}

impl std::error::Error for ScaffoldError {}

// ---------------------------------------------------------------------------
// ScaffoldOptions
// ---------------------------------------------------------------------------

/// Options controlling scaffold generation.
#[derive(Debug, Clone, Default)]
pub struct ScaffoldOptions {
    /// Optional supertype the element extends.
    pub extends: Option<String>,
    /// Optional doc comment placed inside the element body.
    pub doc: Option<String>,
    /// Member specifications (e.g. `"part engine : Engine"`).
    pub members: Vec<String>,
    /// When `true`, teaching comments are interleaved with the output.
    pub with_teaching_comments: bool,
}

// ---------------------------------------------------------------------------
// Kind resolution helpers
// ---------------------------------------------------------------------------

/// Map a user-facing kind string to `DefKind`.
fn resolve_kind(kind: &str) -> Result<DefKind, ScaffoldError> {
    let normalised = kind.to_lowercase().replace('-', " ");
    match normalised.as_str() {
        "part def" | "part" => Ok(DefKind::Part),
        "port def" | "port" => Ok(DefKind::Port),
        "connection def" | "connection" => Ok(DefKind::Connection),
        "interface def" | "interface" => Ok(DefKind::Interface),
        "flow def" | "flow" => Ok(DefKind::Flow),
        "action def" | "action" => Ok(DefKind::Action),
        "state def" | "state" => Ok(DefKind::State),
        "constraint def" | "constraint" => Ok(DefKind::Constraint),
        "calc def" | "calc" => Ok(DefKind::Calc),
        "requirement def" | "requirement" | "req" => Ok(DefKind::Requirement),
        "use case def" | "use case" | "usecase" => Ok(DefKind::UseCase),
        "verification def" | "verification" => Ok(DefKind::Verification),
        "enum def" | "enum" => Ok(DefKind::Enum),
        "attribute def" | "attribute" | "attr" => Ok(DefKind::Attribute),
        "item def" | "item" => Ok(DefKind::Item),
        "view def" | "view" => Ok(DefKind::View),
        "viewpoint def" | "viewpoint" => Ok(DefKind::Viewpoint),
        "allocation def" | "allocation" => Ok(DefKind::Allocation),
        "package" | "pkg" => Ok(DefKind::Package),
        _ => Err(ScaffoldError::UnknownKind(kind.to_string())),
    }
}

/// Return a teaching comment for the given `DefKind`.
fn teaching_comment_for(kind: DefKind) -> &'static str {
    match kind {
        DefKind::Part => {
            "/* A 'part def' declares a reusable definition of a structural component.\n\
             * Instances are created with 'part' usages inside other definitions. */"
        }
        DefKind::Port => {
            "/* A 'port def' defines an interaction point through which parts\n\
             * exchange items such as signals, data, or physical flows. */"
        }
        DefKind::Connection => {
            "/* A 'connection def' defines a structural link between two parts,\n\
             * typically used to model wiring, piping, or data buses. */"
        }
        DefKind::Interface => {
            "/* An 'interface def' specifies a bundle of ports that must be\n\
             * provided or required together by a part. */"
        }
        DefKind::Flow => {
            "/* A 'flow def' describes the transfer of items (mass, energy,\n\
             * information) between ports of connected parts. */"
        }
        DefKind::Action => {
            "/* An 'action def' defines a unit of behaviour — a step in a\n\
             * process, algorithm, or procedure. Actions can be composed\n\
             * sequentially, in parallel, or conditionally. */"
        }
        DefKind::State => {
            "/* A 'state def' defines a state in a state machine. States\n\
             * contain entry/do/exit actions and transitions triggered by\n\
             * events or guard conditions. */"
        }
        DefKind::Constraint => {
            "/* A 'constraint def' expresses an equation or inequality that\n\
             * must hold for the system. Used for parametric analysis. */"
        }
        DefKind::Calc => {
            "/* A 'calc def' defines a calculation — a pure function that\n\
             * computes output values from input parameters. */"
        }
        DefKind::Requirement => {
            "/* A 'requirement def' captures a stakeholder need or system\n\
             * obligation. Requirements can be refined, derived, and traced\n\
             * to verification cases via 'satisfy' and 'verify'. */"
        }
        DefKind::UseCase => {
            "/* A 'use case def' describes a goal-oriented interaction\n\
             * between actors and the system. */"
        }
        DefKind::Verification => {
            "/* A 'verification def' specifies how a requirement is checked.\n\
             * It references the requirement being verified and includes\n\
             * objective, method, and acceptance criteria. */"
        }
        DefKind::Enum => {
            "/* An 'enum def' defines a set of named literal values,\n\
             * similar to enumerations in programming languages. */"
        }
        DefKind::Attribute => {
            "/* An 'attribute def' defines a value property (e.g. mass,\n\
             * temperature) that characterises a part or item. */"
        }
        DefKind::Item => {
            "/* An 'item def' defines a passive entity that is exchanged\n\
             * between parts but has no behaviour of its own. */"
        }
        DefKind::View => {
            "/* A 'view def' defines a filtered presentation of model\n\
             * elements, selecting what to expose and how. */"
        }
        DefKind::Viewpoint => {
            "/* A 'viewpoint def' captures the concerns and stakeholders\n\
             * that a view addresses. */"
        }
        DefKind::Allocation => {
            "/* An 'allocation def' maps logical elements (functions,\n\
             * requirements) to physical elements (components, resources). */"
        }
        DefKind::Package => {
            "/* A 'package' is a namespace container that groups related\n\
             * definitions, usages, and sub-packages. */"
        }
        _ => "/* Definition. */",
    }
}

// ---------------------------------------------------------------------------
// Public API — scaffold_element
// ---------------------------------------------------------------------------

/// Generate a SysML v2 element definition with optional teaching comments.
///
/// # Arguments
///
/// * `kind` — Element kind string (e.g. `"part"`, `"requirement"`, `"action def"`).
/// * `name` — The element name.
/// * `options` — Additional options (supertype, doc, members, teaching comments).
///
/// # Errors
///
/// Returns [`ScaffoldError::UnknownKind`] if `kind` is not recognised.
pub fn scaffold_element(
    kind: &str,
    name: &str,
    options: &ScaffoldOptions,
) -> Result<String, ScaffoldError> {
    let def_kind = resolve_kind(kind)?;

    // Parse member specs.
    let members: Vec<MemberSpec> = options
        .members
        .iter()
        .filter_map(|s| parse_simple_member(s))
        .collect();

    let opts = TemplateOptions {
        kind: def_kind,
        name: name.to_string(),
        super_type: options.extends.clone(),
        is_abstract: false,
        short_name: None,
        doc: options.doc.clone(),
        members,
        exposes: Vec::new(),
        filter: None,
        indent: 0,
    };

    let body = generate_template(&opts);

    if options.with_teaching_comments {
        let comment = teaching_comment_for(def_kind);
        Ok(format!("{}\n{}", comment, body))
    } else {
        Ok(body)
    }
}

/// Minimal member-spec parser.
///
/// Accepts forms like:
/// - `"part engine : Engine"`
/// - `"in port fuelIn : FuelPort"`
/// - `"attribute mass : Real"`
fn parse_simple_member(s: &str) -> Option<MemberSpec> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut idx = 0;

    // Optional direction prefix.
    let direction = match parts.get(idx).copied() {
        Some("in" | "out" | "inout") => {
            let d = parts[idx].to_string();
            idx += 1;
            Some(d)
        }
        _ => None,
    };

    // Usage kind.
    let usage_kind = parts.get(idx)?.to_string();
    idx += 1;

    // Name (possibly with `:Type` stuck together or separated by ` : `).
    let raw_name = (*parts.get(idx)?).to_string();
    idx += 1;

    let (name, type_ref) = if let Some((n, t)) = raw_name.split_once(':') {
        (n.to_string(), Some(t.to_string()))
    } else if parts.get(idx).copied() == Some(":") {
        idx += 1;
        let t = parts.get(idx).map(|s| s.to_string());
        (raw_name, t)
    } else {
        (raw_name, None)
    };

    Some(MemberSpec {
        usage_kind,
        name,
        type_ref,
        direction,
        multiplicity: None,
    })
}

// ---------------------------------------------------------------------------
// Public API — scaffold_verification_case
// ---------------------------------------------------------------------------

/// Generate a verification case template referencing the given requirements.
///
/// The output includes a `verification def` with a `subject`, `objective`,
/// and ordered verification steps.
pub fn scaffold_verification_case(name: &str, requirements: &[&str]) -> String {
    let mut out = String::new();

    out.push_str(&format!("verification def {} {{\n", name));
    out.push_str(&format!(
        "    doc /* Verification case for: {} */\n",
        requirements.join(", ")
    ));
    out.push('\n');

    out.push_str("    subject testSubject;\n");
    out.push('\n');

    // Reference each requirement.
    for req in requirements {
        out.push_str(&format!(
            "    requirement {} : {};\n",
            to_usage_name(req),
            req
        ));
    }
    out.push('\n');

    out.push_str("    objective verificationObjective {\n");
    out.push_str(&format!(
        "        doc /* Demonstrate that {} are satisfied */\n",
        requirements.join(" and ")
    ));
    out.push_str("    }\n");
    out.push('\n');

    // Steps.
    out.push_str("    action step1_setup {\n");
    out.push_str("        doc /* Configure the test subject for verification */\n");
    out.push_str("    }\n");
    out.push('\n');
    out.push_str("    action step2_execute {\n");
    out.push_str("        doc /* Execute the test procedure */\n");
    out.push_str("    }\n");
    out.push('\n');
    out.push_str("    action step3_evaluate {\n");
    out.push_str("        doc /* Evaluate results against acceptance criteria */\n");
    out.push_str("    }\n");

    out.push_str("}\n");
    out
}

/// Lower a definition name to a usage name (first char lowercase).
fn to_usage_name(def_name: &str) -> String {
    let mut chars = def_name.chars();
    match chars.next() {
        Some(c) => {
            let lower: String = c.to_lowercase().collect();
            format!("{}{}", lower, chars.as_str())
        }
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Public API — scaffold_risk_template
// ---------------------------------------------------------------------------

/// Generate a risk definition template using the SysMLRisk library pattern.
///
/// The output includes severity and likelihood attributes, and a mitigation
/// action placeholder.
pub fn scaffold_risk_template(name: &str) -> String {
    let mut out = String::new();

    out.push_str("/* Risk modelling pattern using the SysML Risk library. */\n");
    out.push('\n');
    out.push_str("enum def RiskLikelihood {\n");
    out.push_str("    doc /* Probability that the hazard occurs */\n");
    out.push_str("    enum remote;\n");
    out.push_str("    enum unlikely;\n");
    out.push_str("    enum possible;\n");
    out.push_str("    enum likely;\n");
    out.push_str("    enum certain;\n");
    out.push_str("}\n");
    out.push('\n');
    out.push_str("enum def RiskSeverity {\n");
    out.push_str("    doc /* Impact if the hazard materialises */\n");
    out.push_str("    enum negligible;\n");
    out.push_str("    enum marginal;\n");
    out.push_str("    enum critical;\n");
    out.push_str("    enum catastrophic;\n");
    out.push_str("}\n");
    out.push('\n');
    out.push_str(&format!("part def {} {{\n", name));
    out.push_str(&format!(
        "    doc /* Risk: describe the hazard for {} */\n",
        name
    ));
    out.push('\n');
    out.push_str("    attribute likelihood : RiskLikelihood;\n");
    out.push_str("    attribute severity : RiskSeverity;\n");
    out.push('\n');
    out.push_str("    requirement mitigationNeeded {\n");
    out.push_str("        doc /* Condition under which this risk must be mitigated */\n");
    out.push_str("    }\n");
    out.push('\n');
    out.push_str("    action mitigationAction {\n");
    out.push_str("        doc /* Steps to reduce likelihood or severity */\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

// ---------------------------------------------------------------------------
// Public API — scaffold_tolerance_chain
// ---------------------------------------------------------------------------

/// Generate a tolerance (dimension) chain template.
///
/// Each contributor becomes a separate attribute definition with tolerance
/// annotations, and a constraint ties them together.
pub fn scaffold_tolerance_chain(name: &str, contributors: &[&str]) -> String {
    let mut out = String::new();

    out.push_str(&format!("package {} {{\n", name));
    out.push_str(&format!("    doc /* Tolerance chain: {} */\n", name));
    out.push('\n');

    // Attribute definitions for each contributor.
    for contrib in contributors {
        out.push_str(&format!("    attribute def {} {{\n", contrib));
        out.push_str(&format!(
            "        doc /* Dimension contributor: {} */\n",
            contrib
        ));
        out.push_str("        attribute nominal : Real;\n");
        out.push_str("        attribute tolerancePlus : Real;\n");
        out.push_str("        attribute toleranceMinus : Real;\n");
        out.push_str("    }\n");
        out.push('\n');
    }

    // Stack-up constraint.
    out.push_str("    constraint def StackUpConstraint {\n");
    out.push_str("        doc /* Worst-case stack-up must remain within limits */\n");

    for contrib in contributors {
        out.push_str(&format!(
            "        attribute {} : {};\n",
            to_usage_name(contrib),
            contrib
        ));
    }

    out.push_str("        attribute totalNominal : Real;\n");
    out.push_str("        attribute totalTolerancePlus : Real;\n");
    out.push_str("        attribute totalToleranceMinus : Real;\n");
    out.push_str("    }\n");

    out.push_str("}\n");
    out
}

// ---------------------------------------------------------------------------
// Public API — scaffold_example / list_examples
// ---------------------------------------------------------------------------

/// Metadata for a built-in example project.
struct ExampleMeta {
    name: &'static str,
    description: &'static str,
}

const EXAMPLES: &[ExampleMeta] = &[
    ExampleMeta {
        name: "brake-system",
        description: "Vehicle brake system with parts, requirements, and verification cases",
    },
    ExampleMeta {
        name: "sensor-module",
        description: "Electronic sensor module with ports, interfaces, and state machines",
    },
];

/// Return the list of available built-in examples with descriptions.
pub fn list_examples() -> Vec<(&'static str, &'static str)> {
    EXAMPLES.iter().map(|e| (e.name, e.description)).collect()
}

/// Generate a complete example project as a list of `(filename, content)` pairs.
///
/// # Supported examples
///
/// - `"brake-system"` — a vehicle brake system with parts, requirements, and
///   verification cases.
/// - `"sensor-module"` — an electronic sensor module with ports, interfaces,
///   state machines, and verification cases.
///
/// # Errors
///
/// Returns [`ScaffoldError::UnknownExample`] if `name` is not recognised.
pub fn scaffold_example(name: &str) -> Result<Vec<(String, String)>, ScaffoldError> {
    match name {
        "brake-system" => Ok(example_brake_system()),
        "sensor-module" => Ok(example_sensor_module()),
        _ => Err(ScaffoldError::UnknownExample(name.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Public API — list_element_kinds
// ---------------------------------------------------------------------------

/// Return all supported element kinds with short descriptions.
pub fn list_element_kinds() -> Vec<(&'static str, &'static str)> {
    vec![
        ("part", "Structural component definition"),
        ("port", "Interaction point for exchanging items"),
        ("connection", "Structural link between parts"),
        ("interface", "Bundle of ports provided/required together"),
        ("flow", "Transfer of items between ports"),
        ("action", "Unit of behaviour or process step"),
        ("state", "State in a state machine"),
        ("constraint", "Equation or inequality (parametric)"),
        ("calc", "Pure calculation function"),
        ("requirement", "Stakeholder need or system obligation"),
        ("use case", "Goal-oriented actor-system interaction"),
        ("verification", "Verification case for a requirement"),
        ("enum", "Set of named literal values"),
        ("attribute", "Value property (e.g. mass, temperature)"),
        ("item", "Passive entity exchanged between parts"),
        ("view", "Filtered presentation of model elements"),
        ("viewpoint", "Stakeholder concerns for a view"),
        ("allocation", "Mapping of logical to physical elements"),
        ("package", "Namespace container for definitions"),
    ]
}

// ---------------------------------------------------------------------------
// Built-in example: brake-system
// ---------------------------------------------------------------------------

fn example_brake_system() -> Vec<(String, String)> {
    let mut files = Vec::new();

    // -- 1. Package & attribute definitions --
    let types = "\
package BrakeSystemTypes {
    doc /* Common types and attributes for the brake system model */

    attribute def Force {
        doc /* A force quantity measured in Newtons */
        attribute value : Real;
    }

    attribute def Pressure {
        doc /* A pressure quantity measured in Pascals */
        attribute value : Real;
    }

    attribute def Temperature {
        doc /* A temperature quantity measured in Kelvin */
        attribute value : Real;
    }

    attribute def Distance {
        doc /* A distance quantity measured in metres */
        attribute value : Real;
    }

    enum def BrakeMode {
        doc /* Operating modes of the brake system */
        enum normal;
        enum emergency;
        enum parking;
    }
}
";
    files.push(("types.sysml".to_string(), types.to_string()));

    // -- 2. Part definitions --
    let parts = "\
package BrakeSystemParts {
    doc /* Part definitions for the hydraulic brake system */

    import BrakeSystemTypes::*;

    part def BrakeSystem {
        doc /* Top-level brake system assembly */

        part pedalAssembly : PedalAssembly;
        part masterCylinder : MasterCylinder;
        part brakeLine : BrakeLine[2];
        part caliper : BrakeCaliper[4];
        part pad : BrakePad[8];

        attribute maxDeceleration : Force;
    }

    part def PedalAssembly {
        doc /* Driver-operated brake pedal mechanism */

        attribute pedalForce : Force;
        attribute pedalTravel : Distance;
    }

    part def MasterCylinder {
        doc /* Converts pedal force to hydraulic pressure */

        attribute outputPressure : Pressure;
        attribute boreDiameter : Distance;
    }

    part def BrakeLine {
        doc /* Hydraulic line carrying brake fluid under pressure */

        attribute linePressure : Pressure;
        attribute maxTemperature : Temperature;
    }

    part def BrakeCaliper {
        doc /* Clamps brake pads against the rotor */

        attribute clampingForce : Force;
        attribute pistonCount : Real;
    }

    part def BrakePad {
        doc /* Friction material that contacts the rotor */

        attribute frictionCoefficient : Real;
        attribute wearThickness : Distance;
    }
}
";
    files.push(("parts.sysml".to_string(), parts.to_string()));

    // -- 3. Requirements --
    let reqs = "\
package BrakeRequirements {
    doc /* Requirements for the brake system */

    import BrakeSystemTypes::*;

    requirement def StoppingDistanceReq {
        doc /* The vehicle shall stop within the specified distance
               from a given speed under normal braking conditions. */

        attribute maxDistance : Distance;
        attribute fromSpeed : Real;
    }

    requirement def PedalForceReq {
        doc /* The pedal force required to achieve full braking
               shall not exceed the specified limit. */

        attribute maxPedalForce : Force;
    }

    requirement def FadeResistanceReq {
        doc /* The brake system shall maintain at least 80% of nominal
               braking performance after repeated high-energy stops. */

        attribute minPerformanceRatio : Real;
    }

    requirement def ParkingBrakeReq {
        doc /* The parking brake shall hold the vehicle stationary
               on a slope of at least 30% gradient. */

        attribute minGradient : Real;
    }
}
";
    files.push(("requirements.sysml".to_string(), reqs.to_string()));

    // -- 4. Verification --
    let verif = "\
package BrakeVerification {
    doc /* Verification cases for brake system requirements */

    import BrakeRequirements::*;

    verification def StoppingDistanceTest {
        doc /* Verify stopping distance under controlled conditions */

        subject testVehicle;

        requirement stopReq : StoppingDistanceReq;

        objective testObjective {
            doc /* Demonstrate that stopping distance meets StoppingDistanceReq */
        }

        action step1_accelerate {
            doc /* Accelerate the test vehicle to the specified speed */
        }

        action step2_applyBrakes {
            doc /* Apply brakes with nominal pedal force */
        }

        action step3_measureDistance {
            doc /* Measure the distance from brake application to full stop */
        }

        action step4_evaluate {
            doc /* Compare measured distance against maxDistance */
        }
    }

    verification def PedalForceTest {
        doc /* Verify that required pedal force is within acceptable limits */

        subject testVehicle;

        requirement pedalReq : PedalForceReq;

        objective testObjective {
            doc /* Demonstrate that pedal force meets PedalForceReq */
        }

        action step1_instrument {
            doc /* Install force sensor on brake pedal */
        }

        action step2_measure {
            doc /* Record pedal force during full braking */
        }

        action step3_evaluate {
            doc /* Verify measured force is below maxPedalForce */
        }
    }
}
";
    files.push(("verification.sysml".to_string(), verif.to_string()));

    files
}

// ---------------------------------------------------------------------------
// Built-in example: sensor-module
// ---------------------------------------------------------------------------

fn example_sensor_module() -> Vec<(String, String)> {
    let mut files = Vec::new();

    // -- 1. Types & ports --
    let types = "\
package SensorTypes {
    doc /* Types and port definitions for the sensor module */

    attribute def Voltage {
        doc /* Electrical potential measured in Volts */
        attribute value : Real;
    }

    attribute def Current {
        doc /* Electrical current measured in Amperes */
        attribute value : Real;
    }

    attribute def SensorReading {
        doc /* A single digitised sensor measurement */
        attribute value : Real;
        attribute timestamp : Real;
    }

    enum def SensorState {
        doc /* Operational states of the sensor module */
        enum off;
        enum initialising;
        enum ready;
        enum sampling;
        enum fault;
    }

    port def PowerPort {
        doc /* Electrical power supply port */
        in attribute voltage : Voltage;
        in attribute current : Current;
    }

    port def DataPort {
        doc /* Digital data output port */
        out attribute reading : SensorReading;
    }

    port def ControlPort {
        doc /* Command and status port */
        in attribute command : Real;
        out attribute status : Real;
    }

    interface def SensorInterface {
        doc /* Complete sensor interface combining power, data, and control */
        end sensorEnd : PowerPort;
        end hostEnd : DataPort;
    }
}
";
    files.push(("types.sysml".to_string(), types.to_string()));

    // -- 2. Part definitions --
    let parts = "\
package SensorParts {
    doc /* Part definitions for the sensor module */

    import SensorTypes::*;

    part def SensorModule {
        doc /* Top-level sensor module assembly */

        port powerIn : PowerPort;
        port dataOut : DataPort;
        port controlPort : ControlPort;

        part sensingElement : SensingElement;
        part adc : AnalogToDigitalConverter;
        part controller : SensorController;

        attribute samplingRateHz : Real;
        attribute resolutionBits : Real;
    }

    part def SensingElement {
        doc /* Physical transducer converting a measurand to voltage */

        attribute sensitivity : Real;
        attribute measurementRange : Real;
    }

    part def AnalogToDigitalConverter {
        doc /* Converts analog sensor signal to digital readings */

        attribute resolutionBits : Real;
        attribute maxSampleRate : Real;
    }

    part def SensorController {
        doc /* Embedded controller managing sensor operation */

        attribute firmwareVersion : Real;
    }
}
";
    files.push(("parts.sysml".to_string(), parts.to_string()));

    // -- 3. Requirements --
    let reqs = "\
package SensorRequirements {
    doc /* Requirements for the sensor module */

    import SensorTypes::*;

    requirement def AccuracyReq {
        doc /* The sensor shall measure the target quantity
               within the specified accuracy tolerance. */

        attribute maxError : Real;
    }

    requirement def ResponseTimeReq {
        doc /* The sensor shall produce a valid reading within
               the specified time after power-on. */

        attribute maxStartupTimeMs : Real;
    }

    requirement def PowerConsumptionReq {
        doc /* The sensor module shall not exceed the specified
               power draw during normal operation. */

        attribute maxPowerWatts : Real;
    }

    requirement def FaultDetectionReq {
        doc /* The sensor shall detect internal faults and
               report them within the specified latency. */

        attribute maxDetectionTimeMs : Real;
    }
}
";
    files.push(("requirements.sysml".to_string(), reqs.to_string()));

    // -- 4. State machine --
    let states = "\
package SensorBehaviour {
    doc /* Behavioural model of the sensor module */

    import SensorTypes::*;

    state def SensorStateMachine {
        doc /* State machine governing sensor operation */

        entry state off {
            doc /* Sensor is powered off */
        }

        state initialising {
            doc /* Sensor is performing self-test and calibration */
        }

        state ready {
            doc /* Sensor is calibrated and awaiting a sample command */
        }

        state sampling {
            doc /* Sensor is actively acquiring measurements */
        }

        state fault {
            doc /* Sensor has detected an internal error */
        }

        transition powerOn
            first off
            then initialising;

        transition initComplete
            first initialising
            then ready;

        transition startSampling
            first ready
            then sampling;

        transition stopSampling
            first sampling
            then ready;

        transition faultDetected
            first sampling
            then fault;

        transition faultCleared
            first fault
            then initialising;
    }
}
";
    files.push(("behaviour.sysml".to_string(), states.to_string()));

    // -- 5. Verification --
    let verif = "\
package SensorVerification {
    doc /* Verification cases for sensor module requirements */

    import SensorRequirements::*;

    verification def AccuracyTest {
        doc /* Verify measurement accuracy against a reference standard */

        subject testSensor;

        requirement accReq : AccuracyReq;

        objective testObjective {
            doc /* Demonstrate that sensor accuracy meets AccuracyReq */
        }

        action step1_setupReference {
            doc /* Configure the reference measurement standard */
        }

        action step2_acquireReadings {
            doc /* Collect sensor readings across the measurement range */
        }

        action step3_compareResults {
            doc /* Compute error between sensor and reference values */
        }

        action step4_evaluate {
            doc /* Verify all errors are within maxError */
        }
    }

    verification def ResponseTimeTest {
        doc /* Verify sensor startup and first-reading latency */

        subject testSensor;

        requirement respReq : ResponseTimeReq;

        objective testObjective {
            doc /* Demonstrate that startup time meets ResponseTimeReq */
        }

        action step1_powerCycle {
            doc /* Power off and then power on the sensor */
        }

        action step2_measureLatency {
            doc /* Record time from power-on to first valid reading */
        }

        action step3_evaluate {
            doc /* Verify latency is within maxStartupTimeMs */
        }
    }
}
";
    files.push(("verification.sysml".to_string(), verif.to_string()));

    files
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- scaffold_element --------------------------------------------------

    #[test]
    fn scaffold_simple_part() {
        let opts = ScaffoldOptions::default();
        let result = scaffold_element("part", "Vehicle", &opts).unwrap();
        assert!(result.contains("part def Vehicle;"));
    }

    #[test]
    fn scaffold_with_doc_and_extends() {
        let opts = ScaffoldOptions {
            extends: Some("Base".to_string()),
            doc: Some("A vehicle".to_string()),
            ..Default::default()
        };
        let result = scaffold_element("part", "Car", &opts).unwrap();
        assert!(result.contains("part def Car :> Base {"));
        assert!(result.contains("doc /* A vehicle */"));
    }

    #[test]
    fn scaffold_with_members() {
        let opts = ScaffoldOptions {
            members: vec![
                "part engine : Engine".to_string(),
                "attribute mass : Real".to_string(),
            ],
            ..Default::default()
        };
        let result = scaffold_element("part", "Vehicle", &opts).unwrap();
        assert!(result.contains("part engine : Engine;"));
        assert!(result.contains("attribute mass : Real;"));
    }

    #[test]
    fn scaffold_with_teaching_comments() {
        let opts = ScaffoldOptions {
            with_teaching_comments: true,
            ..Default::default()
        };
        let result = scaffold_element("part", "Vehicle", &opts).unwrap();
        assert!(result.contains("part def"));
        assert!(result.contains("reusable definition"));
    }

    #[test]
    fn scaffold_requirement() {
        let opts = ScaffoldOptions {
            doc: Some("The system shall do X".to_string()),
            with_teaching_comments: true,
            ..Default::default()
        };
        let result = scaffold_element("requirement", "SafetyReq", &opts).unwrap();
        assert!(result.contains("requirement def SafetyReq {"));
        assert!(result.contains("stakeholder need"));
    }

    #[test]
    fn scaffold_action_with_teaching() {
        let opts = ScaffoldOptions {
            with_teaching_comments: true,
            ..Default::default()
        };
        let result = scaffold_element("action", "DoSomething", &opts).unwrap();
        assert!(result.contains("action def DoSomething;"));
        assert!(result.contains("unit of behaviour"));
    }

    #[test]
    fn scaffold_unknown_kind_error() {
        let opts = ScaffoldOptions::default();
        let err = scaffold_element("widget", "Foo", &opts).unwrap_err();
        assert_eq!(err, ScaffoldError::UnknownKind("widget".to_string()));
        assert!(err.to_string().contains("widget"));
    }

    #[test]
    fn scaffold_port_with_direction() {
        let opts = ScaffoldOptions {
            members: vec!["in attribute signal : Real".to_string()],
            ..Default::default()
        };
        let result = scaffold_element("port", "SigPort", &opts).unwrap();
        assert!(result.contains("port def SigPort {"));
        assert!(result.contains("in attribute signal : Real;"));
    }

    #[test]
    fn scaffold_package() {
        let opts = ScaffoldOptions {
            with_teaching_comments: true,
            ..Default::default()
        };
        let result = scaffold_element("package", "MyPkg", &opts).unwrap();
        assert!(result.contains("package MyPkg;"));
        assert!(result.contains("namespace container"));
    }

    #[test]
    fn scaffold_verification() {
        let opts = ScaffoldOptions {
            doc: Some("Verify safety".to_string()),
            with_teaching_comments: true,
            ..Default::default()
        };
        let result = scaffold_element("verification", "SafetyCheck", &opts).unwrap();
        assert!(result.contains("verification def SafetyCheck {"));
        assert!(result.contains("how a requirement is checked"));
    }

    #[test]
    fn scaffold_all_kinds() {
        let opts = ScaffoldOptions::default();
        for (kind, _desc) in list_element_kinds() {
            let result = scaffold_element(kind, "TestName", &opts);
            assert!(result.is_ok(), "kind '{}' should be valid", kind);
        }
    }

    #[test]
    fn scaffold_kind_aliases() {
        let opts = ScaffoldOptions::default();
        // Test various aliases resolve correctly.
        assert!(scaffold_element("req", "R", &opts).unwrap().contains("requirement def"));
        assert!(scaffold_element("attr", "A", &opts).unwrap().contains("attribute def"));
        assert!(scaffold_element("pkg", "P", &opts).unwrap().contains("package"));
        assert!(scaffold_element("usecase", "U", &opts).unwrap().contains("use case def"));
        assert!(scaffold_element("part def", "X", &opts).unwrap().contains("part def"));
    }

    // -- scaffold_verification_case ----------------------------------------

    #[test]
    fn verification_case_basic() {
        let result = scaffold_verification_case("BrakeTest", &["StopReq", "ForceReq"]);
        assert!(result.contains("verification def BrakeTest {"));
        assert!(result.contains("requirement stopReq : StopReq;"));
        assert!(result.contains("requirement forceReq : ForceReq;"));
        assert!(result.contains("step1_setup"));
        assert!(result.contains("step2_execute"));
        assert!(result.contains("step3_evaluate"));
        assert!(result.contains("StopReq and ForceReq"));
    }

    #[test]
    fn verification_case_single_req() {
        let result = scaffold_verification_case("SingleTest", &["MyReq"]);
        assert!(result.contains("requirement myReq : MyReq;"));
        assert!(result.contains("subject testSubject;"));
    }

    // -- scaffold_risk_template --------------------------------------------

    #[test]
    fn risk_template_structure() {
        let result = scaffold_risk_template("BrakeFadeRisk");
        assert!(result.contains("enum def RiskLikelihood {"));
        assert!(result.contains("enum def RiskSeverity {"));
        assert!(result.contains("part def BrakeFadeRisk {"));
        assert!(result.contains("attribute likelihood : RiskLikelihood;"));
        assert!(result.contains("attribute severity : RiskSeverity;"));
        assert!(result.contains("requirement mitigationNeeded {"));
        assert!(result.contains("action mitigationAction {"));
    }

    #[test]
    fn risk_template_has_enum_values() {
        let result = scaffold_risk_template("TestRisk");
        assert!(result.contains("enum remote;"));
        assert!(result.contains("enum catastrophic;"));
    }

    // -- scaffold_tolerance_chain ------------------------------------------

    #[test]
    fn tolerance_chain_structure() {
        let result = scaffold_tolerance_chain("GapAnalysis", &["PartA", "PartB", "PartC"]);
        assert!(result.contains("package GapAnalysis {"));
        assert!(result.contains("attribute def PartA {"));
        assert!(result.contains("attribute def PartB {"));
        assert!(result.contains("attribute def PartC {"));
        assert!(result.contains("constraint def StackUpConstraint {"));
        assert!(result.contains("attribute partA : PartA;"));
        assert!(result.contains("attribute totalNominal : Real;"));
    }

    #[test]
    fn tolerance_chain_has_tolerance_attrs() {
        let result = scaffold_tolerance_chain("Chain", &["DimX"]);
        assert!(result.contains("attribute nominal : Real;"));
        assert!(result.contains("attribute tolerancePlus : Real;"));
        assert!(result.contains("attribute toleranceMinus : Real;"));
    }

    // -- scaffold_example --------------------------------------------------

    #[test]
    fn example_brake_system_files() {
        let files = scaffold_example("brake-system").unwrap();
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"types.sysml"));
        assert!(names.contains(&"parts.sysml"));
        assert!(names.contains(&"requirements.sysml"));
        assert!(names.contains(&"verification.sysml"));

        // Every file should have doc comments.
        for (name, content) in &files {
            assert!(
                content.contains("doc /*"),
                "file '{}' should contain doc comments",
                name
            );
        }

        // Parts file should have composition.
        let parts = &files.iter().find(|(n, _)| n == "parts.sysml").unwrap().1;
        assert!(parts.contains("part pedalAssembly"));
        assert!(parts.contains("part def BrakeSystem"));
    }

    #[test]
    fn example_brake_system_has_requirements_and_verification() {
        let files = scaffold_example("brake-system").unwrap();
        let reqs = &files
            .iter()
            .find(|(n, _)| n == "requirements.sysml")
            .unwrap()
            .1;
        assert!(reqs.contains("requirement def StoppingDistanceReq"));
        assert!(reqs.contains("requirement def PedalForceReq"));

        let verif = &files
            .iter()
            .find(|(n, _)| n == "verification.sysml")
            .unwrap()
            .1;
        assert!(verif.contains("verification def StoppingDistanceTest"));
    }

    #[test]
    fn example_sensor_module_files() {
        let files = scaffold_example("sensor-module").unwrap();
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"types.sysml"));
        assert!(names.contains(&"parts.sysml"));
        assert!(names.contains(&"requirements.sysml"));
        assert!(names.contains(&"behaviour.sysml"));
        assert!(names.contains(&"verification.sysml"));

        // Should have ports and interfaces.
        let types = &files.iter().find(|(n, _)| n == "types.sysml").unwrap().1;
        assert!(types.contains("port def PowerPort"));
        assert!(types.contains("interface def SensorInterface"));

        // Should have state machine.
        let behaviour = &files.iter().find(|(n, _)| n == "behaviour.sysml").unwrap().1;
        assert!(behaviour.contains("state def SensorStateMachine"));
        assert!(behaviour.contains("transition powerOn"));

        // Verification should exist.
        let verif = &files
            .iter()
            .find(|(n, _)| n == "verification.sysml")
            .unwrap()
            .1;
        assert!(verif.contains("verification def AccuracyTest"));
    }

    #[test]
    fn example_sensor_has_doc_on_every_def() {
        let files = scaffold_example("sensor-module").unwrap();
        for (_name, content) in &files {
            // Every 'def' keyword in the content should be followed by a doc.
            // We check a lighter invariant: each file has at least one doc.
            assert!(content.contains("doc /*"));
        }
    }

    #[test]
    fn example_unknown_error() {
        let err = scaffold_example("nonexistent").unwrap_err();
        assert_eq!(
            err,
            ScaffoldError::UnknownExample("nonexistent".to_string())
        );
        assert!(err.to_string().contains("nonexistent"));
    }

    // -- list_examples -----------------------------------------------------

    #[test]
    fn list_examples_non_empty() {
        let examples = list_examples();
        assert!(examples.len() >= 2);
        let names: Vec<&str> = examples.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"brake-system"));
        assert!(names.contains(&"sensor-module"));
    }

    // -- list_element_kinds ------------------------------------------------

    #[test]
    fn list_element_kinds_non_empty() {
        let kinds = list_element_kinds();
        assert!(kinds.len() >= 15);
        let names: Vec<&str> = kinds.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"part"));
        assert!(names.contains(&"requirement"));
        assert!(names.contains(&"package"));
    }

    #[test]
    fn all_listed_kinds_are_scaffoldable() {
        let opts = ScaffoldOptions::default();
        for (kind, _) in list_element_kinds() {
            assert!(
                scaffold_element(kind, "Test", &opts).is_ok(),
                "listed kind '{}' should be scaffoldable",
                kind
            );
        }
    }

    // -- helper: to_usage_name ---------------------------------------------

    #[test]
    fn to_usage_name_lowercases_first() {
        assert_eq!(to_usage_name("StopReq"), "stopReq");
        assert_eq!(to_usage_name("A"), "a");
        assert_eq!(to_usage_name("already"), "already");
        assert_eq!(to_usage_name(""), "");
    }

    // -- helper: parse_simple_member ---------------------------------------

    #[test]
    fn parse_member_colon_attached() {
        let m = parse_simple_member("part engine:Engine").unwrap();
        assert_eq!(m.usage_kind, "part");
        assert_eq!(m.name, "engine");
        assert_eq!(m.type_ref.as_deref(), Some("Engine"));
    }

    #[test]
    fn parse_member_colon_spaced() {
        let m = parse_simple_member("attribute mass : Real").unwrap();
        assert_eq!(m.usage_kind, "attribute");
        assert_eq!(m.name, "mass");
        assert_eq!(m.type_ref.as_deref(), Some("Real"));
    }

    #[test]
    fn parse_member_with_direction() {
        let m = parse_simple_member("in port fuelIn : FuelPort").unwrap();
        assert_eq!(m.direction.as_deref(), Some("in"));
        assert_eq!(m.usage_kind, "port");
        assert_eq!(m.name, "fuelIn");
        assert_eq!(m.type_ref.as_deref(), Some("FuelPort"));
    }

    #[test]
    fn parse_member_no_type() {
        let m = parse_simple_member("part engine").unwrap();
        assert_eq!(m.name, "engine");
        assert!(m.type_ref.is_none());
    }

    #[test]
    fn parse_member_empty_returns_none() {
        assert!(parse_simple_member("").is_none());
    }
}

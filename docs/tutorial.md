# Tutorial: Building and Managing a SysML v2 Model

Build a complete systems engineering model from scratch using the `sysml` interactive wizard. You'll model a **weather station** — an embedded system with sensors, a controller, and a display — then validate, diagram, simulate, and run lifecycle management on it.

> **Two ways to use sysml:** The interactive wizard (`sysml add`) is the primary workflow — it guides you through creating elements with model-aware suggestions. Every wizard action has an equivalent flag-based command for scripting and CI. This tutorial shows both.

## Prerequisites

```sh
git clone --recurse-submodules https://github.com/jackhale98/sysml-cli.git
cd sysml-cli && cargo install --path crates/sysml-cli
sysml --version
```

## Part 1: Project Setup

```sh
mkdir weather-station && cd weather-station
sysml init
```

This creates `.sysml/config.toml`. If a `libraries/` directory exists, it is automatically configured for import resolution.

Copy the domain libraries into your project:

```sh
cp -r /path/to/sysml-cli/libraries .
sysml init --force    # Re-init to detect libraries/
```

Explore built-in help:

```sh
sysml guide                    # list topics
sysml guide getting-started
```

## Part 2: Building the Model

In SysML v2, **definitions** (`part def Sensor`) are reusable types. **Usages** (`part tempSensor : Sensor`) are instances placed inside an assembly.

### 2.1 Create the model file

```sh
sysml add --stdout package WeatherStation --doc "Weather station model" > model.sysml
```

> This is the one case where `--stdout >` is needed — we're creating the file itself. Everything else uses `sysml add model.sysml`.

### 2.2 Interactive mode — the fastest way to build

Launch the wizard with a file to get model-aware suggestions:

```
$ sysml add model.sysml
? Where will this element go? > Add to an existing file
? What are you creating? > Enumeration
? Name: SensorStatus
? Enum members (comma-separated): ok,degraded,failed

Preview:
  enum def SensorStatus {
      enum ok;
      enum degraded;
      enum failed;
  }

Wrote SensorStatus to model.sysml
```

Repeat for `DisplayMode`:

```
$ sysml add model.sysml
? What are you creating? > Enumeration
? Name: DisplayMode
? Enum members: summary,detailed,alert
```

### 2.3 Add port definitions

Using flags (faster for known structures):

```sh
sysml add model.sysml port-def SensorDataPort -m "out item reading:ScalarValues::Real"
sysml add model.sysml port-def DisplayDataPort -m "in item displayValue:ScalarValues::Real"
sysml add model.sysml port-def PowerPort -m "in item voltage:ScalarValues::Real"
```

### 2.4 Add part definitions

Create an abstract base sensor:

```sh
sysml add model.sysml part-def Sensor --abstract \
    --doc "Base type for all sensors" \
    -m "attribute status:SensorStatus,attribute sampleRate:ScalarValues::Real" \
    -m "port dataOut:SensorDataPort,port power:PowerPort"
```

Specialize it (the wizard shows available supertypes from your model):

```sh
sysml add model.sysml part-def TemperatureSensor --extends Sensor \
    --doc "Measures ambient temperature" \
    -m "attribute range_min:ScalarValues::Real,attribute range_max:ScalarValues::Real"

sysml add model.sysml part-def HumiditySensor --extends Sensor \
    --doc "Measures relative humidity"

sysml add model.sysml part-def PressureSensor --extends Sensor \
    --doc "Measures barometric pressure"

sysml add model.sysml part-def WindSensor --extends Sensor \
    --doc "Measures wind speed and direction"
```

Remaining components:

```sh
sysml add model.sysml part-def Controller \
    --doc "Central processing unit" \
    -m "port tempIn:SensorDataPort,port humidIn:SensorDataPort" \
    -m "port pressIn:SensorDataPort,port windIn:SensorDataPort" \
    -m "port displayOut:DisplayDataPort,port power:PowerPort"

sysml add model.sysml part-def Display \
    --doc "LCD display" \
    -m "port dataIn:DisplayDataPort,port power:PowerPort" \
    -m "attribute mode:DisplayMode"

sysml add model.sysml part-def PowerSupply \
    --doc "Solar-powered battery pack" \
    -m "attribute capacity_ah:ScalarValues::Real,attribute voltage:ScalarValues::Real"

sysml add model.sysml part-def Enclosure \
    --doc "Weather-resistant housing" \
    -m "attribute ip_rating:ScalarValues::String"
```

### 2.5 Build the assembly with part usages

Create the top-level assembly definition, then add part usages inside it:

```sh
sysml add model.sysml part-def WeatherStationUnit \
    --doc "Complete weather station assembly"

# Part usages (instances inside the assembly)
sysml add model.sysml part tempSensor -t TemperatureSensor --inside WeatherStationUnit
sysml add model.sysml part humiditySensor -t HumiditySensor --inside WeatherStationUnit
sysml add model.sysml part pressureSensor -t PressureSensor --inside WeatherStationUnit
sysml add model.sysml part windSensor -t WindSensor --inside WeatherStationUnit
sysml add model.sysml part controller -t Controller --inside WeatherStationUnit
sysml add model.sysml part display -t Display --inside WeatherStationUnit
sysml add model.sysml part power -t PowerSupply --inside WeatherStationUnit
sysml add model.sysml part enclosure -t Enclosure --inside WeatherStationUnit
```

### 2.6 Add connections

```sh
sysml add model.sysml connection tempConn \
    --connect "tempSensor.dataOut to controller.tempIn" --inside WeatherStationUnit

sysml add model.sysml connection humidConn \
    --connect "humiditySensor.dataOut to controller.humidIn" --inside WeatherStationUnit

sysml add model.sysml connection pressConn \
    --connect "pressureSensor.dataOut to controller.pressIn" --inside WeatherStationUnit

sysml add model.sysml connection windConn \
    --connect "windSensor.dataOut to controller.windIn" --inside WeatherStationUnit

sysml add model.sysml connection displayConn \
    --connect "controller.displayOut to display.dataIn" --inside WeatherStationUnit
```

### 2.7 Add a state machine

```sh
sysml add model.sysml state-def StationStates \
    --doc "Operating states" \
    -m "entry; then off;" \
    -m "state off,state initializing,state monitoring,state alerting,state lowPower" \
    -m "transition first off accept powerOn then initializing" \
    -m "transition first initializing then monitoring" \
    -m "transition first monitoring accept alertTrigger then alerting" \
    -m "transition first alerting accept clearAlert then monitoring" \
    -m "transition first monitoring accept lowBattery then lowPower" \
    -m "transition first lowPower accept charged then monitoring"
```

### 2.8 Add an action definition

```sh
sysml add model.sysml action-def ReadSensors \
    --doc "Read all sensors and update display" \
    -m "action readTemp,action readHumidity,action readPressure,action readWind" \
    -m "action processData,action updateDisplay" \
    -m "first readTemp then readHumidity" \
    -m "first readHumidity then readPressure" \
    -m "first readPressure then readWind" \
    -m "first readWind then processData" \
    -m "first processData then updateDisplay"
```

### 2.9 Add constraints and calculations

```sh
sysml add model.sysml constraint-def TemperatureLimit \
    --doc "Operating temperature range" \
    -m "in attribute temp:ScalarValues::Real" \
    -m "constraint temp >= -40 and temp <= 60"

sysml add model.sysml constraint-def PowerBudget \
    --doc "Maximum power consumption" \
    -m "in attribute consumption:ScalarValues::Real" \
    -m "constraint consumption <= 500"

sysml add model.sysml calc-def BatteryRuntime \
    --doc "Calculate battery runtime in hours" \
    -m "in attribute capacity:ScalarValues::Real" \
    -m "in attribute consumption:ScalarValues::Real" \
    -m "return hours:ScalarValues::Real"
```

### 2.10 Validate and explore

```sh
sysml lint model.sysml
sysml list model.sysml
sysml list --kind parts model.sysml
sysml list --parent WeatherStationUnit model.sysml
sysml show model.sysml WeatherStationUnit
sysml stats model.sysml
```

## Part 3: Requirements and Traceability

### 3.1 Create the requirements file

```sh
sysml add --stdout package WeatherStationRequirements \
    --doc "Weather station requirements" > requirements.sysml

sysml add requirements.sysml import "WeatherStation::*"
```

### 3.2 Add requirements

```sh
sysml add requirements.sysml requirement TemperatureAccuracy \
    --doc "Temperature sensor shall measure with +/- 0.5C accuracy"

sysml add requirements.sysml requirement OperatingRange \
    --doc "Station shall operate from -40C to +60C"

sysml add requirements.sysml requirement BatteryLife \
    --doc "Station shall operate 72 hours without solar charging"

sysml add requirements.sysml requirement UpdateRate \
    --doc "Display shall update readings every 5 seconds"

sysml add requirements.sysml requirement IPRating \
    --doc "Enclosure shall achieve IP65 or higher"
```

### 3.3 Link requirements to implementation

```sh
sysml add requirements.sysml satisfy TemperatureAccuracy --by TemperatureSensor
sysml add requirements.sysml satisfy OperatingRange --by WeatherStationUnit
sysml add requirements.sysml satisfy BatteryLife --by PowerSupply
sysml add requirements.sysml satisfy UpdateRate --by Controller
sysml add requirements.sysml satisfy IPRating --by Enclosure
```

### 3.4 Traceability matrix

```sh
$ sysml trace requirements.sysml
Requirement          Satisfied By         Verified By
------------------------------------------------------------
TemperatureAccuracy  TemperatureSensor    -
OperatingRange       WeatherStationUnit   -
BatteryLife          PowerSupply          -
UpdateRate           Controller           -
IPRating             Enclosure            -

Coverage: 5/5 satisfied (100%), 0/5 verified (0%)
```

CI gate: `sysml trace --check --min-coverage 80 requirements.sysml`

## Part 4: Verification

### 4.1 Create verification file

```sh
sysml add --stdout package WeatherStationVerification \
    --doc "Verification cases" > verification.sysml

sysml add verification.sysml import "WeatherStation::*"
sysml add verification.sysml import "WeatherStationRequirements::*"
```

### 4.2 Add verification cases

```sh
sysml add verification.sysml verification-def TestTemperatureAccuracy \
    --doc "Verify temperature sensor accuracy against reference thermometer" \
    -m "subject testSubject" \
    -m "requirement tempReq:TemperatureAccuracy"

sysml add verification.sysml verification-def TestOperatingRange \
    --doc "Environmental chamber test across full temperature range" \
    -m "subject testSubject" \
    -m "requirement rangeReq:OperatingRange"

sysml add verification.sysml verification-def TestBatteryLife \
    --doc "Continuous operation test without solar input" \
    -m "subject testSubject" \
    -m "requirement batteryReq:BatteryLife"
```

Link verification to requirements:

```sh
sysml add verification.sysml verify TemperatureAccuracy --by TestTemperatureAccuracy
sysml add verification.sysml verify OperatingRange --by TestOperatingRange
sysml add verification.sysml verify BatteryLife --by TestBatteryLife
```

### 4.3 Check verification coverage

```sh
sysml verify coverage verification.sysml requirements.sysml
sysml verify list verification.sysml
sysml verify status verification.sysml requirements.sysml
```

### 4.4 Execute a verification case

```sh
$ sysml verify run verification.sysml --case TestTemperatureAccuracy --author "Jane Smith"
Verification Case: TestTemperatureAccuracy
  Verifies: TemperatureAccuracy
  Steps: 3

? Step 1 - Setup: Reference thermometer calibrated? [Y/n] Y
? Step 2 - Execute: Measured accuracy (C): 0.3
? Step 3 - Evaluate: Within +/- 0.5C? [Y/n] Y

Result: PASS
Measurements:
  accuracy = 0.3 C [OK]

Record written: .sysml/records/verify-execution-20260310-JaneSmith-ab12.toml
```

## Part 5: Diagrams

```sh
sysml diagram -t bdd model.sysml                              # Block definition
sysml diagram -t ibd --scope WeatherStationUnit model.sysml    # Internal blocks
sysml diagram -t stm --scope StationStates model.sysml         # State machine
sysml diagram -t act --scope ReadSensors model.sysml           # Activity
sysml diagram -t req requirements.sysml                        # Requirements
sysml diagram -t pkg model.sysml                               # Package
sysml diagram -t trace model.sysml                             # V-model traceability
```

Other output formats:

```sh
sysml diagram -t bdd -o plantuml model.sysml
sysml diagram -t bdd -o dot model.sysml
sysml diagram -t bdd -o d2 model.sysml
```

## Part 6: Simulation

### 6.1 State machine simulation

```sh
$ sysml simulate sm model.sysml -n StationStates -e powerOn,alertTrigger,clearAlert
State Machine: StationStates
Initial state: off
  Step 0: off -- [powerOn]--> initializing
  Step 1: initializing --> monitoring
  Step 2: monitoring -- [alertTrigger]--> alerting
  Step 3: alerting -- [clearAlert]--> monitoring
```

Without `-e`, it prompts interactively for events.

### 6.2 Constraint evaluation

```sh
$ sysml simulate eval model.sysml -n TemperatureLimit -b temp=25
constraint TemperatureLimit: satisfied

$ sysml simulate eval model.sysml -n TemperatureLimit -b temp=70
constraint TemperatureLimit: violated

$ sysml simulate eval model.sysml -n BatteryRuntime -b capacity=12,consumption=200
calc BatteryRuntime: 60
```

### 6.3 Action flow

```sh
sysml simulate af model.sysml -n ReadSensors
```

## Part 7: Analysis

```sh
sysml deps model.sysml WeatherStationUnit           # What does it depend on?
sysml deps model.sysml TemperatureSensor --reverse   # What uses it?
sysml interfaces --unconnected model.sysml           # Find unconnected ports
sysml allocation model.sysml                         # Allocation matrix
sysml coverage model.sysml                           # Model completeness
sysml diff model.sysml model-v2.sysml                # Semantic diff
```

## Part 8: Risk Management

### 8.1 Create risks interactively

```sh
$ sysml risk add
? Risk title: Moisture ingress
? Severity: > critical
? Likelihood: > remote
? Detectability: > moderate
? Category: > safety

Preview:
  part riskMoistureIngress : RiskDef {
      doc /* Moisture ingress */
      attribute redefines severity = SeverityLevel::critical;
      attribute redefines likelihood = LikelihoodLevel::remote;
      attribute redefines detectability = DetectabilityLevel::moderate;
  }
  RPN: 60 (4 x 2 x 3)

? Write to risks.sysml? [Y/n]
```

Or with flags:

```sh
sysml add --stdout package WeatherStationRisks --doc "Risk register" > risks.sysml
sysml add risks.sysml import "WeatherStation::*"
sysml risk add --file risks.sysml
```

### 8.2 Analyze risks

```sh
$ sysml risk list risks.sysml
Risks (2):
  riskMoistureIngress  [S:Critical L:Remote RPN:60]
  riskSolarFailure     [S:Moderate L:Occasional RPN:36]

$ sysml risk matrix risks.sysml
              Improbable  Remote     Occasional  Probable  Frequent
Negligible         -         -           -          -         -
Marginal           -         -           -          -         -
Moderate           -         -    riskSolarFail     -         -
Critical           -    riskMoisture    -          -         -
Catastrophic       -         -           -          -         -

$ sysml risk fmea risks.sysml
Failure Mode              S    L    D  RPN Mitigation     Status
riskMoistureIngress      4    2    3   60 -              open
riskSolarFailure         3    3    4   36 -              open
```

## Part 9: Manufacturing and Quality

### 9.1 SPC analysis

```sh
$ sysml mfg spc --parameter SensorCalibration \
    --values 0.48,0.52,0.50,0.49,0.51,0.50,0.53,0.47
  Mean: 0.500  Std: 0.019  UCL: 0.557  LCL: 0.443
  All points within control limits
```

### 9.2 Production lot tracking

```sh
$ sysml mfg start-lot model.sysml --routing AssemblyProcess --quantity 100
Lot: mfg-lot-20260310-engineer-5a7b
  Routing:  AssemblyProcess
  Quantity: 100
  Progress: 0/5 steps
Record written: .sysml/records/mfg-lot-20260310-engineer-5a7b.toml

$ sysml mfg step
Lot: mfg-lot-20260310-engineer-5a7b — Step 1/5: Inspection
? Resistance reading: 4.7
? Insulation reading: 1500.0
Readings: Resistance = 4.7 [OK], Insulation = 1500.0 [OK]
Step status: Passed
```

### 9.3 Quality management

```sh
# Sampling plans
$ sysml qc sample-size --lot-size 500
  AQL: 1.0  Level: Normal  Sample: 50  Accept: 5  Reject: 6

# Process capability
$ sysml qc capability --usl 10.05 --lsl 9.95 \
    --values 10.01,9.99,10.02,9.98,10.00,10.01,9.99,10.00
  Cp: 1.67  Cpk: 1.33  Process is capable

# NCR → RCA → CAPA workflow
$ sysml quality create --type ncr
? Affected part: Enclosure
? Category: > Dimensional
? Severity: > Major
? Description: IP seal gap exceeds tolerance
NCR Created: quality-ncr-20260310-engineer-7f3a
Record: .sysml/records/quality-ncr-20260310-engineer-7f3a.toml

$ sysml quality rca --source quality-ncr-20260310-engineer-7f3a --method five-why
? Why 1: IP seal gap exceeds tolerance → Tool wear on seal groove
? Why 2: Tool wear on seal groove → No scheduled tool replacement
? Why 3: No scheduled tool replacement → Missing PM schedule
? Why 4: Missing PM schedule → New tooling, no PM baseline
? Why 5: No PM baseline → Commissioning checklist incomplete
Root Cause: Commissioning checklist incomplete
Record: .sysml/records/quality-rca-20260310-engineer-2e4f.toml

$ sysml quality create --type capa
? Title: Add PM schedule for seal groove tooling
? Source: > Ncr
? CAPA type: > Corrective
CAPA Created: quality-capa-20260310-engineer-4b2c

$ sysml quality action --capa quality-capa-20260310-engineer-4b2c
? Action type: > Procedure Update
? Description: Create PM schedule with tool replacement intervals
? Owner: alice
? Due date: 2026-03-20
Action added to CAPA

$ sysml quality trend --group-by category
Category      Open  In Progress  Closed  Total
Dimensional      1            0       0      1
```

## Part 10: Editing

### 10.1 Add new elements

```sh
sysml add model.sysml part-def RainGauge --extends Sensor --doc "Measures rainfall"
sysml add model.sysml part rainGauge -t RainGauge --inside WeatherStationUnit
```

### 10.2 Preview and multiplicity

```sh
sysml add model.sysml part-def Vehicle \
    -m "part wheels:Wheel[4],attribute doors:Door[2..5]" --dry-run
```

### 10.3 Remove and rename

```sh
sysml remove model.sysml RainGauge --dry-run    # Preview
sysml remove model.sysml RainGauge              # Apply
sysml rename model.sysml WindSensor Anemometer
```

### 10.4 Learn SysML syntax

```sh
sysml add --stdout --teach part-def Motor
sysml add --stdout --teach state-def Lifecycle
```

## Part 11: Export and Reports

```sh
sysml export interfaces model.sysml --part Controller   # FMI 3.0
sysml export modelica model.sysml --part Controller      # Modelica
sysml export ssp model.sysml -o system.ssd               # SSP XML

sysml report dashboard model.sysml requirements.sysml verification.sysml
sysml report gate model.sysml requirements.sysml --gate-name CDR --min-coverage 80
```

## Part 12: Formatting and CI

```sh
sysml fmt model.sysml                   # Format in place
sysml fmt --check model.sysml           # CI mode
sysml fmt --diff model.sysml            # Preview changes

sysml pipeline run ci                   # Run validation pipeline
sysml lint -f json model.sysml          # JSON output for tooling
```

## Quick Reference

| Task | Interactive | Flags |
|------|------------|-------|
| Create any element | `sysml add` | `sysml add <file> <kind> <name>` |
| Add with model context | `sysml add <file>` | `sysml add <file> <kind> <name> --inside Parent` |
| Enum with members | wizard prompts | `sysml add <file> enum-def Color -m red,green,blue` |
| State machine | wizard prompts | `sysml add <file> state-def S -m "state idle,transition first idle accept go then running"` |
| Connection | wizard prompts | `sysml add <file> connection c --connect "a.x to b.y" --inside Assy` |
| Satisfy requirement | wizard prompts | `sysml add <file> satisfy Req --by Element` |
| Verify requirement | wizard prompts | `sysml add <file> verify Req --by TestCase` |
| Verification case | wizard prompts | `sysml add <file> verification-def Test --doc "..." -m "subject s"` |
| Import | wizard prompts | `sysml add <file> import "Pkg::*"` |
| Remove element | — | `sysml remove <file> Name` |
| Rename element | — | `sysml rename <file> Old New` |
| Generate to stdout | — | `sysml add --stdout <kind> <name>` |
| Risk creation | `sysml risk add` | `sysml risk add --file risks.sysml` |
| Verify execution | `sysml verify run <files>` | `sysml verify run <files> --case Name` |
| NCR creation | `sysml quality create --type ncr` | — |
| Lot tracking | `sysml mfg start-lot` | `sysml mfg start-lot <files> --routing Name` |

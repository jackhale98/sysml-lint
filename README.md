# sysml

A fast, standalone SysML v2 command-line toolchain for model validation, simulation, diagram generation, and full product lifecycle management.

Built on [tree-sitter](https://tree-sitter.github.io/) for reliable parsing of SysML v2 textual notation. Zero runtime dependencies — just a single binary.

## Documentation

| | |
|---|---|
| [Tutorial](docs/tutorial.md) | Build a weather station model from scratch using the CLI |
| [Validation & Diagnostics](docs/validation.md) | 9 lint checks, diagnostic codes, output formats |
| [Domain Libraries](docs/domain-libraries.md) | Base types for risk, tolerance, BOM, manufacturing, quality |
| [Architecture](docs/architecture.md) | Crate structure, design decisions, 12-crate workspace |
| [CI & Editor Integration](docs/ci-integration.md) | GitHub Actions workflow, Emacs sysml2-mode, JSON output |
| **Command references** | [Analysis](docs/commands/analysis.md) &#183; [Diagrams](docs/commands/diagrams.md) &#183; [Editing](docs/commands/editing.md) &#183; [Simulation](docs/commands/simulation.md) &#183; [Lifecycle](docs/commands/lifecycle.md) &#183; [Project](docs/commands/project.md) |

## Installation

### From source

```sh
git clone --recurse-submodules https://github.com/jackhale98/sysml-cli.git
cd sysml-cli
cargo install --path crates/sysml-cli
```

Or build manually:

```sh
cargo build --release
cp target/release/sysml ~/.local/bin/
```

The build compiles the [tree-sitter-sysml](https://github.com/jackhale98/tree-sitter-sysml) grammar from source (included as a submodule). Requires Rust 1.70+ and a C compiler (gcc or clang).

To enable the optional SQLite-backed persistent cache:

```sh
cargo install --path crates/sysml-cli --features sqlite
```

### Shell completions

```sh
sysml completions bash > ~/.local/share/bash-completion/completions/sysml
sysml completions zsh > ~/.zfunc/_sysml
sysml completions fish > ~/.config/fish/completions/sysml.fish
```

## Quick Start

```sh
sysml lint model.sysml                          # Validate a model
sysml list model.sysml                          # List all elements
sysml show model.sysml Vehicle                  # Element details
sysml diagram -t bdd model.sysml                # Block definition diagram
sysml diagram -t trace model.sysml              # Traceability diagram
sysml simulate state-machine model.sysml        # Interactive state machine
sysml fmt model.sysml                           # Format source
sysml add --stdout part-def Vehicle             # Generate template to stdout
sysml add model.sysml part-def Engine           # Insert into file
sysml add                                       # Interactive wizard
sysml init                                      # Initialize a project
sysml risk matrix model.sysml                   # Risk matrix
sysml tol analyze model.sysml --method rss      # Tolerance stack-up
sysml bom rollup model.sysml --root Vehicle     # BOM rollup
sysml quality create --type ncr                 # Create NCR interactively
sysml report dashboard model.sysml              # Project health
```

## Highlights

### Guided model authoring — no SysML syntax required

`sysml add` launches a concept-first wizard that speaks domain vocabulary, not grammar rules. Pick what you're building ("a new type: physical component"), give it a name, and the tool generates valid SysML v2:

```sh
$ sysml add
? What are you creating?
  > A new type: physical component or assembly
? Name: Controller
? Brief description: Central processing unit for sensor data
? Extend another type? Sensor

# Generates:
part def Controller :> Sensor {
    doc /* Central processing unit for sensor data */
}
```

Power users skip the wizard entirely: `sysml add model.sysml part-def Controller --extends Sensor`

### 10 diagram types, 4 output formats

Generate BDD, IBD, state machine, activity, requirements, package, parametric, traceability, allocation, and use case diagrams — output as Mermaid, PlantUML, Graphviz DOT, or D2:

```sh
sysml diagram -t bdd model.sysml                        # block definition diagram
sysml diagram -t ibd --scope Vehicle model.sysml         # internal block diagram
sysml diagram -t stm --scope StateMachine -o d2 model.sysml
sysml diagram -t trace -o plantuml requirements.sysml    # V-model traceability
```

### Simulate state machines and evaluate constraints

Step through state machines interactively or with scripted events. Evaluate constraints and calculations with variable bindings:

```sh
$ sysml simulate sm model.sysml -n StationStates -e powerOn,alertTrigger,clearAlert
State Machine: StationStates
Initial state: off
  Step 0: off -- [powerOn]--> initializing
  Step 1: initializing --> monitoring
  Step 2: monitoring -- [alertTrigger]--> alerting
  Step 3: alerting -- [clearAlert]--> monitoring

$ sysml simulate eval constraints.sysml -n PowerBudget -b consumption=450
constraint PowerBudget: satisfied
```

### Semantic diff — compare models, not text

```sh
$ sysml diff model-v1.sysml model-v2.sysml
  Added:   part def RainGauge :> Sensor
  Removed: attribute maxSpeed in WindSensor
  Changed: TemperatureSensor.range_max (line 42 → 45)
```

### Manufacturing SPC and process capability

Run statistical process control analysis with control charts, and compute Cp/Cpk capability indices:

```sh
$ sysml mfg spc --parameter SensorCalibration --values 0.48,0.52,0.50,0.49,0.51,0.50,0.53,0.47
  Mean: 0.500  Std: 0.019  UCL: 0.557  LCL: 0.443
  All points within control limits ✓

$ sysml qc capability --usl 10.05 --lsl 9.95 --values 10.01,9.99,10.02,9.98,10.00
  Cp: 1.67  Cpk: 1.33  Process is capable ✓
```

### CI pipelines from config

Define named validation pipelines in `.sysml/config.toml` and run them locally or in CI. Stops at the first failure:

```toml
[[pipeline]]
name = "ci"
steps = [
    "lint model.sysml requirements.sysml",
    "fmt --check model.sysml",
    "trace --check --min-coverage 80 requirements.sysml",
]
```

```sh
sysml pipeline run ci
```

### Full lifecycle in one tool

Risk matrices, FMEA, tolerance stack-ups (worst-case/RSS/Monte Carlo), BOM rollups, supplier RFQs, verification execution, NCR/CAPA/Deviation tracking — all driven from SysML models with domain library types:

```sh
sysml risk matrix model.sysml -I libraries/
sysml tol analyze model.sysml --method monte-carlo --iterations 50000
sysml bom rollup model.sysml --root Vehicle --include-mass --include-cost
sysml verify run verification.sysml --case TestAccuracy
sysml quality create --type ncr          # interactive NCR wizard
sysml quality rca --source NCR-001 --method fishbone
```

### Global Options

| Flag | Description |
|------|-------------|
| `-f, --format <FORMAT>` | Output format: `text`, `json` (default: `text`) |
| `-q, --quiet` | Suppress summary line on stderr |
| `-I, --include <PATH>` | Additional files/directories for import resolution |
| `--stdlib-path <PATH>` | Path to the SysML v2 standard library directory (env: `SYSML_STDLIB_PATH`, config: `stdlib_path`) |

## Commands

| Command | Description | Docs |
|---------|-------------|------|
| **Analysis** | | [analysis](docs/commands/analysis.md) |
| `lint` | Validate SysML v2 files against structural rules | |
| `list` (`ls`) | List model elements with filters | |
| `show` | Show detailed element information | |
| `check` | Validate models and project integrity | |
| `trace` | Requirements traceability matrix | |
| `interfaces` | Analyze port interfaces and connections | |
| `deps` | Dependency analysis for an element | |
| `diff` | Semantic diff between two SysML files | |
| `allocation` | Logical-to-physical allocation matrix | |
| `coverage` | Model quality and completeness report | |
| `stats` | Aggregate model statistics | |
| **Diagrams** | | [diagrams](docs/commands/diagrams.md) |
| `diagram` | Generate diagrams (bdd, ibd, stm, act, req, pkg, par, trace, alloc, ucd) | |
| **Editing** | | [editing](docs/commands/editing.md) |
| `add` | Add elements interactively, to a file, or to stdout | |
| `remove` | Remove an element from a SysML file | |
| `rename` | Rename an element and update all references | |
| `example` | Generate example projects with teaching comments | |
| `fmt` | Format SysML v2 source files | |
| **Simulation & Export** | | [simulation](docs/commands/simulation.md) |
| `simulate` | Evaluate constraints, state machines, action flows | |
| `export` | Export FMI 3.0, Modelica, SSP artifacts | |
| **Lifecycle** | | [lifecycle](docs/commands/lifecycle.md) |
| `verify` | Verification case management, coverage, interactive execution | |
| `risk` | Risk management, matrix, FMEA, interactive risk creation | |
| `tol` | Tolerance stack-up analysis (worst-case, RSS, Monte Carlo) | |
| `bom` | Bill of materials rollup, where-used, export | |
| `source` | Supplier management, RFQ, approved source lists | |
| `mfg` | Manufacturing routings, SPC, lot tracking, step execution | |
| `qc` | Quality control, sampling plans, Cp/Cpk | |
| `quality` | Quality management (NCR, CAPA, Process Deviation, RCA) | |
| **Project** | | [project](docs/commands/project.md) |
| `init` | Initialize a `.sysml/` project | |
| `index` | Build or rebuild project index | |
| `pipeline` | Run named validation pipelines from config | |
| `report` | Cross-domain reports (dashboard, traceability, gate) | |
| `guide` | Built-in help topics and tutorials | |
| `completions` | Generate shell completion scripts | |

## Domain Libraries

The tool ships with SysML v2 domain libraries that provide base types for lifecycle workflows. Users specialize these types in their models — the tool recognizes all specializations automatically.

| Library | Package | Purpose |
|---------|---------|---------|
| `sysml-verification-ext.sysml` | `SysMLVerification` | Verification status, methods, acceptance criteria |
| `sysml-risk.sysml` | `SysMLRisk` | Severity/likelihood enums, RiskDef, MitigationDef |
| `sysml-tolerance.sysml` | `SysMLTolerance` | ToleranceDef, DimensionChainDef, GD&T types |
| `sysml-bom.sysml` | `SysMLBOM` | PartIdentity, MassProperty, SupplierDef |
| `sysml-manufacturing.sysml` | `SysMLManufacturing` | ProcessDef, RoutingDef, WorkInstructionDef |
| `sysml-quality.sysml` | `SysMLQuality` | InspectionPlanDef, GaugeRRDef, sampling |
| `sysml-capa.sysml` | `SysMLCAPA` | NCR/CAPA/Deviation lifecycles, categories, dispositions |
| `sysml-project.sysml` | `SysMLProject` | Phase gates, milestone definitions |

See [Domain Libraries](docs/domain-libraries.md) for detailed type references and usage patterns.

## License

GPL-3.0-or-later

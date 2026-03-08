# sysml-lint

A fast, standalone SysML v2 model validator, linter, and simulator for CI pipelines and editor integration.

Built on [tree-sitter](https://tree-sitter.github.io/) for reliable parsing of SysML v2 textual notation. Produces structured diagnostics in text or JSON format with configurable severity filtering. Includes a built-in simulation engine for constraints, calculations, state machines, and action flows.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Linting](#linting)
- [Simulation](#simulation)
- [Checks](#checks)
- [Diagnostic Codes](#diagnostic-codes)
- [Output Formats](#output-formats)
- [CI Integration](#ci-integration)
- [Editor Integration](#editor-integration)
- [Building from Source](#building-from-source)
- [Architecture](#architecture)
- [License](#license)

## Installation

### From source

```sh
git clone https://github.com/jackhale98/sysml-lint.git
cd sysml-lint
cargo build --release
cp target/release/sysml-lint ~/.local/bin/
```

The build compiles the [tree-sitter-sysml](https://github.com/jackhale98/tree-sitter-sysml) grammar from source. The grammar must be available at `./tree-sitter-sysml/src/` (vendored) or `../tree-sitter-sysml/src/` (sibling directory).

## Usage

sysml-lint has two subcommands: `lint` and `simulate`.

```sh
# Lint SysML files
sysml-lint lint model.sysml

# Simulate constructs
sysml-lint simulate list model.sysml
```

### Global Options

```
-f, --format <FORMAT>  Output format: text, json [default: text]
-q, --quiet            Suppress summary line on stderr
-h, --help             Print help
-V, --version          Print version
```

## Linting

```sh
# Lint a single file
sysml-lint lint model.sysml

# Lint multiple files
sysml-lint lint src/*.sysml

# JSON output for tooling
sysml-lint lint --format json model.sysml

# Only show warnings and errors
sysml-lint lint --severity warning model.sysml

# Disable specific checks
sysml-lint lint --disable unused,unresolved model.sysml
```

### Lint Options

```
sysml-lint lint [OPTIONS] <FILES>...

Arguments:
  <FILES>...  SysML v2 files to validate

Options:
  -d, --disable <DISABLE>    Disable specific checks (comma-separated)
  -s, --severity <SEVERITY>  Minimum severity: note, warning, error [default: note]
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No errors found (may have warnings or notes) |
| 1 | One or more errors found, or a file could not be read |

## Simulation

sysml-lint includes a built-in simulation engine that can evaluate constraints, run calculations, simulate state machines, and execute action flows — all from the command line.

### List Simulatable Constructs

```sh
sysml-lint simulate list model.sysml
```

Output:
```
Constraints:
  SpeedLimit (speed: Real)

Calculations:
  KineticEnergy (mass: Real, velocity: Real) -> Real

State Machines:
  TrafficLight [entry: red, states: red, yellow, green, transitions: 3]

Actions:
  ProcessOrder (7 steps)
```

### Evaluate Constraints and Calculations

```sh
# Evaluate a constraint
sysml-lint simulate eval model.sysml -b speed=100 -n SpeedLimit
# Output: constraint SpeedLimit: satisfied

sysml-lint simulate eval model.sysml -b speed=150 -n SpeedLimit
# Output: constraint SpeedLimit: violated

# Evaluate a calculation
sysml-lint simulate eval model.sysml -b mass=1500,velocity=30 -n KineticEnergy
# Output: calc KineticEnergy: 675000

# Evaluate all constraints and calcs
sysml-lint simulate eval model.sysml -b speed=100,mass=1500

# JSON output
sysml-lint -f json simulate eval model.sysml -b speed=100
```

### Simulate State Machines

```sh
# Simulate with events
sysml-lint simulate state-machine model.sysml -n TrafficLight -e next,next,next

# Output:
# State Machine: TrafficLight
# Initial state: red
#
#   Step 0: red -- [next]--> green
#   Step 1: green -- [next]--> yellow
#   Step 2: yellow -- [next]--> red
#
# Status: deadlocked (3 steps, current: red)

# With guard variable bindings
sysml-lint simulate state-machine model.sysml -n Controller -b temperature=150

# Limit simulation steps
sysml-lint simulate state-machine model.sysml -n Loop -m 50

# JSON trace output
sysml-lint -f json simulate state-machine model.sysml -n TrafficLight -e next
```

#### State Machine Options

```
sysml-lint simulate state-machine [OPTIONS] <FILE>

Options:
  -n, --name <NAME>        State machine name (default: first found)
  -e, --events <EVENTS>    Events to inject (comma-separated)
  -m, --max-steps <N>      Maximum simulation steps [default: 100]
  -b, --bind <BINDINGS>    Variable bindings for guards (name=value)
```

#### State Machine Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Completed or still running |
| 1 | Deadlocked (no enabled transitions) |
| 2 | Max steps reached |

### Execute Action Flows

```sh
# Execute an action flow
sysml-lint simulate action-flow model.sysml -n ProcessOrder

# Output:
# Action: ProcessOrder
#
#   Step 0: [perform] perform validate
#   Step 1: [perform] perform checkInventory
#   Step 2: [perform] perform ship
#   Step 3: [perform] perform notifyCustomer
#
# Status: completed (4 steps)

# With variable bindings for conditionals
sysml-lint simulate action-flow model.sysml -n Workflow -b priority=high

# JSON output
sysml-lint -f json simulate action-flow model.sysml -n ProcessOrder
```

#### Action Flow Options

```
sysml-lint simulate action-flow [OPTIONS] <FILE>

Options:
  -n, --name <NAME>        Action name (default: first found)
  -m, --max-steps <N>      Maximum execution steps [default: 1000]
  -b, --bind <BINDINGS>    Variable bindings (name=value)
```

### Simulation Capabilities

The simulation engine supports:

**Constraint Evaluation:**
- Boolean expressions with comparison operators (`<`, `>`, `<=`, `>=`, `==`, `!=`)
- Logical operators (`and`, `or`, `xor`, `not`, `implies`)
- Arithmetic operators (`+`, `-`, `*`, `/`, `%`, `**`)
- Built-in functions: `abs`, `sqrt`, `floor`, `ceil`, `round`, `min`, `max`, `sum`
- Variable bindings from command line

**State Machines:**
- Entry state initialization
- Signal-based triggers (`accept signal`)
- Guard conditions with expression evaluation
- Entry/exit/do actions
- Effects on transitions
- Deadlock detection
- Step-by-step or run-to-completion modes

**Action Flows:**
- Sequential action execution
- Fork/join (parallel simulation)
- If/else conditionals
- While loops with guard expressions
- Assign actions (updates environment)
- Send actions
- Decide/merge nodes

## Checks

sysml-lint ships with 9 validation checks. Each can be individually disabled with `--disable <name>`.

| Check | Name | Severity | Description |
|-------|------|----------|-------------|
| Syntax | `syntax` | Error | Reports tree-sitter parse errors and missing syntax elements |
| Duplicates | `duplicates` | Error | Detects definitions of the same kind with identical names |
| Unused | `unused` | Note | Definitions that are never referenced anywhere in the file |
| Unresolved | `unresolved` | Warning | Type references and connection/allocation targets that don't resolve to any definition or usage |
| Unsatisfied | `unsatisfied` | Warning | Requirement definitions with no corresponding `satisfy` statement |
| Unverified | `unverified` | Warning | Requirement definitions with no corresponding `verify` statement |
| Port Types | `port-types` | Warning | Connected ports with incompatible types |
| Constraints | `constraints` | Warning | Constraint definitions with a body but no constraint expression |
| Calculations | `calculations` | Warning | Calculation definitions with a body but no return statement |

### Check Details

#### Syntax (`syntax`)

Reports nodes where tree-sitter encountered a parse error (`ERROR`) or expected a syntax element that was missing (`MISSING`). These indicate genuinely broken syntax, not style issues.

#### Duplicates (`duplicates`)

Flags when two definitions of the same kind share the same name within a file. For example, two `part def Vehicle` declarations. The diagnostic references the line number of the first definition.

#### Unused (`unused`)

Finds definitions that are never referenced by any usage, specialization, connection, flow, satisfaction, verification, or allocation in the file. Package definitions are excluded since they are structural containers rather than referenceable types.

#### Unresolved (`unresolved`)

Checks that type references (`: SomeType`), connection endpoints, and allocation targets resolve to a known name. Known names include all definitions and usages in the file, plus the SysML v2 standard library types (ScalarValues, ISQ, SI units, base types). Qualified stdlib references like `ISQ::MassValue` or `SI::kg` are recognized automatically.

#### Unsatisfied (`unsatisfied`)

Checks that every `requirement def` in the file has at least one `satisfy` statement referencing it. Missing satisfaction traces are a common MBSE gap.

#### Unverified (`unverified`)

Checks that every `requirement def` in the file has at least one `verify` statement referencing it. Verification traceability is critical for systems engineering V&V.

#### Port Types (`port-types`)

When two ports are connected, checks that their declared types are compatible. Types are compatible if they are identical, or if one is the conjugate of the other (prefixed with `~`). For example, `FuelPort` and `~FuelPort` are compatible, but `FuelPort` and `ElectricalPort` are not.

#### Constraints (`constraints`)

Checks that `constraint def` declarations with a body block actually contain a constraint expression. A constraint definition like `constraint def C { in x : Real; }` that declares parameters but has no expression (`x > 0;`) is likely incomplete. Forward-declared constraints (`constraint def C;`) are not flagged.

#### Calculations (`calculations`)

Checks that `calc def` declarations with a body block contain a `return` statement. A calculation definition like `calc def F { in x : Real; }` with parameters but no `return` is likely incomplete. Forward-declared calcs (`calc def F;`) are not flagged.

## Diagnostic Codes

### Errors

| Code | Check | Message |
|------|-------|---------|
| E001 | syntax | `Syntax error: near <context>` or `Missing expected syntax element: near <context>` |
| E002 | duplicates | `duplicate <kind> '<name>' (first defined at line <n>)` |

### Warnings

| Code | Check | Message |
|------|-------|---------|
| W001 | unused | `<kind> '<name>' is defined but never referenced` |
| W002 | unsatisfied | `requirement def '<name>' has no corresponding satisfy statement` |
| W003 | unverified | `requirement def '<name>' has no corresponding verify statement` |
| W004 | unresolved | `type '<name>' is not defined in this file` |
| W005 | unresolved | `reference '<name>' does not resolve to any definition or usage` |
| W006 | port-types | `connected ports have different types: '<a>' is '<typeA>' but '<b>' is '<typeB>'` |
| W007 | constraints | `constraint def '<name>' has a body but no constraint expression` |
| W008 | calculations | `calc def '<name>' has a body but no return statement` |

## Output Formats

### Text (default)

Standard compiler-style diagnostics:

```
model.sysml:12:5: warning[W002]: requirement def `MassReq` has no corresponding satisfy statement
model.sysml:20:5: note[W001]: part def `Engine` is defined but never referenced
```

Format: `file:line:col: severity[code]: message`

### JSON

Structured output for editor integration and CI tooling:

```json
[
  {
    "file": "model.sysml",
    "span": {
      "start_row": 12,
      "start_col": 5,
      "end_row": 15,
      "end_col": 6,
      "start_byte": 234,
      "end_byte": 310
    },
    "severity": "warning",
    "code": "W002",
    "message": "requirement def `MassReq` has no corresponding satisfy statement"
  }
]
```

## CI Integration

### GitHub Actions

```yaml
name: SysML Lint
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install sysml-lint
        run: |
          git clone https://github.com/jackhale98/tree-sitter-sysml.git
          git clone https://github.com/jackhale98/sysml-lint.git
          cd sysml-lint
          cargo build --release
          echo "$PWD/target/release" >> $GITHUB_PATH

      - name: Lint SysML models
        run: sysml-lint lint --severity warning models/**/*.sysml
```

### GitLab CI

```yaml
sysml-lint:
  stage: test
  script:
    - sysml-lint lint --format json --severity warning models/*.sysml > lint-results.json
  artifacts:
    reports:
      codequality: lint-results.json
```

### Pre-commit Hook

```sh
#!/bin/sh
# .git/hooks/pre-commit
sysml_files=$(git diff --cached --name-only --diff-filter=ACM | grep '\.sysml$')
if [ -n "$sysml_files" ]; then
    sysml-lint lint --severity error $sysml_files
fi
```

## Editor Integration

### Emacs (sysml2-mode)

sysml-lint integrates with [sysml2-mode](https://github.com/jackhale98/sysml2-mode) for both Flymake diagnostics and interactive simulation. With `sysml-lint` on your `$PATH`:

- **Flymake**: Diagnostics appear inline as you edit
- **Simulation**: Run `M-x sysml2-simulate` to simulate constraints, state machines, and action flows interactively

### Generic (JSON pipe)

Any editor that can run an external command and parse JSON can integrate with sysml-lint:

```sh
sysml-lint lint --format json --quiet model.sysml
```

The JSON output includes byte offsets (`start_byte`, `end_byte`) for precise highlighting and line/column positions for gutter markers.

## Building from Source

### Prerequisites

- Rust 1.70+ (stable)
- C compiler (gcc or clang) for tree-sitter grammar compilation
- [tree-sitter-sysml](https://github.com/jackhale98/tree-sitter-sysml) grammar source

### Build

```sh
# Clone with grammar as sibling
git clone https://github.com/jackhale98/tree-sitter-sysml.git
git clone https://github.com/jackhale98/sysml-lint.git
cd sysml-lint

# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

### Vendoring the Grammar

To make the project self-contained, copy the grammar source into the project:

```sh
mkdir -p tree-sitter-sysml/src
cp ../tree-sitter-sysml/src/parser.c tree-sitter-sysml/src/
cp ../tree-sitter-sysml/src/tree_sitter/ tree-sitter-sysml/src/ -r
```

The build script checks `./tree-sitter-sysml/src/` first, then falls back to `../tree-sitter-sysml/src/`.

## Architecture

```
src/
  main.rs          CLI entry point (clap subcommands)
  lib.rs           Public module exports
  parser.rs        Tree-sitter FFI + parse tree -> Model extraction
  model.rs         Model types: definitions, usages, connections, flows, etc.
  diagnostic.rs    Diagnostic/Severity types and error codes
  output.rs        Text and JSON formatters
  checks/
    mod.rs         Check trait + registry
    syntax.rs      E001: parse errors
    duplicates.rs  E002: duplicate definitions
    references.rs  W001/W004/W005: unused defs, unresolved types
    requirements.rs W002/W003: unsatisfied/unverified requirements
    ports.rs       W006: port type mismatches
    constraints.rs W007: empty constraint bodies
    calculations.rs W008: calc missing return
  sim/
    mod.rs         Simulation engine modules
    expr.rs        Expression AST and runtime values (Value, Expr, Env)
    expr_parser.rs Tree-sitter -> Expr AST extraction
    eval.rs        Expression evaluator with built-in functions
    constraint_eval.rs  Constraint/calc model extraction and evaluation
    state_machine.rs    State machine model types
    state_parser.rs     Tree-sitter -> state machine extraction
    state_sim.rs        State machine simulation engine
    action_flow.rs      Action flow model types
    action_parser.rs    Tree-sitter -> action flow extraction
    action_exec.rs      Action flow execution engine
tests/
  integration.rs   Integration tests (17 tests)
test/
  fixtures/        SysML v2 example files for testing
```

### Parser

The parser uses tree-sitter-sysml via FFI (`extern "C"`) to parse SysML v2 textual notation into a concrete syntax tree. A single-pass recursive walk extracts structural model elements:

- **Definitions** (27 SysML/KerML kinds): `part def`, `port def`, `action def`, etc.
- **Usages**: `part`, `port`, `action`, `state`, `binding`, `succession`, etc.
- **Connections**: From `connect_clause` in connection usages
- **Flows**: `flow of Type from A to B`
- **Satisfactions**: `satisfy Requirement` with optional `by`
- **Verifications**: `require` statements inside verification definitions
- **Allocations**: `allocate A to B`

### Model

The extracted model provides two key queries used by checks:

- `defined_names()` -- All definition names in the file
- `referenced_names()` -- All names referenced via typing, specialization, connections, flows, satisfactions, verifications, allocations, and explicit type references

### Standard Library

sysml-lint recognizes 49 built-in SysML v2 standard library types so they are not flagged as unresolved. These cover:

- **ScalarValues**: `Boolean`, `String`, `Integer`, `Real`, `Natural`, `Complex`, `Rational`
- **ISQ quantities**: `MassValue`, `LengthValue`, `TimeValue`, `PowerValue`, `ForceValue`, and 15 others
- **SI units**: `kg`, `m`, `s`, `A`, `K`, `mol`, `cd`, `N`, `Pa`, `J`, `W`, `Hz`
- **Base types**: `Anything`, `Nothing`, `Object`, `Part`, `Port`, `Action`, `State`, etc.
- **Libraries**: `TradeStudy`, `VerdictKind`, `SampledFunction`, etc.

Qualified standard library references (e.g., `ISQ::MassValue`, `SI::kg`, `ScalarValues::Real`) are also recognized via namespace prefix matching.

### Simulation Engine

The simulation engine re-parses source files using tree-sitter and extracts behavioral constructs into typed models:

- **Expressions**: Full AST with arithmetic, comparison, logical, and function call nodes
- **Constraints**: Boolean expression evaluation with variable bindings
- **Calculations**: Expression evaluation with parameterized inputs
- **State Machines**: Event-driven simulation with triggers, guards, effects, and entry/exit actions
- **Action Flows**: Sequential execution, fork/join parallelism, conditionals, loops, and assignments

## License

GPL-3.0-or-later

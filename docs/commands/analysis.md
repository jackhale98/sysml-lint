# Analysis Commands

Commands for validating, inspecting, and querying SysML v2 models.

## lint

Validate SysML v2 files against structural rules.

```sh
sysml lint model.sysml
sysml lint src/*.sysml                       # Multiple files
sysml lint model.sysml -I lib/               # Include imports
sysml lint -f json model.sysml               # JSON output
sysml lint --severity warning model.sysml    # Only warnings+
sysml lint --disable unused,unresolved model.sysml
```

| Option | Description |
|--------|-------------|
| `-d, --disable <CHECKS>` | Disable checks (comma-separated). See [Validation](../validation.md). |
| `-s, --severity <LEVEL>` | Minimum severity: `note`, `warning`, `error` (default: `note`) |

Exit codes: `0` = no errors, `1` = errors found.

## list

List model elements with optional filters. Alias: `ls`.

```sh
sysml list model.sysml
sysml list --kind parts model.sysml          # Only part definitions
sysml list --kind port model.sysml           # Only port usages
sysml list --name Vehicle model.sysml        # Name search
sysml list --parent Vehicle model.sysml      # Children of Vehicle
sysml list --unused model.sysml              # Unreferenced defs
sysml list -f json model.sysml               # JSON output
```

| Option | Description |
|--------|-------------|
| `-k, --kind <KIND>` | Filter: `parts`, `ports`, `actions`, `states`, `requirements`, `constraints`, `all`, `definitions`, `usages` |
| `-n, --name <PATTERN>` | Substring name filter |
| `-p, --parent <NAME>` | Filter by parent definition |
| `--unused` | Show only unreferenced definitions |
| `--abstract` | Show only abstract definitions |
| `--visibility <VIS>` | Filter by `public`, `private`, `protected` |
| `--view <NAME>` | Apply a SysML v2 view definition as a filter preset |

## show

Show detailed information about a specific element.

```sh
sysml show model.sysml Vehicle
sysml show -f json model.sysml Engine
```

Displays: kind, visibility, parent, documentation, type, children, relationships.

## check

Validate SysML models and project integrity. Superset of `lint` — runs all lint checks plus record and project validation.

```sh
sysml check model.sysml
sysml check --lint-only model.sysml          # Same as lint
sysml check --severity warning model.sysml
```

| Option | Description |
|--------|-------------|
| `-d, --disable <CHECKS>` | Disable specific checks (comma-separated) |
| `-s, --severity <LEVEL>` | Minimum severity: `note`, `warning`, `error` (default: `note`) |
| `--lint-only` | Run only lint checks (no record/project checks) |

## trace

Generate a requirements traceability matrix.

```sh
sysml trace model.sysml
sysml trace --check --min-coverage 80 model.sysml    # CI gate
sysml trace -f json model.sysml
```

| Option | Description |
|--------|-------------|
| `--check` | Exit with error if requirements lack satisfaction/verification |
| `--min-coverage <PCT>` | Minimum coverage percentage (with `--check`) |

## interfaces

Analyze port interfaces and identify unconnected ports.

```sh
sysml interfaces model.sysml
sysml interfaces --unconnected model.sysml
```

| Option | Description |
|--------|-------------|
| `--unconnected` | Show only unconnected ports (interface gaps) |

## deps

Analyze dependencies for a specific element — what it depends on and what references it.

```sh
sysml deps model.sysml Vehicle
sysml deps model.sysml Engine --reverse       # Only show "referenced by"
sysml deps model.sysml Engine --forward       # Only show "depends on"
sysml deps -f json model.sysml Vehicle
```

| Option | Description |
|--------|-------------|
| `--reverse` | Show only reverse dependencies (what references this element) |
| `--forward` | Show only forward dependencies (what this element depends on) |

## diff

Compare two SysML files and report semantic differences (added/removed/changed definitions, usages, connections).

```sh
sysml diff old.sysml new.sysml
sysml diff -f json v1.sysml v2.sysml
```

Unlike text-based diff, this compares at the model level — detecting renamed types, changed members, and structural modifications regardless of formatting changes.

## allocation

Display the logical-to-physical allocation matrix. In SysML v2, allocations map actions and use-cases to parts.

```sh
sysml allocation model.sysml
sysml allocation --unallocated model.sysml    # Only show gaps
sysml allocation --check model.sysml          # CI: exit 1 if gaps exist
sysml allocation -f json model.sysml
```

| Option | Description |
|--------|-------------|
| `--check` | Exit with error if unallocated elements exist |
| `--unallocated` | Show only unallocated elements |

## coverage

Generate a model quality report: documentation coverage, typed usages, populated definitions, requirement satisfaction/verification, and an overall score.

```sh
sysml coverage model.sysml
sysml coverage --check --min-score 80 model.sysml    # CI gate
sysml coverage -f json model.sysml
```

| Option | Description |
|--------|-------------|
| `--check` | Exit with error if score is below minimum |
| `--min-score <PCT>` | Minimum overall score percentage (default: 0, used with `--check`) |

**Reported metrics:**

| Metric | Description |
|--------|-------------|
| Documentation | Percentage of definitions with doc comments |
| Typed usages | Percentage of usages with explicit type references |
| Populated defs | Percentage of definitions with at least one member |
| Req satisfaction | Percentage of requirements with a satisfy statement |
| Req verification | Percentage of requirements with a verify statement |
| Overall score | Weighted average of all metrics |

## stats

Show aggregate model statistics: element counts by kind, relationship counts, documentation coverage, and nesting depth.

```sh
sysml stats model.sysml
sysml stats -f json model.sysml              # JSON output
sysml stats src/*.sysml                       # Multiple files
```

Output includes definitions/usages by kind, connection/flow/satisfaction/verification/allocation counts, package count, abstract definitions, import count, max nesting depth, and documentation coverage percentage.

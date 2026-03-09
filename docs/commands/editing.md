# Editing Commands

Commands for generating, modifying, and formatting SysML v2 source files.

## new

Generate a SysML v2 definition template to stdout. Use this as a starting point or pipe into a file.

The `new` command creates template text — it does not modify existing files. To add elements to existing files, use [`edit add`](#edit).

```sh
sysml new part-def Vehicle
sysml new part-def Vehicle --extends Base --doc "A vehicle"
sysml new part-def Vehicle -m "part engine:Engine" -m "part wheels:Wheel"
sysml new port-def FuelPort -m "in item fuel:FuelType"
sysml new view-def PartsView --expose "Vehicle::*" --filter part
sysml new constraint-def SpeedLimit -m "in speed:Real"
sysml new package VehiclePkg
```

**Available kinds:** `part-def`, `port-def`, `action-def`, `state-def`, `constraint-def`, `calc-def`, `requirement` (`req`), `enum-def`, `attribute-def` (`attr`), `item-def`, `view-def`, `viewpoint-def`, `package` (`pkg`), `use-case`, `connection-def`, `interface-def`, `flow-def`, `allocation-def`

| Option | Description |
|--------|-------------|
| `--extends <TYPE>` | Specialization supertype (`:>` syntax) |
| `--abstract` | Mark as abstract |
| `--short-name <ALIAS>` | Short name (`<alias>` before the name) |
| `--doc <TEXT>` | Documentation comment (`doc /* text */`) |
| `-m, --member <SPEC>` | Add member (repeatable): `"[dir] kind name[:type]"` |
| `--expose <PATTERN>` | (view-def) Expose clause: `"Vehicle::*"` |
| `--filter <KIND>` | (view-def) Filter by element kind |

## edit

Surgically modify SysML v2 files using CST-aware byte-accurate positions.

### edit add

Add a definition or usage to an existing file. For usage-level elements (`part`, `port`, etc.), automatically inserts inside an existing definition body.

```sh
sysml edit add model.sysml part engine -t Engine
sysml edit add model.sysml port fuelIn -t FuelPort --inside Vehicle
sysml edit add model.sysml part-def Wheel --dry-run
sysml edit add model.sysml attribute mass -t Real --inside Vehicle
```

| Option | Description |
|--------|-------------|
| `-t, --type-ref <TYPE>` | Type reference (`: Type` for usages, `:>` for defs) |
| `--inside <NAME>` | Insert inside this definition (auto-detected for usages) |
| `--doc <TEXT>` | Documentation comment |
| `--extends <TYPE>` | Specialization supertype (definition kinds) |
| `--abstract` | Mark as abstract (definition kinds) |
| `--short-name <ALIAS>` | Short name alias |
| `-m, --member <SPEC>` | Add members (definition kinds) |
| `--dry-run` | Preview as unified diff |

### edit remove

Remove an element by name.

```sh
sysml edit remove model.sysml Engine --dry-run
sysml edit remove model.sysml Engine
```

### edit rename

Rename an element and update all references.

```sh
sysml edit rename model.sysml Engine Motor --dry-run
sysml edit rename model.sysml Engine Motor
```

## fmt

Format SysML v2 files. CST-aware indentation that handles nested definitions, comments, and state machines correctly.

```sh
sysml fmt model.sysml
sysml fmt --check model.sysml         # CI: exit 1 if unformatted
sysml fmt --diff model.sysml          # Show diff without writing
sysml fmt --indent-width 2 model.sysml
```

| Option | Description |
|--------|-------------|
| `--check` | Check formatting without modifying (exit 1 if unformatted) |
| `--diff` | Print diff instead of writing files |
| `--indent-width <N>` | Indentation width (default: 4) |

## scaffold

Generate SysML v2 elements with teaching comments and complete example projects.

### scaffold element

Generate a single element with inline teaching comments explaining every SysML v2 construct used.

```sh
sysml scaffold element part-def Vehicle
sysml scaffold element part-def Vehicle --extends Base --doc "A vehicle"
sysml scaffold element requirement SafetyReq --no-comments
```

| Option | Description |
|--------|-------------|
| `--extends <TYPE>` | Specialization supertype |
| `--doc <TEXT>` | Documentation comment |
| `--no-comments` | Disable teaching comments |

### scaffold example

Generate a complete example project with multiple files demonstrating domain workflows.

```sh
sysml scaffold example brake-system
sysml scaffold example sensor-module -o ./examples/
sysml scaffold list-examples                    # Show available examples
sysml scaffold list-kinds                       # Show available element kinds
```

| Option | Description |
|--------|-------------|
| `-o, --output <PATH>` | Output directory (default: current directory) |

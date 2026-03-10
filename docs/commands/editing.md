# Editing Commands

Commands for generating, modifying, and formatting SysML v2 source files.

## add

Add an element to a SysML model — interactively, to a file, or to stdout.

With no arguments, launches a guided wizard using domain vocabulary. With a file, kind, and name, inserts directly. With `--stdout`, prints to terminal without modifying files.

```sh
sysml add                                                  # interactive wizard
sysml add model.sysml part-def Vehicle                     # insert into file
sysml add model.sysml part engine -t Engine --inside Vehicle  # usage inside def
sysml add --stdout part-def Vehicle                        # print to stdout
sysml add --stdout --teach part-def Vehicle                # with teaching comments
sysml add --stdout part-def Vehicle --extends Base --doc "A vehicle"
sysml add --stdout part-def Vehicle -m "part engine:Engine" -m "part wheels:Wheel"
sysml add --stdout port-def FuelPort -m "in item fuel:FuelType"
sysml add --stdout view-def PartsView --expose "Vehicle::*" --filter part
```

**Available kinds:** `part-def`, `port-def`, `action-def`, `state-def`, `constraint-def`, `calc-def`, `requirement` (`req`), `enum-def`, `attribute-def` (`attr`), `item-def`, `view-def`, `viewpoint-def`, `package` (`pkg`), `use-case`, `connection-def`, `interface-def`, `flow-def`, `allocation-def`

Usage-level kinds (no `-def` suffix) generate `kind name [: type];` usages suitable for insertion inside a definition.

| Option | Description |
|--------|-------------|
| `-t, --type-ref <TYPE>` | Type reference (`: Type` for usages, `:> Type` for defs with `--extends`) |
| `--inside <NAME>` | Insert inside this definition (auto-detected for usages) |
| `--extends <TYPE>` | Specialization supertype (`:>` syntax) |
| `--abstract` | Mark as abstract |
| `--short-name <ALIAS>` | Short name (`<alias>` before the name) |
| `--doc <TEXT>` | Documentation comment (`doc /* text */`) |
| `-m, --member <SPEC>` | Add member (repeatable): `"[dir] kind name[:type]"` |
| `--expose <PATTERN>` | (view-def) Expose clause: `"Vehicle::*"` |
| `--filter <KIND>` | (view-def) Filter by element kind |
| `--stdout` | Print to stdout without modifying files |
| `--teach` | Include teaching comments explaining every SysML v2 construct used |
| `--dry-run` | Preview changes as a unified diff without writing |
| `-i, --interactive` | Launch interactive wizard even when other args are provided |

### Dispatch modes

| file | kind | name | `--stdout` | Behavior |
|------|------|------|------------|----------|
| None | None | None | false | Full interactive wizard |
| None | Some | Some | any | Stdout (infer `--stdout`) |
| Some | None | None | false | Guided file mode: parse file, wizard picks kind/name/parent |
| Some | Some | Some | false | Direct insert into file |

## remove

Remove a named element from a SysML file.

```sh
sysml remove model.sysml Engine --dry-run    # preview
sysml remove model.sysml Engine              # apply
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Preview changes as a unified diff without writing |

## rename

Rename an element and update all whole-word references in the file.

```sh
sysml rename model.sysml Engine Motor --dry-run    # preview
sysml rename model.sysml Engine Motor              # apply
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Preview changes as a unified diff without writing |

## example

Generate complete example projects with multiple files demonstrating domain workflows.

```sh
sysml example brake-system                   # generate in current directory
sysml example sensor-module -o ./examples/   # custom output directory
sysml example --list                         # show available examples
```

| Option | Description |
|--------|-------------|
| `-o, --output <PATH>` | Output directory (default: current directory) |
| `--list` | List available example projects |

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

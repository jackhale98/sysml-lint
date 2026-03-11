# CI & Editor Integration

## GitHub Actions

```yaml
name: SysML Lint
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install sysml
        run: |
          git clone --recurse-submodules https://github.com/jackhale98/sysml-cli.git /tmp/sysml-cli
          cd /tmp/sysml-cli
          cargo build --release
          echo "/tmp/sysml-cli/target/release" >> $GITHUB_PATH

      - name: Initialize project
        run: sysml init --force

      - name: Lint models
        run: sysml lint --severity warning models/**/*.sysml

      - name: Check formatting
        run: sysml fmt --check models/**/*.sysml

      - name: Check requirement coverage
        run: sysml trace --check --min-coverage 80 models/**/*.sysml

      - name: Check model quality
        run: sysml coverage --check --min-score 70 models/**/*.sysml

      - name: Check allocations
        run: sysml allocation --check models/**/*.sysml
```

### CI gate commands

| Command | Purpose | Exit code |
|---------|---------|-----------|
| `sysml lint --severity error` | Block on syntax/duplicate errors | 1 if errors |
| `sysml fmt --check` | Enforce formatting | 1 if unformatted |
| `sysml trace --check --min-coverage 80` | Require requirement coverage | 1 if below threshold |
| `sysml coverage --check --min-score 70` | Require model quality score | 1 if below threshold |
| `sysml allocation --check` | Require all allocations | 1 if gaps exist |

## Editor Integration

### Emacs (sysml2-mode)

`sysml` integrates with [sysml2-mode](https://github.com/jackhale98/sysml2-mode) for Flymake diagnostics, interactive simulation, and FMI export. With `sysml` on your `$PATH`:

- **Flymake**: Diagnostics appear inline as you edit
- **Simulation**: `M-x sysml2-simulate` for constraints, state machines, action flows
- **FMI Export**: `M-x sysml2-fmi-extract-interfaces` extracts interfaces
- **Diagrams**: `M-x sysml2-diagram` generates diagrams inline

### JSON output for other editors

All commands support `-f json` for structured output suitable for editor integration:

```sh
sysml lint -f json model.sysml          # Diagnostics as JSON array
sysml list -f json model.sysml          # Element list as JSON
sysml simulate list -f json model.sysml # Simulatable items as JSON
```

This works with any editor that can parse JSON from a subprocess — VS Code extensions, Neovim plugins, etc.

## Library resolution in CI

If your project has `.sysml/config.toml` with `library_paths` configured (set automatically by `sysml init` when `libraries/` exists), all commands resolve imports automatically — no `-I` flag needed in CI steps.

For projects without a config, run `sysml init --force` as a CI step (see GitHub Actions example above) or pass `-I` explicitly.

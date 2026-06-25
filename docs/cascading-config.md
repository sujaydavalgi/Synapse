# Cascading Configuration

Spanda configuration merges in a fixed order so operators can share a base profile and override only what changes per environment, deployment, or robot.

## Layer stack

```
base
  ↓
environment
  ↓
deployment
  ↓
robot-specific
```

Declare layers in the root `spanda.toml`:

```toml
[extends]
base = "configs/base.toml"
environment = "configs/warehouse-a.toml"
deployment = "configs/production.toml"
robot = "configs/rover-001.toml"
```

Each layer file may itself contain an `[extends]` table for nested inheritance. The resolver detects circular references and reports them as errors.

## Merge semantics

| Strategy | Behavior |
|----------|----------|
| `replace` | Later scalar or array replaces earlier value (default) |
| `append` | Arrays are concatenated |
| `merge_by_id` | Array-of-tables merged by `id` field |

Set per-key hints in the root manifest:

```toml
[merge]
fleet = "merge_by_id"
capabilities = "append"
```

## Conflict detection

When two layers set incompatible values for the same key, the later layer wins. Use `spanda config diff` to inspect changes between any two files:

```bash
spanda config diff configs/base.toml configs/production.toml
```

Validation (`spanda config validate`) reports structural conflicts such as duplicate ports, buses, or serial numbers in the **resolved** tree.

## Inspecting resolution

```bash
spanda config resolve          # full merged TOML as JSON
spanda config graph            # dependency graph and merge order
spanda config report           # human-readable multi-section report
```

## JSON layers

Machine-generated layers may use `.json` files. The resolver converts JSON to the same internal representation as TOML before merging.

## Example

See `crates/spanda-config/tests/fixtures/warehouse/` for a working layered project with base and environment overrides.

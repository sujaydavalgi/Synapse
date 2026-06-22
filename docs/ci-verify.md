# CI integration for `spanda verify`

Robotics teams adopt Spanda when **hardware fit is checked in CI** before hardware exists on the bench. This guide shows how to run `spanda check` and `spanda verify` in GitHub Actions and GitLab CI, parse `--json` output, and fail the pipeline on incompatible deploy targets.

## What to gate in CI

| Command | When to run | Fail condition |
|---------|-------------|----------------|
| `spanda check` | Every push and PR | Non-zero exit (type/safety errors) |
| `spanda verify --json` | Every push and PR (after `check` passes) | `compatible: false` or non-zero exit |
| `spanda verify --json --all-targets` | Nightly or release branches | Any matrix cell incompatible for a required target |

`spanda verify` answers: *Will this program run on the declared deploy target?* — memory, sensors, actuators, task timing, battery estimates, and AI model requirements.

## JSON output shape

```bash
spanda verify examples/showcase/hardware_compatibility.sd --json --target RoverV1
```

Successful compatible deploy:

```json
{
  "ok": true,
  "compatible": true,
  "target": "RoverV1",
  "items": [
    { "category": "memory", "message": "...", "severity": "info", "line": null, "column": null }
  ]
}
```

Incompatible deploy (non-zero exit):

```json
{
  "ok": false,
  "compatible": false,
  "target": "ESP32",
  "items": [
    { "category": "memory", "message": "...", "severity": "error", "line": 12, "column": 3 }
  ]
}
```

**CI rule:** treat `compatible == false` or `ok == false` as a build failure. Do not rely on parsing human-readable stdout.

With `--all-targets`, the response includes a `matrix` object with `cells[]` — each cell has `robot`, `target`, and `compatible`.

## Install Spanda in CI

**Option A — build from source (recommended for contributors):**

```yaml
- uses: dtolnay/rust-toolchain@stable
- run: cargo build -p spanda-cli --release
- run: echo "$PWD/target/release" >> $GITHUB_PATH
```

**Option B — prebuilt release (evaluators):**

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.sh | sh
```

See [installation.md](./installation.md) for platform packages.

## GitHub Actions example

```yaml
name: Spanda verify

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build -p spanda-cli --release
      - run: echo "$PWD/target/release" >> $GITHUB_PATH

      - name: Type check
        run: spanda check examples/showcase/killer_demo.sd

      - name: Verify deploy target (JSON)
        run: |
          spanda verify examples/showcase/hardware_compatibility.sd \
            --json --target RoverV1 > verify.json
          python3 - <<'PY'
          import json, sys
          data = json.load(open("verify.json"))
          if not data.get("compatible"):
              print("Deploy incompatible:", data, file=sys.stderr)
              sys.exit(1)
          print("Deploy compatible with", data.get("target"))
          PY

      - name: Upload verify report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: spanda-verify-report
          path: verify.json
```

### Matrix over multiple targets

```yaml
      - name: Verify all declared targets
        run: |
          spanda verify src/main.sd --json --all-targets > matrix.json
          python3 - <<'PY'
          import json, sys
          data = json.load(open("matrix.json"))
          failures = [
              c for c in data.get("matrix", {}).get("cells", [])
              if not c.get("compatible")
          ]
          if failures:
              for c in failures:
                  print(f"INCOMPATIBLE: {c['robot']} -> {c['target']}", file=sys.stderr)
              sys.exit(1)
          PY
```

## GitLab CI example

```yaml
stages:
  - verify

spanda-verify:
  stage: verify
  image: rust:bookworm
  script:
    - cargo build -p spanda-cli --release
    - export PATH="$PWD/target/release:$PATH"
    - spanda check examples/showcase/killer_demo.sd
    - |
      spanda verify examples/showcase/hardware_compatibility.sd \
        --json --target RoverV1 > verify.json
      python3 -c "
      import json, sys
      d = json.load(open('verify.json'))
      sys.exit(0 if d.get('compatible') else 1)
      "
  artifacts:
    when: always
    paths:
      - verify.json
    expire_in: 1 week
```

## Recommended workflow

```bash
# Local (same order as CI)
spanda check my_robot.sd
spanda verify my_robot.sd --json --target RoverV1
```

Week 1: add `check` + `verify` to CI.  
Week 2: pin `--target` to your production hardware profile.  
Week 3: add `--simulate` on release branches for fault-injection warnings.

## Flagship examples for smoke tests

| Pillar | File | Command |
|--------|------|---------|
| Safety | `examples/showcase/ai_safety_violation.sd` | `spanda check` (expect failure) |
| Verify | `examples/showcase/hardware_compatibility.sd` | `spanda verify --json` |
| Sim | `examples/showcase/killer_demo.sd` | `spanda sim` |

Full walkthrough: [killer-demo.md](./killer-demo.md). Adoption path: [adoption-path.md](./adoption-path.md).

Golden path: `./scripts/ci_verify_golden_path.sh` (CI job `ci-verify-golden-path`).

## Related

- [hardware-compatibility.md](./hardware-compatibility.md) — language constructs and report categories
- [man/spanda-verify.md](./man/spanda-verify.md) — CLI reference
- [api-contract.json](./api-contract.json) — JSON schema for verify output

# Spanda v0.1.0-alpha is live

Spanda v0.1.0-alpha is now available for public evaluation.

Release: <https://github.com/Davalgi/Spanda/releases/tag/v0.1.0-alpha>

## Install

Prebuilt packages for **Linux**, **macOS**, and **Windows** are published on [GitHub Releases](https://github.com/Davalgi/Spanda/releases). Semver tags (for example `v0.1.0`) ship platform archives, shell/PowerShell installers, a Windows `.msi`, and a Homebrew formula.

```bash
# Linux / macOS — replace the tag with the release you are installing
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.sh | sh
```

Windows: download the `.msi` or run the PowerShell installer from the same release page.

Full instructions: [installation.md](./installation.md)

To build from source instead:

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
npm install
npm run build:rust
export PATH="$PWD/target/release:$PATH"
```

## What to try first

Start with the showcase examples:

- `examples/showcase/rover_navigation.sd` — sensors + AI planning + SafeAction
- `examples/showcase/warehouse_robot.sd` — tasks + communication + safety zones
- `examples/showcase/ai_safety_violation.sd` — unsafe AI proposal rejection
- `examples/showcase/hardware_compatibility.sd` — deploy target verification
- `examples/showcase/communication_demo.sd` — message/topic/service/action
- `examples/showcase/digital_twin_demo.sd` — twin telemetry + replay

## Quick commands

```bash
spanda check examples/showcase/rover_navigation.sd
spanda run examples/showcase/rover_navigation.sd
spanda verify examples/showcase/hardware_compatibility.sd --json
spanda check examples/showcase/ai_safety_violation.sd
```

## Scope of this alpha

v0.1.0-alpha focuses on:

- Stability
- Examples
- Documentation
- CI/CD
- Developer experience

This is a public evaluation release, not a production robotics release.

## Feedback requested

Please share:

- parser/typechecker/runtime bugs
- safety and verification edge cases
- docs gaps in onboarding
- showcase examples that should be added next

Open issues: <https://github.com/Davalgi/Spanda/issues>

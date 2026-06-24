<p align="center">
  <img src="assets/image/low_res_logo.png" alt="Spanda — The Autonomous Systems Platform" width="360">
</p>

# Spanda

**The Autonomous Systems Platform** — *with a safety-first programming language at its core.*

*Build. Verify. Simulate. Deploy. Operate.*

Spanda is an autonomous systems platform centered on the **Spanda Language** (`.sd` files): typed robot programs, safety gates, hardware verification, simulation, replay, fleet operations, mission assurance, and **37** official packages.

Repository: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda)

---

## Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

**Spanda** is a Sanskrit term meaning *the divine pulse* — the creative vibration of consciousness and energy that manifests as expansion and contraction, the first stir of awareness that creates and sustains the universe.

---

## What is Spanda?

Spanda is an **autonomous systems platform** built around the **Spanda Language** — a typed programming language where sensors, AI models, actuators, safety rules, and deployment targets are first-class concepts in source code.

You write a `robot` block with sensors, actuators, safety zones, and agents. The compiler enforces physical units, validates AI proposals before they reach hardware, and checks that your program fits the deployment target before you ship.

```spanda
robot SafePatrol {
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  ai_model planner: LLM { provider: "mock"; model: "patrol"; }

  safety {
    max_speed = 0.5 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  behavior patrol() {
    loop every 100ms {
      let proposal = planner.reason(prompt: "Plan motion", input: lidar.read());
      wheels.execute(safety.validate(proposal));
    }
  }
}
```

---

## Why Spanda?

Building autonomous systems today means stitching together Python scripts, C++ drivers, ROS2 nodes, safety monitors, and deployment checklists — with no single platform that treats **AI output as untrusted**, **hardware fit as compile-time**, and **safety as mandatory**.

**Traditional languages focus on:** algorithms, data structures, applications.

**Spanda focuses on:** autonomous systems, safety, hardware awareness, capability verification, simulation, and operational health.

Spanda is the coordination layer where perception, planning, safety validation, simulation, verification, and deployment live together — with the `.sd` language as the expressive core.

---

## Quick start

```bash
# Install (from clone)
git clone https://github.com/Davalgi/Spanda.git
cd Spanda && ./scripts/install.sh
# Or: cargo install --path crates/spanda-cli --locked

spanda demo rover          # flagship platform demo
spanda demo assurance      # mission assurance CLI suite
spanda demo self-healing   # recovery policies, heal/recover/sim

# Or step by step:
spanda check examples/showcase/killer_demo.sd      # type-check
spanda verify examples/showcase/hardware_compatibility.sd  # hardware fit
spanda sim examples/showcase/killer_demo.sd        # simulate
```

Install options: [docs/installation.md](docs/installation.md) · First project: [docs/getting-started.md](docs/getting-started.md)

Video script: [docs/demo-script.md](docs/demo-script.md) · Killer demo: [docs/killer-demo.md](docs/killer-demo.md)

---

## What you get

| | | |
|---|---|---|
| **Language (.sd)** | **Safety validation** | **Hardware verify** |
| **Simulation & replay** | **Mission assurance** | **Packages (37)** |
| **Health & readiness** | **Fleet & OTA** | **ROS2 / IoT / AI bridges** |

More samples: [docs/overview/code-samples.md](docs/overview/code-samples.md) · Demos: [docs/overview/demos-and-examples.md](docs/overview/demos-and-examples.md)

---

## Documentation

| Start here | Description |
|------------|-------------|
| **[docs/overview/](docs/overview/README.md)** | Expanded project home — platform, architecture, CLI, library |
| [docs/getting-started.md](docs/getting-started.md) | First robot in 10 minutes |
| [docs/platform-overview.md](docs/platform-overview.md) | Platform components and workflow |
| [docs/tutorials/README.md](docs/tutorials/README.md) | Tutorials and learning paths |
| [examples/README.md](examples/README.md) | Runnable examples library |
| [docs/README.md](docs/README.md) | Full documentation index |

| Topic | Guide |
|-------|--------|
| Feature status | [docs/feature-status.md](docs/feature-status.md) |
| Mission assurance | [docs/mission-assurance.md](docs/mission-assurance.md) |
| Architecture | [docs/overview/architecture.md](docs/overview/architecture.md) (pipeline) · [docs/architecture.md](docs/architecture.md) (full) |
| CLI reference | [docs/overview/cli.md](docs/overview/cli.md) · [docs/spanda-reference.md](docs/spanda-reference.md) |
| Packages & registry | [docs/overview/packages.md](docs/overview/packages.md) · [docs/packages.md](docs/packages.md) |
| Roadmap | [docs/roadmap.md](docs/roadmap.md) |

### Overview subpages

Full index: [docs/overview/README.md](docs/overview/README.md) — platform structure, components, differentiators, layers, library, packages, web playground, CLI, code samples, demos.

---

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

```bash
cargo test --workspace && npm test
```

---

## License

Apache-2.0 — see [LICENSE](LICENSE).

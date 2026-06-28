<p align="center">
  <img src="assets/image/banner.png" alt="Spanda — The Autonomous Systems Platform" width="640">
</p>

# Spanda

**The Autonomous Systems Platform** — *with a safety-first programming language at its core.*

*Build. Verify. Simulate. Deploy. Operate.*

Spanda is an autonomous systems platform centered on the **Spanda Language** (`.sd` files): typed robot programs, safety gates, hardware verification, cascading TOML configuration, simulation, replay, fleet operations, mission assurance, mission continuity, and **38** official packages.

**Spanda focuses on Readiness, Assurance, and Diagnosis for safety-critical autonomous systems.**

Repository: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda)

---

## Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

**Spanda** (*Pronounced **SPUN-duh** (/ˈspʌndə/)*) is a Sanskrit term meaning *the divine pulse* — the creative vibration of consciousness and energy that manifests as expansion and contraction in all entities, bridging stillness and movement within consciousness; and the first stir of awareness that creates and sustains the universe.

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

What Spanda is / isn't: [docs/overview/what-spanda-is.md](docs/overview/what-spanda-is.md) · Why Spanda (detail): [docs/overview/philosophy.md](docs/overview/philosophy.md)

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
spanda demo continuity     # takeover, delegation, succession

# Or step by step:
spanda check examples/showcase/killer_demo.sd      # type-check
spanda verify examples/showcase/hardware_compatibility.sd  # hardware fit
spanda sim examples/showcase/killer_demo.sd        # simulate
```

Install options: [docs/installation.md](docs/installation.md) · First project: [docs/getting-started.md](docs/getting-started.md)

---

## Explore further

| Topic | Guide |
|-------|--------|
| **5-minute eval & flagship demos** | [docs/overview/flagship-demos.md](docs/overview/flagship-demos.md) |
| **Where should I start?** (by role) | [docs/overview/where-to-start.md](docs/overview/where-to-start.md) |
| **Signature capabilities** | [docs/overview/signature-capabilities.md](docs/overview/signature-capabilities.md) |
| **Platform components** | [docs/overview/platform-components.md](docs/overview/platform-components.md) |
| **Feature status** | [docs/overview/feature-snapshot.md](docs/overview/feature-snapshot.md) · [docs/feature-status.md](docs/feature-status.md) |
| **Demos & examples** | [docs/overview/demos-and-examples.md](docs/overview/demos-and-examples.md) |
| **Code samples** | [docs/overview/code-samples.md](docs/overview/code-samples.md) |
| **Differentiators** | [docs/overview/differentiators.md](docs/overview/differentiators.md) |

**Full overview index:** [docs/overview/README.md](docs/overview/README.md)

---

## Documentation

| Start here | Description |
|------------|-------------|
| [docs/getting-started.md](docs/getting-started.md) | First robot in 10 minutes |
| [docs/platform-overview.md](docs/platform-overview.md) | Platform components and workflow |
| [docs/spanda-language.md](docs/spanda-language.md) | Language guide |
| [docs/tutorials/README.md](docs/tutorials/README.md) | Tutorials and learning paths |
| [examples/README.md](examples/README.md) | Runnable examples library |
| [docs/README.md](docs/README.md) | Full documentation index |

CLI reference: `spanda man <command>` · [docs/man/](docs/man/README.md) · Language API: [docs/spanda-reference.md](docs/spanda-reference.md)

---

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

```bash
cargo test --workspace && npm test
```

---

## License

Apache-2.0 — see [LICENSE](LICENSE).

<p align="center">
  <img src="assets/image/low_res_logo.png" alt="Spanda — The Autonomous Systems Platform" width="360">
</p>

# Spanda

**The Autonomous Systems Platform** — *with a safety-first programming language at its core.*

*Build. Verify. Simulate. Deploy. Operate.*

Spanda is an autonomous systems platform centered on the **Spanda Language** (`.sd` files): typed robot programs, safety gates, hardware verification, simulation, replay, fleet operations, mission assurance, and **37** official packages.

Repository: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda)

---

## Quick start

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda && ./scripts/install.sh

spanda demo rover          # flagship platform demo
spanda demo assurance      # mission assurance CLI suite
spanda check examples/showcase/killer_demo.sd
spanda verify examples/showcase/hardware_compatibility.sd --json
spanda sim examples/showcase/killer_demo.sd
```

Install options: [docs/installation.md](docs/installation.md) · First project: [docs/getting-started.md](docs/getting-started.md)

---

## What you get

| | | |
|---|---|---|
| **Language (.sd)** | **Safety validation** | **Hardware verify** |
| **Simulation & replay** | **Mission assurance** | **Packages (37)** |
| **Health & readiness** | **Fleet & OTA** | **ROS2 / IoT / AI bridges** |

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

More samples: [docs/overview/code-samples.md](docs/overview/code-samples.md)

---

## Documentation

| Start here | Description |
|------------|-------------|
| **[docs/overview/](docs/overview/README.md)** | **Expanded project home** (philosophy, differentiators, CLI, demos) |
| [docs/getting-started.md](docs/getting-started.md) | First robot in 10 minutes |
| [docs/platform-overview.md](docs/platform-overview.md) | Platform components and workflow |
| [docs/tutorials/README.md](docs/tutorials/README.md) | Tutorials and learning paths |
| [examples/README.md](examples/README.md) | Runnable examples library |
| [docs/README.md](docs/README.md) | Full documentation index |

| Topic | Guide |
|-------|--------|
| Feature status | [docs/feature-status.md](docs/feature-status.md) |
| Mission assurance | [docs/mission-assurance.md](docs/mission-assurance.md) |
| Architecture | [docs/architecture.md](docs/architecture.md) |
| CLI reference | [docs/overview/cli.md](docs/overview/cli.md) · [docs/spanda-reference.md](docs/spanda-reference.md) |
| Packages & registry | [docs/packages.md](docs/packages.md) · [docs/registry.md](docs/registry.md) |
| Roadmap | [docs/roadmap.md](docs/roadmap.md) |

---

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

```bash
cargo test --workspace && npm test
```

---

## License

Apache-2.0 — see [LICENSE](LICENSE).

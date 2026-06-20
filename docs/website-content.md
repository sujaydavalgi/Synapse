# Website Content

Draft copy for a future Spanda project website. Use as-is or adapt for static site generators.

---

## Homepage

### Hero

**Spanda — The Autonomous Systems Language**

Write robots, agents, and digital twins with built-in safety validation and hardware verification.

```bash
git clone https://github.com/sujaydavalgi/Spanda.git
npm run build:rust
spanda run examples/showcase/rover_navigation.sd
```

[Get Started](getting-started) · [View on GitHub](https://github.com/sujaydavalgi/Spanda)

### Tagline

The pulse of autonomous intelligence.

### Value proposition (3 columns)

**Safety-first AI**  
LLMs propose actions. Only `safety.validate()` unlocks actuators. Compile-time and runtime enforcement.

**Deploy with confidence**  
`spanda verify` checks sensors, memory, timing, and power against hardware profiles before you ship.

**Robot-native syntax**  
Sensors, actuators, topics, tasks, and safety zones are language keywords — not framework boilerplate.

### Code snippet (homepage)

```spanda
robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  ai_model planner: LLM { provider: "mock"; }

  safety { max_speed = 1.0 m/s; stop_if lidar.nearest_distance < 0.5 m; }

  agent Navigator {
    uses planner;
    plan {
      let action = safety.validate(planner.reason(prompt: "Navigate", input: lidar.read()));
      wheels.execute(action);
    }
  }
}
```

### Not replacing Python, C++, or Rust

Spanda orchestrates AI, robotics, safety, verification, and deployment. Keep your models in Python and drivers in C++.

---

## Features page

### AI-native agents

- `ai_model`, `agent`, `goal`, `memory`
- Mock providers for testing (no API keys required)
- `ActionProposal` → `SafeAction` safety gate

### Robotics primitives

- `robot`, `sensor`, `actuator`, `behavior`, `task every Nms`
- Physical units: `m`, `s`, `rad`, `m/s`
- Trajectories, poses, transforms

### Safety validation

- Safety zones (circle, rectangle)
- `max_speed`, `stop_if`, emergency stop
- Compile-time rejection of unsafe AI patterns

### Hardware verification

- `hardware` profiles and `deploy` targets
- `spanda verify` with matrix and fault injection
- Built-in profiles: RoverV1, JetsonOrin, RaspberryPi5, ESP32

### Communication

- ROS2-style `message`, `topic`, `service`, `action`
- In-memory simulator transport
- Optional ROS2 adapter (experimental)

### Simulation

- `spanda run` / `spanda sim`
- Physics-lite 2D backend
- Test without hardware

### Digital twins

- `twin { mirror pose; replay true; }`
- Shadow state and replay buffer

### Package ecosystem

- `spanda init`, `build`, `test`, `install`
- `spanda.toml` manifest and lockfile

### Tooling

- Native CLI, LSP, web playground, WASM bindings
- Formatter, linter, docgen

---

## Architecture page

### Headline

Rust core, TypeScript tooling, dual-layer architecture.

### Pipeline diagram

`.sd` → lexer → parser → AST → type checker → interpreter + simulator

Optional: SIR → LLVM (experimental)

### Layers

| Layer | Technology |
|-------|------------|
| Language core | Rust (`spanda-core`) |
| CLI | Rust (`spanda-cli`) |
| Bindings | N-API, WASM |
| Developer UX | TypeScript, React, LSP |

Link to [architecture.md](https://github.com/sujaydavalgi/Spanda/blob/main/docs/architecture.md) for full diagrams.

---

## Examples page

### Showcase demos

| Example | Highlights |
|---------|------------|
| [rover_navigation.sd](examples/showcase/rover_navigation.sd) | AI planning + SafeAction |
| [warehouse_robot.sd](examples/showcase/warehouse_robot.sd) | Tasks, comm, safety zones |
| [ai_safety_violation.sd](examples/showcase/ai_safety_violation.sd) | Compile-time safety rejection |
| [hardware_compatibility.sd](examples/showcase/hardware_compatibility.sd) | Deploy verification |
| [communication_demo.sd](examples/showcase/communication_demo.sd) | Message, topic, service, action |
| [digital_twin_demo.sd](examples/showcase/digital_twin_demo.sd) | Twin + replay |

### Quick commands

```bash
spanda run examples/showcase/rover_navigation.sd
spanda verify examples/showcase/hardware_compatibility.sd --json
```

---

## Roadmap page

### v0.1.0-alpha (current)

- Stable interpreter and type system
- AI safety gate with mock providers
- Hardware verification CLI
- Simulation backend
- Package manager (local registry)
- Showcase examples and documentation

### Next

- LLVM production backend
- Published VS Code extension
- Live AI providers (OpenAI, local models)
- In-process Python/C++ FFI
- ROS2 production adapter
- Distributed multi-robot runtime

### Long-term

- Self-hosting compiler
- Digital twin cloud sync
- Fleet orchestration at scale

---

## Footer

- GitHub: [github.com/sujaydavalgi/Spanda](https://github.com/sujaydavalgi/Spanda)
- License: Apache-2.0
- Docs: [docs/README.md](https://github.com/sujaydavalgi/Spanda/blob/main/docs/README.md)

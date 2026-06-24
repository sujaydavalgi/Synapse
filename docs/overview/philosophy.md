# Philosophy & why Spanda

[← Overview](./README.md)

## Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

**Spanda** (Sanskrit: *the divine pulse*) names the creative vibration of consciousness and energy — expansion and contraction — the first stir of awareness that sustains the universe. In software terms: the coordination layer that turns perception into verified, safe action.

Long-form vision: [vision.md](../vision.md) · Product positioning: [product-strategy.md](../product-strategy.md)

---

## What is Spanda?

Spanda is an **autonomous systems platform** built around the **Spanda Language** (`.sd`) — a typed language where sensors, AI models, actuators, safety rules, and deployment targets are first-class in source code.

You write a `robot` block with sensors, actuators, safety zones, and agents. The toolchain enforces physical units, blocks unsafe AI from actuators, and checks hardware fit before deploy.

```spanda
robot SafePatrol {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  ai_model planner: LLM { provider: "mock"; model: "patrol"; }

  safety {
    max_speed = 0.5 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  behavior patrol() {
    loop every 100ms {
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "Plan motion", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }
}
```

Platform vs language: [platform-overview.md](../platform-overview.md)

---

## Why Spanda?

Building autonomous systems today means stitching together Python scripts, C++ drivers, ROS2 nodes, safety monitors, and deployment checklists — with no single platform that treats **AI output as untrusted**, **hardware fit as compile-time**, and **safety as mandatory**.

| Traditional languages focus on | Spanda focuses on |
|-------------------------------|-------------------|
| Algorithms | Autonomous systems |
| Data structures | Safety |
| Applications | Hardware awareness |
| | Capability verification |
| | Simulation |
| | Operational health & assurance |

Spanda is the coordination layer: perception, planning, safety validation, simulation, verification, deployment, and operations in one toolchain — with `.sd` as the expressive core.

More differentiators: [differentiators.md](./differentiators.md)

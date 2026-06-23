# Spanda Roadmap

Version plan for evaluators and contributors. Tiers: **Stable** (CI-backed, documented), **Experimental** (usable with caveats), **Future** (planned, not shipped).

Current release line: **v0.4.0**.

---

## v0.4 — Deploy path (current)

**Theme:** Native binaries, ROS2 polish, distributed fleet docs.

| Item | Status |
|------|--------|
| `spanda deploy --target native` | **Stable** — [native-deploy.md](./native-deploy.md) |
| `spanda compile-native` / LLVM golden paths | **Stable** |
| `spanda ros2 check` | **Stable** — [ros2-golden-path.md](./ros2-golden-path.md) |
| Distributed fleet guide (`--remote`, agents) | **Stable** — [fleet-distributed.md](./fleet-distributed.md) |
| ROS2 rclpy golden path | **Experimental** |
| Hardware adapter trait codegen | Future |
| Twin cloud SaaS | Future |

---

## v0.3 — Tooling polish (complete)

**Theme:** IDE, diagnostics, registry, install ergonomics.

| Item | Status |
|------|--------|
| `cargo install spanda` (crate renamed from `spanda-cli`) | **Stable** |
| Bundled showcase demos (`spanda demo` without clone) | **Stable** |
| `spanda fleet run` multi-robot scoping | **Stable** |
| LSP SafeAction quick-fix + keyword hover | **Stable** |
| Live IoT golden path CI | **Stable** |
| VS Code snippets | **Stable** |

---

## v0.2 — Credibility & onboarding (complete)

**Theme:** Professional OSS platform — trust table, showcase demos, docs site, one-command demos.

| Item | Status |
|------|--------|
| Feature status audit | **Stable** — [feature-status.md](./feature-status.md) |
| Trust / capability matrix in README | **Stable** |
| `spanda demo {rover,safety,verify,fleet,health}` | **Stable** |
| Showcase library, install script, benchmarks | **Stable** |
| mdBook GitHub Pages | **Stable** — `docs-site/` |
| CI showcase smoke tests | **Stable** |

---

## v1.0 — Production positioning

**Theme:** Trust for field deployment — not research features.

| Item | Tier |
|------|------|
| Interpreter + sim as supported LTS runtime | Stable |
| Safety gate + verify + replay as certified workflows | Stable |
| Native codegen for selected HAL profiles | Experimental → Stable |
| Self-hosting compiler subset | Future (not primary) |
| Blockchain / cryptocurrency adapters | **Out of scope** |
| Advanced swarm intelligence research | **Out of scope** |

---

## Completed foundation (Rust core)

Phases 1–35: language core through verification & DX. See [lean-core-roadmap.md](./lean-core-roadmap.md).

---

## Related

- [feature-status.md](./feature-status.md)
- [product-strategy.md](./product-strategy.md)
- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md)

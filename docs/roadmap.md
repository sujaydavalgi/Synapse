# Spanda Roadmap

Version plan for evaluators and contributors. Tiers: **Stable** (CI-backed, documented), **Experimental** (usable with caveats), **Future** (planned, not shipped).

Current release line: **v0.2.0** (lean-core Phases 1–35 complete).

---

## v0.2 — Credibility & onboarding (current)

**Theme:** Professional OSS platform — trust table, showcase demos, docs site, one-command demos.

| Item | Status |
|------|--------|
| Feature status audit | **Stable** — [feature-status.md](./feature-status.md) |
| Trust / capability matrix in README | **Stable** |
| Architecture diagrams | **Stable** — [diagrams/](./diagrams/) |
| Showcase library (`examples/showcase/*`) | **Stable** |
| `spanda demo {rover,safety,verify,fleet,health}` | **Stable** |
| Benchmark script + docs | **Stable** — [benchmarks.md](./benchmarks.md) |
| Known limitations doc | **Stable** — [known-limitations.md](./known-limitations.md) |
| `scripts/install.sh` + cargo install path | **Stable** |
| mdBook GitHub Pages | **Stable** — `docs-site/` |
| Improved SafeAction diagnostics | **Stable** |
| CI showcase smoke tests | **Stable** |

---

## v0.3 — Tooling polish

**Theme:** IDE, diagnostics, registry growth.

| Item | Tier |
|------|------|
| VS Code Marketplace publish | Experimental → Stable |
| LSP verification quick-fixes in editor | Experimental |
| Hover help + snippets coverage | Experimental |
| Expand hosted registry beyond curated set | Experimental |
| Package publish remote upload hardening | Experimental |
| Live IoT golden paths in more CI jobs | Experimental |

---

## v0.4 — Deploy path

**Theme:** Native binaries, ROS2, distributed fleet.

| Item | Tier |
|------|------|
| LLVM backend as optional deploy path | Experimental |
| ROS2 production adapter (zero-config goal) | Future |
| Distributed multi-robot runtime | Experimental |
| Hardware adapter trait codegen | Future |
| Twin cloud SaaS | Future |
| Formal verification integration | Future |

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

Phases 1–11: language core, agents, state machines, goals, capabilities, AI safety, units, contracts, events.

Phases 12–26: triggers, concurrency, fleet CLI, real-time, reliability, replay, regex, twins, ROS2 comm, hardware verify, tooling.

Phases 27–35: verification & DX — `spanda-capability`, health, kill switch, typed handler I/O, live AI/IoT, registry publish, debugger `every` entry.

See [lean-core-roadmap.md](./lean-core-roadmap.md) for crate-level phase detail.

---

## Self-hosting compiler (future, optional)

1. **Bootstrap** — Rust implements full language (current)
2. **Spec stabilization** — grammar + API in `docs/`
3. **Spanda subset in Spanda** — lexer/parser for minimal `.sd`
4. **Incremental migration** — type checker, codegen, runtime
5. **Rust optional** — retained for WASM/embedded

Target: self-hosting type checker before native v1.0 codegen promotion — not a blocker for v0.2–v0.4.

---

## Related

- [feature-status.md](./feature-status.md) — honest capability matrix
- [product-strategy.md](./product-strategy.md) — positioning
- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) — LLVM evolution

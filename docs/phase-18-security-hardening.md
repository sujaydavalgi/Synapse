# Phase 18 — Security, stability, and performance hardening

Follow-up to the post–Phase 17 audit (security **B**, stability **A−**, performance **B**). This plan closes P0–P3 gaps without undoing the lean-core crate layout.

## Goals

| Track | Outcome |
|-------|---------|
| **Security** | Registry integrity, safe extraction, agent auth defaults, bridge timeouts |
| **Stability** | Fewer production panics, shim sunset plan, tests in owning crates |
| **Performance** | Optional slim CLI, pipeline benchmarks, faster incremental builds preserved |

## P0 — Security (this phase)

| ID | Item | Implementation |
|----|------|----------------|
| P0.1 | **Registry tarball SHA-256** | **Complete** — sidecar + `version_checksums` in index |
| P0.2 | **Safe tar extraction** | **Complete** — `tar_extract.rs` rejects path traversal |
| P0.3 | **Agent auth defaults** | **Complete** — non-loopback bind requires `--token` |

## P1 — Stability

| ID | Item | Implementation |
|----|------|----------------|
| P1.1 | **Shim sunset** | **Complete** (Phase 19) — removed remaining `transport*` shims |
| P1.2 | **Panic audit** | **Complete** — replaced `.unwrap()` on twin state in `runtime_twin.rs`; audited CLI hot paths |
| P1.3 | **Test distribution** | **Complete** (Phase 20) — package security tests in `spanda-package`; agent auth tests in `spanda-ota` / `spanda-fleet` |

## P2 — Performance

| ID | Item | Implementation |
|----|------|----------------|
| P2.1 | **Slim CLI** | **Complete** — `spanda-cli` feature `slim` omits `spanda-llvm` (default keeps full binary) |
| P2.2 | **Bridge timeouts** | **Complete** — `SPANDA_BRIDGE_TIMEOUT_SECS` (default 30) in `spanda-bridge::protocol` |
| P2.3 | **Dependency audit CI** | **Complete** — `cargo audit` job in GitHub Actions |

## P3 — Observability

| ID | Item | Implementation |
|----|------|----------------|
| P3.1 | **Pipeline benchmark** | **Complete** — `cargo test -p spanda-driver pipeline_bench -- --ignored` |

## Phase 18b — Signed registry (complete)

| ID | Item | Status |
|----|------|--------|
| B1 | Ed25519 signatures on publish (`SPANDA_REGISTRY_SIGN_KEY`) | **Complete** |
| B2 | Verify on install (`SPANDA_REGISTRY_TRUST_KEY`, `SPANDA_REGISTRY_REQUIRE_SIGNATURE`) | **Complete** |
| B3 | `version_signatures` in registry index | **Complete** |

## Phase 21 — Complete ✓ (hosted registry signing + embedder slimming)

Goal: sign curated hosted registry tarballs in CI and make certification / FFI shims optional on `spanda-core`.

| Step | Status |
|------|--------|
| `registry-index-maintain` binary refreshes checksums + Ed25519 `version_signatures` | **Complete** |
| CI verifies hosted registry signatures (`registry/TRUST_KEY`) | **Complete** |
| Optional `certify` / `bridge` features on `spanda-core` (`default-features = false` omits FFI + certify shims) | **Complete** |

Hosted packages are signed with material `spanda-hosted-registry-v1` unless `SPANDA_REGISTRY_SIGN_KEY` is set. Trust key: `registry/TRUST_KEY`.

## Deferred (post–Phase 21)

Tier 3 product items now have **experimental foundations** in Phase 22 — see [tier-3-experimental.md](./tier-3-experimental.md). LLVM-as-primary, production blockchain, and full self-hosting remain future work.

## Verification

```bash
cargo test -p spanda-package
cargo test -p spanda-ota
cargo test -p spanda-deploy-http
cargo test -p spanda-bridge
cargo test --workspace
cargo clippy --workspace -- -D warnings
python3 scripts/update_registry_checksums.py   # refresh hosted index checksums + signatures
cargo run -p spanda-package --bin registry-index-maintain -- --verify
```

## Related

- [lean-core-roadmap.md](./lean-core-roadmap.md) — Phase 18 checklist
- [security-architecture.md](./security-architecture.md)
- [packages.md](./packages.md)

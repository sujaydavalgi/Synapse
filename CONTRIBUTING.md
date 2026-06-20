# Contributing to Spanda

Thank you for your interest in contributing to Spanda — the Autonomous Systems Language.

---

## Ways to contribute

- **Bug reports** — use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.yml)
- **Feature requests** — use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.yml)
- **Language proposals** — use the [language proposal template](.github/ISSUE_TEMPLATE/language_proposal.yml)
- **Package proposals** — use the [package proposal template](.github/ISSUE_TEMPLATE/package_proposal.yml)
- **Documentation** — fix typos, improve guides, add examples
- **Examples** — add `.sd` programs that demonstrate real use cases
- **Tests** — expand coverage for CLI, safety, verification, communication

---

## Development setup

### Prerequisites

- Node.js 18+
- Rust stable
- npm

### Clone and build

```bash
git clone https://github.com/sujaydavalgi/Spanda.git
cd Spanda
npm install
npm run build:rust
```

### Run tests

```bash
# TypeScript / Vitest
npm test

# Rust
cargo test --workspace

# Both (recommended before PR)
npm test && cargo test --workspace
```

### Format and lint

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
npm run lint
```

After bulk inline documentation edits, run:

```bash
python3 scripts/normalize_inline_docs.py
```

---

## Inline documentation

Every module and function in Rust (`crates/`) and TypeScript (`src/`, `packages/`) must be documented.

### Module headers

- **Rust:** `//!` block at the top of the file describing module purpose.
- **TypeScript:** `/** ... @module */` block at the top of the file.

### Function API docs

Place API documentation **inside** the function body (not `///` or JSDoc above the signature). Required sections: purpose, **Parameters**, **Returns**, **Options**, **Example**.

### Block comments

Before each meaningful logic block (`if`, `match`, loops, error paths), add a plain-English `//` comment explaining what that block does. Do not use generic placeholders like "Process each item" or prefix with `Logic:`.

### Tooling

| Script | Purpose |
|--------|---------|
| `scripts/add_inline_docs.py` | Generate API doc blocks |
| `scripts/add_logic_block_docs.py` | Generate contextual block comments |
| `scripts/normalize_inline_docs.py` | Fix spacing and indentation after bulk edits |

Always run `cargo fmt --all` before committing — inline doc insertion can affect brace indentation.

---

## Project structure

```
crates/spanda-core/     Authoritative language implementation (Rust)
crates/spanda-cli/      Native spanda binary
src/                    TypeScript mirror (tests, CLI wrapper, LSP helpers)
packages/lsp/           Language Server
packages/web/           Web playground
examples/               Sample .sd programs
examples/showcase/      Curated v0.1.0-alpha demos
tests/                  Vitest test suite
docs/                   Documentation
```

**Rule of thumb:** Language semantics changes go in Rust (`spanda-core`) first. TypeScript mirror (`src/`) should be updated for test parity when the change affects parsing, types, or runtime behavior.

---

## Coding standards

### Rust

- Run `cargo fmt --all` before committing
- Fix all `cargo clippy --workspace -- -D warnings` warnings
- Add tests in `crates/spanda-core/tests/` or inline `#[test]` for new behavior
- Keep changes focused — one logical change per commit
- Match existing module organization and naming

### TypeScript

- Run `npm run lint` (TypeScript `--noEmit`)
- Add Vitest tests in `tests/` for mirror changes
- Follow existing patterns in `src/parser/`, `src/runtime/`, etc.

### Spanda examples (`.sd`)

- Use physical units (`m/s`, `rad`, `ms`)
- Include safety blocks for motion examples
- AI examples must use `safety.validate()` before `actuator.execute()`
- Add showcase examples to `examples/showcase/` for high-visibility demos
- Ensure examples pass `spanda check` (and `spanda run` where applicable)

---

## How to add a language feature

1. **Propose first** — open a [language proposal](.github/ISSUE_TEMPLATE/language_proposal.yml) for non-trivial syntax or semantics changes
2. **Implement in Rust** — lexer → parser → AST → type checker → runtime (in that order)
3. **Add tests** — Rust integration tests + Vitest mirror tests
4. **Update docs** — see [Keeping documentation in sync](#keeping-documentation-in-sync) below
5. **Add an example** — demonstrate the feature in `examples/`

For v0.1.0-alpha, we are **not** adding large new language features. Focus on stability, tests, and documentation unless the proposal is critical.

---

## Keeping documentation in sync

**Rule:** Any long, big, or major update must include documentation updates in the same change set. Do not merge feature work with stale README or docs.

### What counts as major

- New language syntax, runtime behavior, or CLI commands/flags
- Architecture, crate layout, or CI workflow changes
- New integrations (ROS2, FFI, fleet, triggers, concurrency, etc.)
- Feature status or roadmap shifts
- New showcase or reference examples

Typos, internal refactors with no user-visible effect, and test-only changes usually do not need doc updates.

### Minimum checklist

After major work, review and update every file that applies:

| File | When |
|------|------|
| `README.md` | Capabilities, CLI, examples, differentiators, roadmap |
| `CHANGELOG.md` | User-visible additions, fixes, or breaking changes |
| `docs/feature-status.md` | Stability or capability matrix changes |
| `docs/getting-started.md` | New commands, workflows, or demo paths |
| `docs/README.md` | New guides or doc index changes |

Topic-specific docs (update when the area changed):

- Language: `docs/spanda-language.md` (+ dedicated guide if one exists, e.g. `docs/triggers.md`)
- Runtime / compiler: `docs/architecture.md`
- Concurrency / fleet / triggers: `docs/concurrency.md`, `docs/triggers.md`
- Hardware / verify: `docs/hardware-compatibility.md`
- Packages: `docs/packages.md`, `docs/spanda-toml.md`
- FFI / ROS2: `docs/ffi-and-ecosystem.md`
- Roadmap / strategy: `docs/roadmap.md`, `docs/product-strategy.md`
- Contributor workflow: this file (`CONTRIBUTING.md`)

Also add a runnable example and link it from `README.md` or `docs/getting-started.md` when it is a key demo.

The Cursor rule `.cursor/rules/documentation-sync.mdc` enforces this for agent-assisted sessions.

---

## How to add an example

1. Create `examples/your_example.sd` (or `examples/showcase/` for curated demos)
2. Verify locally:

```bash
spanda check examples/your_example.sd
spanda run examples/your_example.sd        # if it has runnable behavior
spanda verify examples/your_example.sd     # if it has deploy targets
```

3. Add to golden fixtures if it should stay runnable in CI (`tests/golden/manifest.json`)
4. Reference it in README or `docs/getting-started.md` if it is a key demo

---

## Pull request process

1. Fork the repository and create a feature branch
2. Make focused changes with tests
3. Run the full test suite locally
4. Open a PR against `main` with a clear description
5. CI must pass (Rust tests, fmt, clippy, TypeScript tests, build, ROS2 rclrs native job on Ubuntu 22.04)

---

## Commit messages

- Use clear, descriptive summaries
- Explain *why* in the body when non-obvious
- Split unrelated changes into separate commits
- Do not include agent or tool attribution in commit messages

---

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). Be respectful and constructive.

---

## Questions

Open a [GitHub Discussion](https://github.com/sujaydavalgi/Spanda/discussions) or issue if you are unsure where to start. We are happy to help newcomers find a good first contribution.

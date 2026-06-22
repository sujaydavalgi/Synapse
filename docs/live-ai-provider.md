# Live AI provider path (OpenAI)

Spanda v0.5 beta includes one **real AI provider path** via the Python subprocess bridge. When `OPENAI_API_KEY` is set, `openai_complete` calls the OpenAI Chat Completions API; otherwise it returns a deterministic mock string.

## Quick start

```bash
export OPENAI_API_KEY=sk-your-key
spanda run examples/ffi_openai_live.sd
```

With the `spanda-openai` registry package:

```spanda
import ai.openai;

robot Agent {
  behavior plan() {
    let text = ai.openai.complete("Plan a safe stop");
    let _ = text;
  }
}
```

```bash
spanda add spanda-openai --version 0.1.0
spanda install
```

## How it works

| Layer | Behavior |
|-------|----------|
| `extern python fn openai_complete` | Declared in Spanda, typed at boundary |
| `scripts/spanda_python_bridge.py` | Handler calls OpenAI HTTP API when key present |
| Mock fallback | Returns `mock-completion:<prompt-prefix>` when no key or on error |
| Safety gate | Raw LLM output is still `ActionProposal` until `safety.validate()` |

The bridge uses `gpt-4o-mini` and stdlib `urllib` — no extra Python packages required.

## Safety gate on live proposals

```spanda
ai_model planner: LLM { provider: "openai"; model: "gpt-4o-mini"; }

agent Navigator {
  plan {
    let proposal = planner.reason(prompt: "...", input: scene);
    let action = safety.validate(proposal);  // required before actuators
    wheels.execute(action);
  }
}
```

Unsafe direct execution remains a **compile error** regardless of provider.

## Environment

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | Enables live OpenAI calls |
| `SPANDA_PYTHON_BRIDGE` | Override bridge script path |

## Registry package

`spanda-openai` ships import path `ai.openai` with `complete(prompt)` wrapping `openai_complete`.

Default registry: `SPANDA_REGISTRY_URL` points at the hosted index in this repository (see [registry.md](./registry.md)).

Golden path: `./scripts/registry_golden_path.sh` (CI job `registry-golden-path`).

## Related

- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — bridge architecture
- [killer-demo.md](./killer-demo.md) — safety gate demo (mock planner)
- [`examples/ffi_openai_live.sd`](../examples/ffi_openai_live.sd) — minimal live call

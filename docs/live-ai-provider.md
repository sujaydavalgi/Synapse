# Live AI provider path (OpenAI + Anthropic)

Spanda v0.5 beta includes **real AI provider paths** for `ai_model` blocks:

| Provider | When live | Fallback |
|----------|-----------|----------|
| `openai` | `OPENAI_API_KEY` set | Mock provider |
| `anthropic` | `ANTHROPIC_API_KEY` set | Mock provider |
| `onnx` | `SPANDA_ONNX_MODEL_PATH` set | Mock provider |

When the key is set, `planner.reason(...)` calls the provider via the Python bridge; otherwise it falls back to the deterministic mock provider.

For FFI `extern python fn openai_complete` / `anthropic_complete`, the same bridge applies.

## Anthropic quick start

```bash
export ANTHROPIC_API_KEY=sk-ant-your-key
spanda run examples/ffi_openai_live.sd  # swap provider to "anthropic" in ai_model
```

With the `spanda-anthropic` registry package:

```spanda
import ai.anthropic;

robot Agent {
  behavior plan() {
    let text = ai.anthropic.complete("Plan a safe stop");
    let _ = text;
  }
}
```

## ONNX quick start

```bash
export SPANDA_ONNX_MODEL_PATH=/path/to/model.onnx
spanda run my_robot.sd   # ai_model with provider: "onnx"
```

With the `spanda-onnx` registry package:

```spanda
import ai.onnx;

robot Agent {
  behavior infer() {
    let result = ai.onnx.infer(input_tensor);
    let _ = result;
  }
}
```

Requires `onnxruntime` in the Python environment when using the bridge path.

## OpenAI quick start

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
| `OPENAI_API_KEY` | Enables live OpenAI calls for `provider: "openai"` and FFI bridge |
| `ANTHROPIC_API_KEY` | Enables live Anthropic calls for `provider: "anthropic"` and FFI bridge |
| `SPANDA_ONNX_MODEL_PATH` | Enables ONNX inference for `provider: "onnx"` and `spanda-onnx` package |
| `SPANDA_LIVE_AI=0` | Force mock provider even when API key is set |
| `SPANDA_PYTHON_BRIDGE` | Override bridge script path |

## Registry package

`spanda-openai` ships import path `ai.openai` with `complete(prompt)` wrapping `openai_complete`.

`spanda-anthropic` ships import path `ai.anthropic` with `complete(prompt)` wrapping `anthropic_complete`.

`spanda-onnx` ships import path `ai.onnx` with `infer(...)` wrapping the ONNX bridge when `SPANDA_ONNX_MODEL_PATH` is set.

Default registry: `SPANDA_REGISTRY_URL` points at the hosted index in this repository (see [registry.md](./registry.md)).

Golden path: `./scripts/registry_golden_path.sh` (CI job `registry-golden-path`).

## Related

- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — bridge architecture
- [killer-demo.md](./killer-demo.md) — safety gate demo (mock planner)
- [`examples/ffi_openai_live.sd`](../examples/ffi_openai_live.sd) — minimal live call

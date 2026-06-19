export type Diagnostic = { message: string; line: number; column: number };

export type CheckResponse = { ok: boolean; diagnostics: Diagnostic[] };

export type RunResponse = {
  ok: boolean;
  result?: {
    state: {
      pose: { x: number; y: number; theta: number; z?: number };
      velocity: { linear: number; angular: number };
      emergency_stop: boolean;
    };
    events: string[];
    logs: string[];
  };
  diagnostics?: Diagnostic[];
};

type SpandaWasmModule = {
  default: () => Promise<void>;
  wasm_check: (source: string) => unknown;
  wasm_run: (source: string, max: number) => unknown;
};

let wasmModule: SpandaWasmModule | null = null;

export function isWasmLoaded(): boolean {
  return wasmModule !== null;
}

async function ensureWasm(): Promise<void> {
  if (wasmModule) return;
  try {
    const init = await import("../wasm/spanda_wasm.js");
    await init.default();
    wasmModule = init as SpandaWasmModule;
  } catch {
    wasmModule = null;
  }
}

export async function checkSource(source: string): Promise<CheckResponse> {
  await ensureWasm();
  if (!wasmModule) {
    return { ok: false, diagnostics: [{ message: "WASM module not loaded", line: 1, column: 1 }] };
  }
  return wasmModule.wasm_check(source) as CheckResponse;
}

export async function runSource(source: string, maxLoopIterations: number): Promise<RunResponse> {
  await ensureWasm();
  if (!wasmModule) {
    return { ok: false, diagnostics: [{ message: "WASM module not loaded", line: 1, column: 1 }] };
  }
  return wasmModule.wasm_run(source, maxLoopIterations) as RunResponse;
}

/**
 * spanda wasm module (spanda-wasm.ts).
 * @module
 */

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

type SpandaWasmBindings = {
  wasm_check: (source: string) => unknown;
  wasm_run: (source: string, max: number) => unknown;
};

let wasmModule: SpandaWasmBindings | null = null;

export function isWasmLoaded(): boolean {
  // Description:
  //     IsWasmLoaded.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isWasmLoaded`.
  //
  // Example:
  //     const result = isWasmLoaded();
  // Description:
  //     IsWasmLoaded.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isWasmLoaded`.
  //
  // Example:
  //     const result = isWasmLoaded();

  // const result = isWasmLoaded();
  return wasmModule !== null;
}

async function ensureWasm(): Promise<void> {
  // Description:
  //     EnsureWasm.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `ensureWasm`.
  //
  // Example:
  //     const result = ensureWasm();
  // Description:
  //     EnsureWasm.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `ensureWasm`.
  //
  // Example:
  //     const result = ensureWasm();

  // const result = ensureWasm();
  if (wasmModule) return;

  // Try the operation and handle failures below.
  try {
    const init = await import("../wasm/spanda_wasm.js");
    await init.default();
    wasmModule = {
      wasm_check: (source) => init.wasm_check(source),
      wasm_run: (source, max) => init.wasm_run(source, max),
    };
  } catch {
    wasmModule = null;
  }
}

export async function checkSource(source: string): Promise<CheckResponse> {
  // Description:
  //     CheckSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: Promise<CheckResponse>
  //         Return value from `checkSource`.
  //
  // Example:
  //     const result = checkSource(source);
  // Description:
  //     CheckSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: Promise<CheckResponse>
  //         Return value from `checkSource`.
  //
  // Example:
  //     const result = checkSource(source);

  // const result = checkSource(source);
  await ensureWasm();

  // continue when wasmModule is falsy.
  if (!wasmModule) {
    return { ok: false, diagnostics: [{ message: "WASM module not loaded", line: 1, column: 1 }] };
  }
  return wasmModule.wasm_check(source) as CheckResponse;
}

export async function runSource(source: string, maxLoopIterations: number): Promise<RunResponse> {
  // Description:
  //     RunSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     maxLoopIterations: number
  //         Caller-supplied maxLoopIterations.
  //
  // Outputs:
  //     result: Promise<RunResponse>
  //         Return value from `runSource`.
  //
  // Example:
  //     const result = runSource(source, maxLoopIterations);
  // Description:
  //     RunSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     maxLoopIterations: number
  //         Caller-supplied maxLoopIterations.
  //
  // Outputs:
  //     result: Promise<RunResponse>
  //         Return value from `runSource`.
  //
  // Example:
  //     const result = runSource(source, maxLoopIterations);

  // const result = runSource(source, maxLoopIterations);
  await ensureWasm();

  // continue when wasmModule is falsy.
  if (!wasmModule) {
    return { ok: false, diagnostics: [{ message: "WASM module not loaded", line: 1, column: 1 }] };
  }
  return wasmModule.wasm_run(source, maxLoopIterations) as RunResponse;
}

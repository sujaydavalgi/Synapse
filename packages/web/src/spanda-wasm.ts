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

export type TelemetryStats = {
  total_events: number;
  device_events: number;
  sensor_events: number;
  heartbeat_events: number;
  device_heartbeat_events: number;
  health_events: number;
  session_events: number;
  runtime_metrics_events: number;
  tracked_tasks: number;
  tracked_devices: number;
};

export type TelemetryResponse = {
  ok: boolean;
  error?: string;
  stats?: TelemetryStats;
  body?: string;
};

type SpandaWasmBindings = {
  wasm_check: (source: string) => unknown;
  wasm_run: (source: string, max: number) => unknown;
  wasm_telemetry_clear: () => unknown;
  wasm_telemetry_append: (line: string) => unknown;
  wasm_telemetry_stats: () => unknown;
  wasm_telemetry_prometheus: () => unknown;
  wasm_telemetry_otlp: () => unknown;
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
      wasm_telemetry_clear: () => init.wasm_telemetry_clear(),
      wasm_telemetry_append: (line) => init.wasm_telemetry_append(line),
      wasm_telemetry_stats: () => init.wasm_telemetry_stats(),
      wasm_telemetry_prometheus: () => init.wasm_telemetry_prometheus(),
      wasm_telemetry_otlp: () => init.wasm_telemetry_otlp(),
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

function telemetryUnavailable(): TelemetryResponse {
  return { ok: false, error: "WASM module not loaded" };
}

export async function telemetryClear(): Promise<TelemetryResponse> {
  await ensureWasm();
  if (!wasmModule) return telemetryUnavailable();
  return wasmModule.wasm_telemetry_clear() as TelemetryResponse;
}

export async function telemetryAppend(line: string): Promise<TelemetryResponse> {
  await ensureWasm();
  if (!wasmModule) return telemetryUnavailable();
  return wasmModule.wasm_telemetry_append(line) as TelemetryResponse;
}

export async function telemetryStats(): Promise<TelemetryResponse> {
  await ensureWasm();
  if (!wasmModule) return telemetryUnavailable();
  return wasmModule.wasm_telemetry_stats() as TelemetryResponse;
}

export async function telemetryPrometheus(): Promise<TelemetryResponse> {
  await ensureWasm();
  if (!wasmModule) return telemetryUnavailable();
  return wasmModule.wasm_telemetry_prometheus() as TelemetryResponse;
}

export async function telemetryOtlp(): Promise<TelemetryResponse> {
  await ensureWasm();
  if (!wasmModule) return telemetryUnavailable();
  return wasmModule.wasm_telemetry_otlp() as TelemetryResponse;
}

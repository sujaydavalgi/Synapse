/**
 * index module (index.ts).
 * @module
 */

export type Diagnostic = {
  message: string;
  line: number;
  column: number;
};

export type CheckResult = {
  ok: boolean;
  diagnostics: Diagnostic[];
};

export type PoseState = {
  x: number;
  y: number;
  theta: number;
  z?: number;
};

export type VelocityState = {
  linear: number;
  angular: number;
};

export type RobotState = {
  pose: PoseState;
  velocity: VelocityState;
  emergency_stop: boolean;
};

export type TaskMetrics = {
  name: string;
  priority: string;
  interval_ms: number;
  ticks: number;
  skipped: number;
  missed_deadlines: number;
  budget_violations: number;
  last_duration_ms: number;
  max_duration_ms: number;
};

export type SchedulerMetrics = {
  multiplexed_tasks: number;
  scheduler_ticks: number;
  base_tick_ms: number;
  emergency_stops: number;
};

export type ExecutionMetrics = {
  spawns: number;
  joins: number;
  parallel_blocks: number;
  fire_and_forget_spawns: number;
};

export type RuntimeTelemetry = {
  tasks: TaskMetrics[];
  scheduler: SchedulerMetrics;
  execution: ExecutionMetrics;
  replay_frames: number;
};

export type RunResult = {
  state: RobotState;
  events: string[];
  logs: string[];
  metrics: RuntimeTelemetry;
};

export type RunOptions = {
  entryBehavior?: string;
  maxLoopIterations?: number;
  traceScheduler?: boolean;
  traceTasks?: boolean;
  replayTrace?: boolean;
};

export interface SpandaNative {
  checkSource(source: string): CheckResult;
  runSource(source: string, options?: RunOptions): RunResult;
  coreVersion(): string;
}

let native: SpandaNative | null = null;
let loadAttempted = false;

export function isNativeAvailable(): boolean {
  // Description:
  //     IsNativeAvailable.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isNativeAvailable`.
  //
  // Example:
  //     const result = isNativeAvailable();
  // Description:
  //     IsNativeAvailable.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isNativeAvailable`.
  //
  // Example:
  //     const result = isNativeAvailable();

  // const result = isNativeAvailable();
  loadNative();
  return native !== null;
}

function loadNative(): void {
  // Description:
  //     LoadNative.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = loadNative();
  // Description:
  //     LoadNative.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = loadNative();

  // const result = loadNative();
  if (loadAttempted) return;
  loadAttempted = true;

  // Try the operation and handle failures below.
  try {
    const mod = require("./native.js") as SpandaNative;
    native = mod;
  } catch {
    native = null;
  }
}

export function checkSource(source: string): CheckResult | null {
  // Description:
  //     CheckSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: CheckResult | null
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
  //     result: CheckResult | null
  //         Return value from `checkSource`.
  //
  // Example:
  //     const result = checkSource(source);

  // const result = checkSource(source);
  loadNative();

  // continue when native is falsy.
  if (!native) return null;
  return native.checkSource(source);
}

export function runSource(source: string, options?: RunOptions): RunResult | null {
  // Description:
  //     RunSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     options?: RunOptions
  //         Caller-supplied options?.
  //
  // Outputs:
  //     result: RunResult | null
  //         Return value from `runSource`.
  //
  // Example:
  //     const result = runSource(source, options?);
  // Description:
  //     RunSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     options?: RunOptions
  //         Caller-supplied options?.
  //
  // Outputs:
  //     result: RunResult | null
  //         Return value from `runSource`.
  //
  // Example:
  //     const result = runSource(source, options?);

  // const result = runSource(source, options?);
  loadNative();

  // continue when native is falsy.
  if (!native) return null;
  return native.runSource(source, options);
}

export function coreVersion(): string | null {
  // Description:
  //     CoreVersion.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `coreVersion`.
  //
  // Example:
  //     const result = coreVersion();
  // Description:
  //     CoreVersion.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `coreVersion`.
  //
  // Example:
  //     const result = coreVersion();

  // const result = coreVersion();
  loadNative();
  return native?.coreVersion() ?? null;
}

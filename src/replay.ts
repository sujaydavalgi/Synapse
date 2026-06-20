/**
 * Deterministic mission trace recording and replay for simulation runs.
 * @module
 */

export type TraceFrame = {
  simTimeMs: number;
  event: string;
  payload?: unknown;
};

export type MissionTrace = {
  version: number;
  source: string;
  deterministic: boolean;
  frames: TraceFrame[];
};

export type TraceVerification = {
  ok: boolean;
  matched: number;
  mismatches: string[];
};

export function createMissionTrace(source: string): MissionTrace {
  // Create an empty mission trace for a source program.
  //
  // Parameters:
  // - `source` — `.sd` file path or label
  //
  // Returns:
  // Empty trace container.
  //
  // Options:
  // None.
  //
  // Example:
  // const trace = createMissionTrace("rover.sd");

  return {
    version: 1,
    source,
    deterministic: true,
    frames: [],
  };
}

export function recordTraceFrame(
  trace: MissionTrace,
  simTimeMs: number,
  event: string,
  payload: unknown = {},
): void {
  // Append one trace frame at the current simulation time.
  //
  // Parameters:
  // - `trace` — mission trace to mutate
  // - `simTimeMs` — simulation clock in milliseconds
  // - `event` — event label
  // - `payload` — structured payload
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:
  // recordTraceFrame(trace, 10.0, "task_tick", { task: "sense" });

  trace.frames.push({ simTimeMs, event, payload });
}

export function traceFramesFrom(trace: MissionTrace, offsetMs: number): TraceFrame[] {
  // Return trace frames starting at or after the requested offset.
  //
  // Parameters:
  // - `trace` — mission trace
  // - `offsetMs` — replay start offset in milliseconds
  //
  // Returns:
  // Slice of frames at/after the offset.
  //
  // Options:
  // None.
  //
  // Example:
  // const slice = traceFramesFrom(trace, 30_000);

  const idx = trace.frames.findIndex((frame) => frame.simTimeMs >= offsetMs);
  return idx === -1 ? [] : trace.frames.slice(idx);
}

export function parseReplayOffset(raw: string): number {
  // Parse replay offset strings such as `T+00:30` into milliseconds.
  //
  // Parameters:
  // - `raw` — CLI offset argument
  //
  // Returns:
  // Offset in milliseconds.
  //
  // Options:
  // None.
  //
  // Example:
  // const ms = parseReplayOffset("T+00:30");

  const asNumber = Number(raw);
  if (!Number.isNaN(asNumber)) {
    return asNumber;
  }

  if (!raw.startsWith("T+")) {
    throw new Error(`Invalid replay offset '${raw}'; expected T+mm:ss or milliseconds`);
  }

  const value = raw.slice(2);
  const parts = value.split(":");
  let totalSecs = 0;

  if (parts.length === 2) {
    totalSecs = Number(parts[0]) * 60 + Number(parts[1]);
  } else if (parts.length === 3) {
    totalSecs = Number(parts[0]) * 3600 + Number(parts[1]) * 60 + Number(parts[2]);
  } else {
    throw new Error(`Invalid replay offset '${raw}'; expected T+mm:ss`);
  }

  return totalSecs * 1000;
}

export function verifyTraces(
  expected: MissionTrace,
  actual: MissionTrace,
  fromMs: number,
): TraceVerification {
  // Compare two mission traces from the same offset for deterministic replay checks.
  //
  // Parameters:
  // - `expected` — reference trace loaded from disk
  // - `actual` — trace recorded during a replay run
  // - `fromMs` — comparison start offset in milliseconds
  //
  // Returns:
  // Verification summary with mismatched frame details.
  //
  // Options:
  // None.
  //
  // Example:
  // const report = verifyTraces(expected, actual, 0.0);

  const exp = traceFramesFrom(expected, fromMs);
  const act = traceFramesFrom(actual, fromMs);
  const mismatches: string[] = [];
  const shared = Math.min(exp.length, act.length);

  for (let index = 0; index < shared; index++) {
    if (exp[index]!.event !== act[index]!.event) {
      mismatches.push(
        `frame ${index}: expected event '${exp[index]!.event}', got '${act[index]!.event}'`,
      );
    } else if (Math.abs(exp[index]!.simTimeMs - act[index]!.simTimeMs) > 0.001) {
      mismatches.push(
        `frame ${index} event '${exp[index]!.event}': expected t=${exp[index]!.simTimeMs.toFixed(3)}ms, got t=${act[index]!.simTimeMs.toFixed(3)}ms`,
      );
    }
  }

  if (exp.length !== act.length) {
    mismatches.push(`frame count mismatch: expected ${exp.length}, got ${act.length}`);
  }

  return {
    ok: mismatches.length === 0,
    matched: shared,
    mismatches,
  };
}

export function serializeMissionTrace(trace: MissionTrace): string {
  return JSON.stringify(trace, null, 2);
}

export function deserializeMissionTrace(text: string): MissionTrace {
  return JSON.parse(text) as MissionTrace;
}

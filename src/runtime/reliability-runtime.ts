/**
 * Runtime reliability state for watchdogs, pipelines, modes, and mission traces.
 * @module
 */

import type { RobotDecl, Stmt } from "../ast/nodes.js";
import type { PipelineDecl, WatchdogDecl } from "../foundations.js";
import { createMissionTrace, recordTraceFrame, type MissionTrace } from "../replay.js";
import { recordTaskHeartbeat } from "../telemetry-store.js";

const RUNTIME_TASK_COST_MS = 5;

export type ReliabilityHost = {
  executeBlock(stmts: Stmt[]): void;
  log(message: string): void;
  getSimTimeMs(): number;
};

export class ReliabilityRuntime {
  activeMode = "normal";
  simTimeMs = 0;
  pipelines = new Map<string, { budgetMs: number; body: Stmt[] }>();
  modes = new Map<string, Stmt[]>();
  watchdogs: Array<{
    name: string;
    target?: string;
    timeoutMs: number;
    body: Stmt[];
    lastFiredAtMs?: number;
  }> = [];
  taskHeartbeats = new Map<string, number>();
  missionTrace: MissionTrace | null = null;

  loadFromRobot(robot: RobotDecl, recordTrace: boolean, traceSource?: string): void {

    // Reset and load reliability declarations from a robot block.
    this.pipelines.clear();
    this.modes.clear();
    this.watchdogs = [];
    this.taskHeartbeats.clear();
    this.activeMode = "normal";
    this.missionTrace = recordTrace ? createMissionTrace(traceSource ?? "program.sd") : null;

    for (const pipeline of robot.pipelines ?? []) {
      this.pipelines.set(pipeline.name, { budgetMs: pipeline.budgetMs, body: pipeline.body });
    }
    for (const watchdog of robot.watchdogs ?? []) {
      this.watchdogs.push({
        name: watchdog.name,
        target: watchdog.target ?? undefined,
        timeoutMs: watchdog.timeoutMs,
        body: watchdog.body,
      });
    }
    for (const mode of robot.modes ?? []) {
      this.modes.set(mode.name, mode.body);
    }
  }

  enterMode(mode: string, host: ReliabilityHost): void {

    // Switch operating mode and execute its body when declared.
    this.activeMode = mode;
    const body = this.modes.get(mode);
    if (!body) {
      host.log(`mode: entered '${mode}' (no body declared)`);
      return;
    }
    host.executeBlock(body);
    this.recordEvent(host, "mode_enter", { mode });
    host.log(`mode: entered '${mode}'`);
  }

  executePipeline(name: string, host: ReliabilityHost): void {

    // Run a named pipeline and record budget telemetry.
    const pipeline = this.pipelines.get(name);
    if (!pipeline) {
      throw new Error(`unknown pipeline '${name}'`);
    }
    const started = performance.now();
    host.executeBlock(pipeline.body);
    const durationMs = Math.max(RUNTIME_TASK_COST_MS, performance.now() - started);
    if (durationMs > pipeline.budgetMs) {
      host.log(
        `pipeline '${name}': budget ${pipeline.budgetMs}ms exceeded (${durationMs.toFixed(2)}ms)`,
      );
    } else {
      host.log(
        `pipeline '${name}': completed in ${durationMs.toFixed(2)}ms (budget ${pipeline.budgetMs}ms)`,
      );
    }
    this.recordEvent(host, "pipeline_run", {
      pipeline: name,
      durationMs,
      budgetMs: pipeline.budgetMs,
    });
  }

  touchHeartbeat(taskName: string, simTimeMs: number): void {
    this.taskHeartbeats.set(taskName, simTimeMs);
    recordTaskHeartbeat(taskName, simTimeMs);
  }

  checkWatchdogs(host: ReliabilityHost): void {

    // Evaluate watchdog timeouts against task heartbeats.
    for (const watchdog of this.watchdogs) {
      const referenceMs = watchdog.target
        ? (this.taskHeartbeats.get(watchdog.target) ?? 0)
        : 0;
      const elapsed = host.getSimTimeMs() - referenceMs;
      const shouldFire =
        elapsed >= watchdog.timeoutMs &&
        (watchdog.lastFiredAtMs === undefined ||
          host.getSimTimeMs() - watchdog.lastFiredAtMs >= watchdog.timeoutMs);
      if (!shouldFire) {
        continue;
      }
      watchdog.lastFiredAtMs = host.getSimTimeMs();
      this.recordEvent(host, "watchdog_timeout", {
        watchdog: watchdog.name,
        elapsedMs: elapsed,
      });
      host.log(
        `watchdog '${watchdog.name}': timeout after ${elapsed.toFixed(1)}ms (limit ${watchdog.timeoutMs}ms)`,
      );
      host.executeBlock(watchdog.body);
    }
  }

  recordEvent(host: ReliabilityHost, event: string, payload: unknown): void {
    if (!this.missionTrace) {
      return;
    }
    recordTraceFrame(this.missionTrace, host.getSimTimeMs(), event, payload);
  }

  takeMissionTrace(): MissionTrace | null {
    const trace = this.missionTrace;
    this.missionTrace = null;
    return trace;
  }
}

export function pipelineFromDecl(decl: PipelineDecl): {
  // Description:
  //     PipelineFromDecl.
  //
  // Inputs:
  //     decl: PipelineDecl
  //         Caller-supplied decl.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = pipelineFromDecl(decl);
 name: string; budgetMs: number; body: Stmt[] } {
  return { name: decl.name, budgetMs: decl.budgetMs, body: decl.body };
}

export function watchdogFromDecl(decl: WatchdogDecl): {
  // Description:
  //     WatchdogFromDecl.
  //
  // Inputs:
  //     decl: WatchdogDecl
  //         Caller-supplied decl.
  //
  // Outputs:
  //     None.
  //
  // Example:

  //     const result = watchdogFromDecl(decl);

  name: string;
  target?: string;
  timeoutMs: number;
  body: Stmt[];
} {
  return {
    name: decl.name,
    target: decl.target ?? undefined,
    timeoutMs: decl.timeoutMs,
    body: decl.body,
  };
}

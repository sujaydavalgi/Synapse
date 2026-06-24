/**
 * Runtime reliability state for watchdogs, pipelines, modes, and mission traces.
 * @module
 */

import type { RobotDecl, Stmt } from "../ast/nodes.js";
import type { PipelineDecl, WatchdogDecl } from "../foundations.js";
import { createMissionTrace, recordTraceFrame, recordTraceFrameWithState, type MissionTrace, type ReplayStateSnapshot } from "../replay.js";
import { recordTaskHeartbeat } from "../telemetry-store.js";

const RUNTIME_TASK_COST_MS = 5;

export type ReliabilityHost = {
  executeBlock(stmts: Stmt[]): void;
  log(message: string): void;
  getSimTimeMs(): number;
};

type TaskMetricState = {
  name: string;
  priority: string;
  interval_ms: number;
  ticks: number;
  skipped: number;
  missed_deadlines: number;
  budget_violations: number;
  last_duration_ms: number;
  max_duration_ms: number;
  max_jitter_ms: number;
  jitter_violations: number;
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
  private taskMetrics = new Map<string, TaskMetricState>();
  private schedulerMetrics = {
    multiplexed_tasks: 0,
    scheduler_ticks: 0,
    base_tick_ms: 0,
    emergency_stops: 0,
  };
  private executionMetrics = {
    spawns: 0,
    joins: 0,
    parallel_blocks: 0,
    fire_and_forget_spawns: 0,
  };

  loadFromRobot(robot: RobotDecl, recordTrace: boolean, traceSource?: string): void {

    // Reset and load reliability declarations from a robot block.
    this.pipelines.clear();
    this.modes.clear();
    this.watchdogs = [];
    this.taskHeartbeats.clear();
    this.taskMetrics.clear();
    this.schedulerMetrics = {
      multiplexed_tasks: 0,
      scheduler_ticks: 0,
      base_tick_ms: 0,
      emergency_stops: 0,
    };
    this.executionMetrics = {
      spawns: 0,
      joins: 0,
      parallel_blocks: 0,
      fire_and_forget_spawns: 0,
    };
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

  touchHeartbeat(taskName: string, simTimeMs: number, robotId?: string): void {
    this.taskHeartbeats.set(taskName, simTimeMs);
    recordTaskHeartbeat(taskName, simTimeMs, robotId);
  }

  configureScheduler(multiplexedTasks: number, baseTickMs: number): void {
    this.schedulerMetrics.multiplexed_tasks = multiplexedTasks;
    this.schedulerMetrics.base_tick_ms = baseTickMs;
  }

  recordSchedulerTick(): void {
    this.schedulerMetrics.scheduler_ticks += 1;
  }

  recordEmergencyStop(): void {
    this.schedulerMetrics.emergency_stops += 1;
  }

  recordTaskTick(
    name: string,
    priority: string,
    intervalMs: number,
    durationMs: number,
    options?: { skipped?: boolean; budgetViolation?: boolean },
  ): void {
    const existing = this.taskMetrics.get(name) ?? {
      name,
      priority,
      interval_ms: intervalMs,
      ticks: 0,
      skipped: 0,
      missed_deadlines: 0,
      budget_violations: 0,
      last_duration_ms: 0,
      max_duration_ms: 0,
      max_jitter_ms: 0,
      jitter_violations: 0,
    };
    if (options?.skipped) {
      existing.skipped += 1;
    } else {
      existing.ticks += 1;
      existing.last_duration_ms = durationMs;
      existing.max_duration_ms = Math.max(existing.max_duration_ms, durationMs);
      const jitter = Math.abs(durationMs - intervalMs);
      existing.max_jitter_ms = Math.max(existing.max_jitter_ms, jitter);
      if (durationMs > intervalMs) {
        existing.missed_deadlines += 1;
      }
    }
    if (options?.budgetViolation) {
      existing.budget_violations += 1;
    }
    this.taskMetrics.set(name, existing);
  }

  recordSpawn(fireAndForget = false): void {
    this.executionMetrics.spawns += 1;
    if (fireAndForget) {
      this.executionMetrics.fire_and_forget_spawns += 1;
    }
  }

  recordJoin(): void {
    this.executionMetrics.joins += 1;
  }

  recordParallelBlock(): void {
    this.executionMetrics.parallel_blocks += 1;
  }

  snapshotRuntimeMetrics(replayFrames = 0): Record<string, unknown> {
    const tasks: Record<string, Omit<TaskMetricState, "name">> = {};
    for (const [name, body] of this.taskMetrics.entries()) {
      const { name: _ignored, ...metrics } = body;
      tasks[name] = metrics;
    }
    return {
      tasks,
      scheduler: { ...this.schedulerMetrics, sim_time_ms: this.simTimeMs },
      execution: { ...this.executionMetrics },
      replay_frames: replayFrames,
    };
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

  recordMissionEvent(
    host: ReliabilityHost,
    event: string,
    payload: unknown,
    state?: ReplayStateSnapshot,
  ): void {
    if (!this.missionTrace) {
      return;
    }
    const simTimeMs = host.getSimTimeMs();
    if (state) {
      recordTraceFrameWithState(this.missionTrace, simTimeMs, event, payload, state);
      return;
    }
    recordTraceFrame(this.missionTrace, simTimeMs, event, payload);
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

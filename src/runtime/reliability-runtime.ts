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

type PipelineMetricState = {
  name: string;
  budget_ms: number;
  executions: number;
  total_duration_ms: number;
  deadline_misses: number;
  slow_stages: number;
};

type WatchdogMetricState = {
  name: string;
  timeouts: number;
  last_timeout_ms: number;
};

type TriggerMetricState = {
  name: string;
  category: string;
  priority: string;
  executions: number;
  failures: number;
  missed_deadlines: number;
  last_duration_ms: number;
  max_duration_ms: number;
};

type TopicMetricState = {
  path: string;
  deadline_misses: number;
  last_elapsed_ms: number;
};

type ProviderMetricState = {
  provider_key: string;
  category: string;
  calls: number;
  failures: number;
  last_duration_ms: number;
  max_duration_ms: number;
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
  private pipelineMetrics = new Map<string, PipelineMetricState>();
  private watchdogMetrics = new Map<string, WatchdogMetricState>();
  private triggerMetrics = new Map<string, TriggerMetricState>();
  private topicMetrics = new Map<string, TopicMetricState>();
  private providerMetrics = new Map<string, ProviderMetricState>();
  private topicQos = new Map<string, { deadline_ms: number }>();
  private topicLastPublishMs = new Map<string, number>();
  private topicDeadlineMisses = new Map<string, number>();

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
    this.pipelineMetrics.clear();
    this.watchdogMetrics.clear();
    this.triggerMetrics.clear();
    this.topicMetrics.clear();
    this.providerMetrics.clear();
    this.topicQos.clear();
    this.topicLastPublishMs.clear();
    this.topicDeadlineMisses.clear();
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
    this.recordPipelineExecution(name, pipeline.budgetMs, durationMs, durationMs > pipeline.budgetMs);
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

  recordPipelineExecution(
    name: string,
    budgetMs: number,
    durationMs: number,
    slowStage = false,
  ): void {
    const existing = this.pipelineMetrics.get(name) ?? {
      name,
      budget_ms: budgetMs,
      executions: 0,
      total_duration_ms: 0,
      deadline_misses: 0,
      slow_stages: 0,
    };
    existing.executions += 1;
    existing.total_duration_ms += durationMs;
    if (durationMs > budgetMs) {
      existing.deadline_misses += 1;
    }
    if (slowStage) {
      existing.slow_stages += 1;
    }
    this.pipelineMetrics.set(name, existing);
  }

  recordWatchdogTimeout(name: string, simTimeMs: number): void {
    const existing = this.watchdogMetrics.get(name) ?? {
      name,
      timeouts: 0,
      last_timeout_ms: 0,
    };
    existing.timeouts += 1;
    existing.last_timeout_ms = simTimeMs;
    this.watchdogMetrics.set(name, existing);
  }

  recordTriggerExecution(
    name: string,
    category: string,
    priority: string,
    durationMs: number,
    failed = false,
  ): void {
    const existing = this.triggerMetrics.get(name) ?? {
      name,
      category,
      priority,
      executions: 0,
      failures: 0,
      missed_deadlines: 0,
      last_duration_ms: 0,
      max_duration_ms: 0,
    };
    existing.executions += 1;
    if (failed) {
      existing.failures += 1;
    }
    existing.last_duration_ms = durationMs;
    existing.max_duration_ms = Math.max(existing.max_duration_ms, durationMs);
    this.triggerMetrics.set(name, existing);
  }

  recordTopicPublish(path: string, elapsedMs: number, deadlineMiss = false): void {
    const existing = this.topicMetrics.get(path) ?? {
      path,
      deadline_misses: 0,
      last_elapsed_ms: 0,
    };
    existing.last_elapsed_ms = elapsedMs;
    if (deadlineMiss) {
      existing.deadline_misses += 1;
    }
    this.topicMetrics.set(path, existing);
  }

  recordProviderCall(
    providerKey: string,
    category: string,
    durationMs: number,
    failed = false,
  ): void {
    const existing = this.providerMetrics.get(providerKey) ?? {
      provider_key: providerKey,
      category,
      calls: 0,
      failures: 0,
      last_duration_ms: 0,
      max_duration_ms: 0,
    };
    existing.calls += 1;
    if (failed) {
      existing.failures += 1;
    }
    existing.last_duration_ms = durationMs;
    existing.max_duration_ms = Math.max(existing.max_duration_ms, durationMs);
    this.providerMetrics.set(providerKey, existing);
  }

  registerTopicQos(path: string, deadlineMs: number | null | undefined): void {
    if (deadlineMs != null && deadlineMs > 0) {
      this.topicQos.set(path, { deadline_ms: deadlineMs });
    }
  }

  noteTopicPublish(path: string, simTimeMs: number): void {
    this.topicLastPublishMs.set(path, simTimeMs);
    this.recordTopicPublish(path, 0);
  }

  checkTopicQosDeadlines(host: ReliabilityHost): void {
    for (const [path, qos] of this.topicQos.entries()) {
      const last = this.topicLastPublishMs.get(path) ?? 0;
      if (last <= 0) {
        continue;
      }
      const elapsed = host.getSimTimeMs() - last;
      if (elapsed <= qos.deadline_ms) {
        continue;
      }
      const misses = this.topicDeadlineMisses.get(path) ?? 0;
      if (misses === 0 || elapsed > qos.deadline_ms * (misses + 1)) {
        this.topicDeadlineMisses.set(path, misses + 1);
        this.recordTopicPublish(path, elapsed, true);
        this.recordMissionEvent(host, "topic_deadline_miss", {
          topic: path,
          elapsed_ms: elapsed,
          deadline_ms: qos.deadline_ms,
        });
        host.log(
          `topic '${path}': deadline miss (${elapsed.toFixed(1)}ms > ${qos.deadline_ms}ms)`,
        );
      }
    }
  }

  snapshotRuntimeMetrics(replayFrames = 0): Record<string, unknown> {
    const tasks: Record<string, Omit<TaskMetricState, "name">> = {};
    for (const [name, body] of this.taskMetrics.entries()) {
      const { name: _ignored, ...metrics } = body;
      tasks[name] = metrics;
    }
    const pipelines: Record<string, Omit<PipelineMetricState, "name">> = {};
    for (const [name, body] of this.pipelineMetrics.entries()) {
      const { name: _ignored, ...metrics } = body;
      pipelines[name] = metrics;
    }
    const watchdogs: Record<string, Omit<WatchdogMetricState, "name">> = {};
    for (const [name, body] of this.watchdogMetrics.entries()) {
      const { name: _ignored, ...metrics } = body;
      watchdogs[name] = metrics;
    }
    const triggers: Record<string, Omit<TriggerMetricState, "name">> = {};
    for (const [name, body] of this.triggerMetrics.entries()) {
      const { name: _ignored, ...metrics } = body;
      triggers[name] = metrics;
    }
    const topics: Record<string, Omit<TopicMetricState, "path">> = {};
    for (const [path, body] of this.topicMetrics.entries()) {
      const { path: _ignored, ...metrics } = body;
      topics[path] = metrics;
    }
    const providers: Record<string, Omit<ProviderMetricState, "provider_key">> = {};
    for (const [key, body] of this.providerMetrics.entries()) {
      const { provider_key: _ignored, ...metrics } = body;
      providers[key] = metrics;
    }
    return {
      tasks,
      triggers,
      pipelines,
      watchdogs,
      topics,
      providers,
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
      this.recordWatchdogTimeout(watchdog.name, host.getSimTimeMs());
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
  name: string;
  budgetMs: number;
  body: Stmt[];
} {
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

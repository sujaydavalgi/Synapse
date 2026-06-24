import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { Interpreter } from "../src/runtime/interpreter.js";
import {
  ConditionTriggerState,
  TriggerRegistry,
  triggerKindFromLegacyEventName,
} from "../src/runtime/trigger-registry.js";

describe("trigger registry", () => {
  it("maps legacy connectivity event names", () => {
    expect(triggerKindFromLegacyEventName("gps.lost")).toEqual({
      kind: "connectivity",
      domain: "gps",
      event: "lost",
    });
  });

  it("sorts handlers by priority", () => {
    const registry = new TriggerRegistry();
    registry.register({
      kind: "TriggerHandlerDecl",
      triggerKind: { kind: "event", name: "low_prio" },
      priority: "low",
      returnType: { kind: "void" },
      body: [],
      span: { start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 1, offset: 0 } },
    });
    registry.register({
      kind: "TriggerHandlerDecl",
      triggerKind: { kind: "event", name: "critical_prio" },
      priority: "critical",
      returnType: { kind: "void" },
      body: [],
      span: { start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 1, offset: 0 } },
    });
    const sorted = TriggerRegistry.sortedByPriority(registry.handlersForEvent("critical_prio"));
    expect(sorted[0]?.priority).toBe("critical");
  });

  it("fires condition triggers on rising edges only", () => {
    const state = new ConditionTriggerState();
    expect(state.shouldFire(1, true)).toBe(true);
    expect(state.shouldFire(1, true)).toBe(false);
    expect(state.shouldFire(1, false)).toBe(false);
    expect(state.shouldFire(1, true)).toBe(true);
  });
});

describe("interpreter trigger registry", () => {
  it("records timer trigger executions in runtime metrics", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  every 20ms {
    wheels.stop();
  }
  task tick every 100ms {
    wheels.stop();
  }
}
`;
    const program = parse(tokenize(source));
    const interpreter = new Interpreter({
      backend: createDefaultSimulator(),
      maxLoopIterations: 8,
    });
    interpreter.run(program);
    const metrics = interpreter.collectRuntimeMetrics();
    const timerMetrics = metrics.triggers as Record<string, { executions: number }>;
    expect(timerMetrics["timer:20ms"]?.executions ?? 0).toBeGreaterThan(0);
  });

  it("records when trigger executions once per rising edge", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  when true -> Bool {
    wheels.stop();
  }
  task tick every 100ms {
    wheels.stop();
  }
}
`;
    const program = parse(tokenize(source));
    const interpreter = new Interpreter({
      backend: createDefaultSimulator(),
      maxLoopIterations: 5,
    });
    interpreter.run(program);
    const metrics = interpreter.collectRuntimeMetrics();
    const whenMetrics = (metrics.triggers as Record<string, { executions: number }>).when;
    expect(whenMetrics?.executions).toBe(1);
  });
});

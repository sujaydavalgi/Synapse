import { describe, it, expect } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { compileWithRegistry, run } from "../src/compile.js";
import { checkWithRegistry } from "../src/types/index.js";
import { ModuleRegistry } from "../src/modules/index.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { Interpreter } from "../src/runtime/interpreter.js";

describe("TypeScript ModuleRegistry", () => {
  it("resolves cross-module export at type-check time", () => {
    const planning = `
module navigation.path_planning;

export fn plan_path() -> Path {
  return trajectory(from: pose(x: 0.0 m, y: 0.0 m), to: pose(x: 2.0 m, y: 0.0 m), steps: 4);
}
`;
    const main = `
module navigation;

import navigation.path_planning;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let route = plan_path();
    let _ = route;
    wheels.stop();
  }
}
`;
    const registry = new ModuleRegistry();
    registry.register("navigation.path_planning", parse(tokenize(planning)));
    expect(() => checkWithRegistry(parse(tokenize(main)), registry)).not.toThrow();
  });

  it("rejects private functions from importer", () => {
    const planning = `
module navigation.path_planning;

private fn helper() -> Path {
  return trajectory(from: pose(x: 0.0 m, y: 0.0 m), to: pose(x: 1.0 m, y: 0.0 m), steps: 2);
}
`;
    const main = `
module navigation;
import navigation.path_planning;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    helper();
    wheels.stop();
  }
}
`;
    const registry = new ModuleRegistry();
    registry.register("navigation.path_planning", parse(tokenize(planning)));
    expect(() => checkWithRegistry(parse(tokenize(main)), registry)).toThrow();
  });

  it("runs imported export fn via TS interpreter", () => {
    const planning = `
module navigation.path_planning;

export fn plan_path() -> Path {
  return trajectory(from: pose(x: 0.0 m, y: 0.0 m), to: pose(x: 2.0 m, y: 0.0 m), steps: 4);
}
`;
    const main = `
module navigation;
import navigation.path_planning;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let route = plan_path();
    let _ = route;
    wheels.stop();
  }
}
`;
    const registry = new ModuleRegistry();
    registry.register("navigation.path_planning", parse(tokenize(planning)));
    const { program } = compileWithRegistry(main, registry);
    expect(() =>
      run(program, { backend: createDefaultSimulator(), maxLoopIterations: 1, moduleRegistry: registry }),
    ).not.toThrow();
  });

  it("dispatches official package imports through the provider registry", () => {
    const gpsModule = `
module positioning.gps;

export fn read() -> GeoPoint {
  return GeoPoint(lat: 0.0, lon: 0.0);
}
`;
    const main = `
module main;

import positioning.gps;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let point = read();
    let _ = point;
    wheels.stop();
  }
}
`;
    const registry = new ModuleRegistry();
    registry.register("positioning.gps", parse(tokenize(gpsModule)));
    const { program } = compileWithRegistry(main, registry);
    const interpreter = new Interpreter({
      backend: createDefaultSimulator(),
      maxLoopIterations: 1,
      moduleRegistry: registry,
      officialPackages: ["spanda-gps"],
    });
    interpreter.run(program);
    const metrics = interpreter.collectRuntimeMetrics();
    expect(
      (metrics.providers as Record<string, { calls: number }>)["positioning.gps"]?.calls,
    ).toBe(1);
  });
});

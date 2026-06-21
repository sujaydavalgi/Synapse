import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { compileFile } from "../src/compile.js";
import {
  coordinateSwarms,
  emptySwarmState,
} from "../src/swarm-coordinator.js";

const swarmExample = join(import.meta.dirname, "..", "examples/robotics/swarm_coordination.sd");

describe("swarm coordinator (TS mirror)", () => {
  it("round_robin advances one member per tick", () => {
    const { program } = compileFile(swarmExample, "typescript");
    const state = emptySwarmState();
    const first = coordinateSwarms(program, "swarm_coordination.sd", state);
    const roundRobin = first.swarms.find((swarm) => swarm.policy === "round_robin");
    expect(roundRobin?.stepsAdvanced).toBe(1);
    expect(roundRobin?.members).toHaveLength(1);
    const firstMember = roundRobin?.members[0]?.robotName;

    const second = coordinateSwarms(program, "swarm_coordination.sd", state);
    const next = second.swarms.find((swarm) => swarm.policy === "round_robin");
    expect(next?.members[0]?.robotName).not.toBe(firstMember);
  });

  it("broadcast advances all fleet members", () => {
    const { program } = compileFile(swarmExample, "typescript");
    const state = emptySwarmState();
    const result = coordinateSwarms(program, "swarm_coordination.sd", state);
    const broadcast = result.swarms.find((swarm) => swarm.policy === "broadcast");
    expect(broadcast?.members).toHaveLength(3);
    expect(broadcast?.stepsAdvanced).toBe(3);
  });
});

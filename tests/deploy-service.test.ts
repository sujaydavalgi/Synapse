import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { compileFile } from "../src/compile.js";
import {
  applyRollout,
  buildDeployPlan,
  emptyDeployState,
  planRollout,
  rollbackTargets,
} from "../src/deploy-service.js";

const otaExample = join(import.meta.dirname, "..", "examples/robotics/ota_deployment.sd");

describe("deploy service (TS mirror)", () => {
  it("builds a deploy plan from OTA example", () => {
    const { program } = compileFile(otaExample, "typescript");
    const plan = buildDeployPlan(program, "ota_deployment.sd", "1.0.0");
    expect(plan.assignments).toHaveLength(1);
    expect(plan.assignments[0]).toEqual({
      robotName: "RoverProgram",
      hardware: "JetsonOrin",
    });
  });

  it("plans canary and full rollouts", () => {
    const { program } = compileFile(otaExample, "typescript");
    const plan = buildDeployPlan(program, "ota_deployment.sd", "1.0.0");
    const full = planRollout(plan, {
      strategy: "all",
      canaryPercent: 10,
      stagedPhases: [10, 50, 100],
      version: "1.0.0",
      dryRun: false,
      requireCertify: false,
    });
    expect(full.success).toBe(true);
    expect(full.steps.every((s) => s.status === "deployed")).toBe(true);

    const canary = planRollout(plan, {
      strategy: "canary",
      canaryPercent: 50,
      stagedPhases: [10, 50, 100],
      version: "1.1.0",
      dryRun: true,
      requireCertify: false,
    });
    expect(canary.success).toBe(true);
    expect(canary.steps.some((s) => s.status === "pending")).toBe(true);
  });

  it("apply and rollback update deploy state", () => {
    const source = readFileSync(otaExample, "utf-8");
    const { program } = compileFile(otaExample, "typescript");
    const plan = buildDeployPlan(program, "ota_deployment.sd", "2.0.0");
    const state = emptyDeployState();
    const rollout = planRollout(plan, {
      strategy: "all",
      canaryPercent: 10,
      stagedPhases: [10, 50, 100],
      version: "2.0.0",
      dryRun: false,
      requireCertify: false,
    });
    applyRollout(state, rollout);
    expect(Object.keys(state.currentVersion)).toHaveLength(1);

    const rollbackPlan = buildDeployPlan(program, "ota_deployment.sd", "rollback");
    const rollback = rollbackTargets(state, rollbackPlan);
    expect(rollback.steps.some((s) => s.status === "skipped")).toBe(true);
    expect(source.length).toBeGreaterThan(0);
  });
});

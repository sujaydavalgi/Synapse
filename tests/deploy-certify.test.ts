import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { compileFile } from "../src/compile.js";
import {
  buildDeployPlan,
  planRollout,
  validateRolloutCertification,
} from "../src/deploy-service.js";
import {
  adapterVerifyOk,
  verifyAdapterPackage,
} from "../src/adapter-package-verify.js";

const otaExample = join(import.meta.dirname, "..", "examples/robotics/ota_deployment.sd");
const certifiedExample = join(
  import.meta.dirname,
  "..",
  "examples/robotics/certified_deployment.sd",
);
const nav2Package = join(
  import.meta.dirname,
  "..",
  "examples/packages/nav2_adapter_package",
);

describe("deploy certification gate (TS mirror)", () => {
  it("blocks uncertified rollout when requireCertify is set", () => {
    const { program } = compileFile(otaExample, "typescript");
    const plan = buildDeployPlan(program, "ota_deployment.sd", "1.0.0");
    const options = {
      strategy: "all" as const,
      canaryPercent: 10,
      stagedPhases: [10, 50, 100],
      version: "1.0.0",
      dryRun: false,
      requireCertify: true,
    };
    expect(validateRolloutCertification(plan, options)).toBeDefined();
    const result = planRollout(plan, options);
    expect(result.success).toBe(false);
    expect(result.steps).toHaveLength(0);
  });

  it("allows certified rollout when requireCertify is set", () => {
    const { program } = compileFile(certifiedExample, "typescript");
    const plan = buildDeployPlan(program, "certified_deployment.sd", "1.0.0");
    const options = {
      strategy: "all" as const,
      canaryPercent: 10,
      stagedPhases: [10, 50, 100],
      version: "1.0.0",
      dryRun: false,
      requireCertify: true,
    };
    expect(validateRolloutCertification(plan, options)).toBeUndefined();
    expect(plan.certificationProof?.passedStrict).toBe(true);
  });
});

describe("adapter package verify (TS mirror)", () => {
  it("passes nav2 adapter example package", () => {
    const issues = verifyAdapterPackage(nav2Package, "navigation.nav2");
    expect(adapterVerifyOk(issues)).toBe(true);
  });
});

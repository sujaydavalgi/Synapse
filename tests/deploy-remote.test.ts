import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type { AddressInfo } from "node:net";
import { compileFile } from "../src/compile.js";
import { buildDeployPlan, deployTargetKey } from "../src/deploy-service.js";
import {
  agentHealth,
  executeRemoteRollout,
  registerAgent,
  type DeployAgentRegistry,
} from "../src/deploy-remote.js";
import { createDeployAgentServer, type AgentState } from "../src/deploy-agent.js";

describe("deploy remote (TS mirror)", () => {
  let server: ReturnType<typeof createDeployAgentServer>;
  let port = 0;
  const target = deployTargetKey("RoverProgram", "JetsonOrin");
  const state: AgentState = { target, currentVersion: "0.0.0" };

  beforeAll(async () => {
    server = createDeployAgentServer(state);
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    port = (server.address() as AddressInfo).port;
  });

  afterAll(async () => {
    await new Promise<void>((resolve, reject) => {
      server.close((err) => (err ? reject(err) : resolve()));
    });
  });

  it("executes remote rollout against a deploy agent", async () => {
    const entry = { target, url: `http://127.0.0.1:${port}` };
    expect(await agentHealth(entry)).toBe(true);
    const { program } = compileFile("examples/robotics/ota_deployment.sd", "typescript");
    const plan = buildDeployPlan(program, "ota_deployment.sd", "2.0.0");
    let registry: DeployAgentRegistry = { agents: [] };
    registry = registerAgent(registry, target, entry.url);
    const result = await executeRemoteRollout(
      plan,
      {
        strategy: "all",
        canaryPercent: 10,
        stagedPhases: [100],
        version: "2.0.0",
        dryRun: false,
      },
      registry,
    );
    expect(result.success).toBe(true);
    expect(state.currentVersion).toBe("2.0.0");
  });
});

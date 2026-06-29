import { describe, expect, it } from "vitest";
import { SpandaClient } from "../src/client.js";
import { SpandaError } from "../src/errors.js";
import { ReadinessReport } from "../src/types.js";

describe("SpandaClient", () => {
  it("constructs local client", () => {
    const client = SpandaClient.local();
    expect(client.baseUrl).toContain("127.0.0.1");
  });

  it("maps permission errors", () => {
    const err = SpandaError.fromStatus(403, "forbidden");
    expect(err.name).toBe("PermissionError");
  });

  it("entityReadiness uses readiness path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      method,
      path,
    ) => {
      captured = `${method} ${path}`;
      return {};
    };
    await client.entityReadiness("rover-001");
    expect(captured).toBe("GET /v1/entities/rover-001/readiness");
  });

  it("entityRelationships uses relationships path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      method,
      path,
    ) => {
      captured = `${method} ${path}`;
      return {};
    };
    await client.entityRelationships("rover-001");
    expect(captured).toBe("GET /v1/entities/rover-001/relationships");
  });
});

describe("ReadinessReport", () => {
  it("extracts score from API envelope", () => {
    const report = ReadinessReport.fromApi({
      report: { score: { total: 88 }, status: "Ready" },
    });
    expect(report.score).toBe(88);
  });
});

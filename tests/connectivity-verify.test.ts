import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { verifyHardwareProgram } from "../src/hardware-verify.js";
import {
  applyGpsPositionFaults,
  faultToConnectivity,
  connectivityLinkToTransport,
  verifyRequiresConnectivity,
} from "../src/connectivity-positioning.js";
import { hardwareProfileFromDecl } from "../src/hardware-profile.js";

const examplesDir = join(import.meta.dirname, "..", "examples", "connectivity");

describe("connectivity verify (TS fallback)", () => {
  it("verifies connectivity requirements on RoverV2 with simulate faults", () => {
    const source = readFileSync(join(examplesDir, "connectivity_hardware_verify.sd"), "utf8");
    const program = parse(tokenize(source));
    const result = verifyHardwareProgram(program, { target: "RoverV2" });
    expect(result.target).toBe("RoverV2");
    expect(result.items.some((i) => i.category === "simulate")).toBe(true);
    expect(result.items.some((i) => i.category === "connectivity" && i.severity === "pass")).toBe(true);
    expect(result.items.some((i) => i.category === "sensors" && i.severity === "error")).toBe(true);
  });

  it("maps faults and link transports", () => {
    expect(faultToConnectivity("NetworkOutage")).toEqual({
      domain: "network",
      event: "disconnected",
    });
    expect(faultToConnectivity("GpsSpoofing")).toEqual({ domain: "gps", event: "spoofed" });
    expect(faultToConnectivity("GpsDrift")).toEqual({ domain: "gps", event: "drift" });
    expect(connectivityLinkToTransport("satellite")).toBe("websocket");
    expect(connectivityLinkToTransport("wifi")).toBe("mqtt");
    expect(connectivityLinkToTransport("cellular")).toBe("dds");
  });

  it("fails when required cellular missing from profile", () => {
    const program = parse(
      tokenize(`
requires_connectivity { cellular: required; }
hardware Tiny { connectivity [ WiFi6, GPS ]; }
robot R { actuator wheels: DifferentialDrive; }
deploy R to Tiny;
`),
    );
    const profile = hardwareProfileFromDecl(program.hardwareProfiles[0]!);
    const items = verifyRequiresConnectivity(program.requiresConnectivity!, profile);
    expect(items.some((i) => i.severity === "error")).toBe(true);
  });

  it("passes rover_deploy against RoverV1 when using builtins", () => {
    const source = readFileSync(
      join(import.meta.dirname, "..", "examples/hardware/rover_deploy.sd"),
      "utf8",
    );
    const program = parse(tokenize(source));
    const result = verifyHardwareProgram(program);
    expect(result.ok).toBe(true);
    expect(result.items.some((i) => i.category === "sensors" && i.severity === "pass")).toBe(true);
  });

  it("warns when deploy targets lack certification metadata", () => {
    const source = readFileSync(
      join(import.meta.dirname, "..", "examples/robotics/ota_deployment.sd"),
      "utf8",
    );
    const program = parse(tokenize(source));
    const result = verifyHardwareProgram(program);
    expect(
      result.items.some(
        (i) =>
          i.category === "certify" &&
          i.severity === "warning" &&
          i.message.includes("without certification metadata"),
      ),
    ).toBe(true);
  });

  it("reports framework adapter imports during verify", () => {
    const program = parse(
      tokenize(`
import navigation.nav2;
robot R { actuator wheels: DifferentialDrive; behavior run() { wheels.stop(); } }
`),
    );
    const result = verifyHardwareProgram(program);
    expect(result.items.some((i) => i.category === "adapter" && i.message.includes("spanda-nav2"))).toBe(
      true,
    );
  });

  it("verifies AI models and adapters in full_compat", () => {
    const source = readFileSync(
      join(import.meta.dirname, "..", "examples/hardware/full_compat.sd"),
      "utf8",
    );
    const program = parse(tokenize(source));
    const result = verifyHardwareProgram(program, { target: "RoverV1" });
    expect(result.items.some((i) => i.category === "ai" && i.severity === "pass")).toBe(true);
    expect(result.items.some((i) => i.category === "adapter" && i.severity === "pass")).toBe(true);
  });

  it("estimates topic bandwidth against deploy target", () => {
    const program = parse(
      tokenize(`
hardware RoverV1 {
  cpu: CortexA78;
  memory: 4 GB;
  network { bandwidth: 100 Mbps; latency: 20 ms; }
  sensors [ Lidar ];
  actuators [ DifferentialDrive ];
  timing { min_period: 10 ms; }
  resource: 15 W;
}
robot R {
  bus sim;
  topic scans: Scan { qos reliable; rate 20Hz; } on sim;
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
}
deploy R to RoverV1;
`),
    );
    const result = verifyHardwareProgram(program);
    expect(result.items.some((i) => i.message.includes("Topic 'scans'"))).toBe(true);
    expect(result.items.some((i) => i.message.includes("within target 100 Mbps"))).toBe(true);
  });

  it("warns when mission energy leaves low battery margin", () => {
    const program = parse(
      tokenize(`
hardware Tiny {
  battery { capacity: 100 Wh; }
  resource: 20 W;
  sensors [ Camera ];
  actuators [ DifferentialDrive ];
  timing { min_period: 10 ms; }
}
robot R {
  mission { duration: 5 h; }
  actuator wheels: DifferentialDrive;
}
deploy R to Tiny;
`),
    );
    const result = verifyHardwareProgram(program);
    expect(result.items.some((i) => i.category === "power" && i.severity === "warning")).toBe(true);
  });

  it("fails verify when SatelliteOutage removes required satellite link", () => {
    const program = parse(
      tokenize(`
requires_connectivity { satellite: required; }
hardware Remote { connectivity [ WiFi6, Satellite ]; sensors [ GPS ]; actuators [ DifferentialDrive ]; timing { min_period: 10 ms; } resource: 10 W; }
robot R { actuator wheels: DifferentialDrive; }
deploy R to Remote;
simulate_compatibility { fault SatelliteOutage; }
`),
    );
    const result = verifyHardwareProgram(program);
    expect(result.items.some((i) => i.category === "connectivity" && i.severity === "error")).toBe(true);
  });
});

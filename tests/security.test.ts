import { describe, it, expect } from "vitest";
import { compile } from "../src/compile.js";
import { sign, verifySignature, publicKeyFromMaterial, sha256Hex } from "../src/security/index.js";
import { readFileSync } from "node:fs";
import { join } from "node:path";

describe("security crypto", () => {
  it("signs and verifies with Ed25519", () => {
    const sig = sign("payload", "device-key-001");
    expect(sig.length).toBe(128);
    expect(verifySignature("payload", sig, "device-key-001")).toBe(true);
    expect(verifySignature("payload", sig, "wrong-key")).toBe(false);
  });

  it("verifies with derived public key", () => {
    const material = "device-key-001";
    const sig = sign("payload", material);
    const pubkey = publicKeyFromMaterial(material);
    expect(verifySignature("payload", sig, pubkey)).toBe(true);
  });

  it("sha256 hex is deterministic", () => {
    expect(sha256Hex("hello")).toBe(sha256Hex("hello"));
    expect(sha256Hex("hello").length).toBe(64);
  });
});

describe("security parser and type checker", () => {
  it("parses security example", () => {
    const source = readFileSync(join(import.meta.dirname, "../examples/std/security.sd"), "utf8");
    expect(() => compile(source)).not.toThrow();
  });

  it("parses identity audit trust permissions secrets", () => {
    const source = `
robot R {
  trust certified;
  permissions [ audit.write, identity.verify ];
  secret key from "dev-key";
  identity RobotIdentity { id: "r1"; public_key: "pk"; }
  audit A { record robot.pose; }
  topic t: Velocity publish on "/t" secure {
    signed = true;
    min_trust = trusted;
    requires = [ identity.verify ];
  };
  behavior run() {}
}
`;
    const result = compile(source);
    const robot = result.program.robots[0]!;
    expect(robot.trust?.level).toBe("certified");
    expect(robot.permissions?.capabilities).toContain("audit.write");
    expect(robot.secrets).toHaveLength(1);
    expect(robot.identity?.fields).toContainEqual(["id", "r1"]);
    expect(robot.audit?.records).toHaveLength(1);
    expect(robot.topics[0]?.secure?.signed).toBe(true);
  });

  it("rejects unknown trust level", () => {
    const source = `
robot R {
  trust bogus;
  behavior run() {}
}
`;
    expect(() => compile(source)).toThrow();
  });
});

describe("SecurityContext strict mode", () => {
  it("does not auto-grant when strict", async () => {
    const { SecurityContext } = await import("../src/security/index.js");
    const ctx = new SecurityContext();
    ctx.enableStrictPermissions();
    ctx.grantIfNotStrict("audit.write");
    expect(ctx.capabilities.has("audit.write")).toBe(false);
    ctx.capabilities.grant("audit.read");
    expect(() => ctx.requireOperation("audit.record")).toThrow(/capability denied/);
  });
});

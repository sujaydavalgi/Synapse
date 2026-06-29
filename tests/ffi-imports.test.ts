import { describe, it, expect } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { typeCheck } from "../src/types/index.js";
import { createFullCheckerHost } from "../src/cli/checker-host.js";
import { resolveFfiImport, ffiBridgeKind } from "../src/ffi/registry.js";

const fullCheckerHost = createFullCheckerHost();

describe("FFI bridge import registry", () => {
  it("resolves known python and cpp bridge paths", () => {
    expect(resolveFfiImport("python.torch")).toBe(true);
    expect(resolveFfiImport("python.opencv")).toBe(true);
    expect(resolveFfiImport("cpp.ros2")).toBe(true);
    expect(resolveFfiImport("cpp.pcl")).toBe(true);
    expect(ffiBridgeKind("python.torch")).toBe("python");
    expect(ffiBridgeKind("cpp.ros2")).toBe("cpp");
  });

  it("rejects invalid bridge paths", () => {
    expect(resolveFfiImport("java.awt")).toBe(false);
    expect(resolveFfiImport("python.")).toBe(false);
  });

  it("type-checks programs importing planned FFI bridges", () => {
    const source = `
import python.torch;
import cpp.ros2;

extern fn stub_add(a: Int, b: Int) -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
`;
    expect(() => typeCheck(parse(tokenize(source)), fullCheckerHost)).not.toThrow();
  });

  it("still rejects unknown imports", () => {
    const source = `
import unknown.vendor.lib;
robot R { actuator wheels: DifferentialDrive; behavior run() { wheels.stop(); } }
`;
    expect(() => typeCheck(parse(tokenize(source)), fullCheckerHost)).toThrow();
  });
});

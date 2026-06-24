import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { collectRecoveryDiagnostics } from "../src/recovery-diagnostics.js";
import { readinessDiagnostics } from "../src/readiness.js";

describe("recovery diagnostics", () => {
  it("warns when high-risk action lacks approval path", () => {
    const program = parse(
      tokenize(`
recovery_policy Risky {
  on gps.failed { resume mission; }
}
robot R {
  sensor gps: GPS;
  actuator w: DifferentialDrive;
  safety { max_speed = 1 m/s; }
  behavior b() {}
}
`),
    );
    const diags = collectRecoveryDiagnostics(program);
    expect(diags.some((d) => d.category === "recovery:approval")).toBe(true);
  });

  it("merges recovery diagnostics into readinessDiagnostics", () => {
    const source = `
recovery_policy Risky {
  on gps.failed { resume mission; }
}
robot R {
  sensor gps: GPS;
  actuator w: DifferentialDrive;
  safety { max_speed = 1 m/s; }
  behavior b() {}
}
`;
    const items = readinessDiagnostics(source);
    expect(items.some((d) => d.category === "recovery:approval")).toBe(true);
  });
});

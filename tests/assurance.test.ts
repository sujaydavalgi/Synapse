import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import {
  assureProgramTs,
  evaluateStateAssuranceTs,
  formatStateReport,
} from "../src/assurance.js";
import { lineColumnForIssue } from "../src/readiness-spans.js";
import { evaluateReadinessSource } from "../src/readiness.js";

function parseSource(source: string) {
  return parse(tokenize(source));
}

describe("mission assurance analysis", () => {
  it("evaluates state estimators into belief state", () => {
    const source = `
state_estimator RoverState {
    inputs [gps.fix, lidar.read];
    output StateEstimate;
}

robot R {
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
`;
    const program = parseSource(source);
    const report = evaluateStateAssuranceTs(program);
    expect(report.passed).toBe(true);
    expect(report.estimators).toHaveLength(1);
    expect(report.belief.estimates).toHaveLength(1);
    expect(formatStateReport(report)).toContain("RoverState");
  });

  it("flags empty state estimator inputs in assure summary", () => {
    const source = `
state_estimator EmptyState {
    inputs [];
    output StateEstimate;
}

robot R {
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
`;
    const program = parseSource(source);
    const summary = assureProgramTs(program, "test.sd");
    expect(summary.passed).toBe(false);
    expect(summary.state.issues.some((i) => i.includes("EmptyState"))).toBe(true);
  });

  it("surfaces state estimator gaps in readiness with spans", () => {
    const source = `
state_estimator EmptyState {
    inputs [];
    output StateEstimate;
}

robot R {
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
`;
    const report = evaluateReadinessSource(source);
    const issue = report.issues.find((i) => i.message.includes("State estimator"));
    expect(issue).toBeDefined();
    const program = parseSource(source);
    const coords = lineColumnForIssue(program, issue!);
    expect(coords.line).toBeGreaterThan(1);
  });
});

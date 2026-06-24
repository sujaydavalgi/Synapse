import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import {
  evaluateContinuityTs,
  planSuccessionTs,
  planTakeoverTs,
  type ContinuityContext,
} from "../src/mission-continuity.js";
import { loadCheckpoint, recordCheckpoint } from "../src/continuity-checkpoint.js";

const warehouseSource = `
hardware RoverV1 {
    sensors [GPS, Camera, Lidar];
    actuators [DifferentialDrive];
}

mission_plan WarehouseInventoryScan {
    step navigate_to_aisle;
    step scan_shelf_a;
}

fleet WarehouseFleet {
    ScannerAlpha;
    ScannerBeta;
}

continuity_policy WarehouseContinuity {
    on robot.failed {
        resume from checkpoint;
        reassign mission;
    }
}

robot ScannerAlpha {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior scan() { }
}

robot ScannerBeta {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior scan() { }
}
`;

function warehouseProgram() {
  return parse(tokenize(warehouseSource));
}

describe("mission continuity TS mirror", () => {
  it("evaluates continuity with successor and checkpoint", () => {
    const program = warehouseProgram();
    const context: ContinuityContext = {
      mission: "WarehouseInventoryScan",
      failed_entity: "ScannerAlpha",
      trigger: "robot_failed",
      progress_percent: 72,
      scope: "fleet",
    };
    const report = evaluateContinuityTs(program, context);
    expect(report.passed).toBe(true);
    expect(report.selected_successor).toBe("ScannerBeta");
    expect(report.checkpoint?.progress_percent).toBe(72);
  });

  it("plans takeover for failed robot", () => {
    const program = warehouseProgram();
    const report = planTakeoverTs(program, {
      mission: "WarehouseInventoryScan",
      failed_entity: "ScannerAlpha",
      trigger: "robot_failed",
      progress_percent: 72,
      scope: "fleet",
    });
    expect(report.succeeded).toBe(true);
    expect(report.successor).toBe("ScannerBeta");
  });

  it("ranks fleet successors", () => {
    const program = warehouseProgram();
    const report = planSuccessionTs(program, {
      mission: "WarehouseInventoryScan",
      failed_entity: "ScannerAlpha",
      trigger: "robot_failed",
      progress_percent: 72,
      scope: "fleet",
    });
    expect(report.rankings.length).toBeGreaterThan(0);
    expect(report.selected).toBeTruthy();
  });

  it("persists checkpoints in TS store mirror", () => {
    const store = recordCheckpoint(
      { entries: {} },
      "WarehouseInventoryScan",
      "ScannerAlpha",
      {
        mission: "WarehouseInventoryScan",
        completed_steps: ["navigate_to_aisle"],
        current_goal: "scan_shelf_a",
        progress_percent: 72,
        checkpoints: [],
      },
    );
    const loaded = loadCheckpoint(store, "WarehouseInventoryScan", "ScannerAlpha");
    expect(loaded?.progress_percent).toBe(72);
  });
});

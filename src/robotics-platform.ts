/**
 * Robotics platform primitives: mission lifecycle, fleet grouping, and program safety zones.
 * @module
 */

export type MissionState = "Pending" | "Running" | "Paused" | "Completed" | "Failed";

export type MissionRuntime = {
  name: string | null;
  steps: string[];
  state: MissionState;
  stepIndex: number;
  durationHours: number | null;
};

export function createMissionRuntime(
  name: string | null,
  steps: string[],
  durationHours: number | null,
): MissionRuntime {
  // Build a mission controller starting in the pending state.
  //
  // Parameters:
  // - `name` — optional mission label
  // - `steps` — ordered step names
  // - `durationHours` — optional duration budget in hours
  //
  // Returns:
  // Fresh mission runtime in the Pending state.
  //
  // Options:
  // None.
  //
  // Example:
  // const mission = createMissionRuntime("Delivery", ["navigate"], 0.5);

  return {
    name,
    steps,
    state: "Pending",
    stepIndex: 0,
    durationHours,
  };
}

export function missionStart(runtime: MissionRuntime): void {
  // Transition a pending mission into the running state.
  //
  // Parameters:
  // - `runtime` — mission controller to update
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:
  // missionStart(mission);

  if (runtime.state === "Pending") {
    runtime.state = "Running";
  }
}

export function missionPause(runtime: MissionRuntime): void {
  // Pause an active mission without losing step progress.
  if (runtime.state === "Running") {
    runtime.state = "Paused";
  }
}

export function missionResume(runtime: MissionRuntime): void {
  // Resume a paused mission from the current step.
  if (runtime.state === "Paused") {
    runtime.state = "Running";
  }
}

export function missionAdvance(runtime: MissionRuntime): string {
  // Move to the next mission step and return its name when one remains.
  if (runtime.state !== "Running") {
    return "";
  }
  if (runtime.stepIndex >= runtime.steps.length) {
    runtime.state = "Completed";
    return "";
  }
  const step = runtime.steps[runtime.stepIndex] ?? "";
  runtime.stepIndex += 1;
  if (runtime.stepIndex >= runtime.steps.length) {
    runtime.state = "Completed";
  }
  return step;
}

export function missionComplete(runtime: MissionRuntime): void {
  // Mark the mission completed regardless of remaining steps.
  runtime.state = "Completed";
  runtime.stepIndex = runtime.steps.length;
}

export function missionFail(runtime: MissionRuntime): void {
  // Mark the mission failed and stop step progression.
  runtime.state = "Failed";
}

export function missionCurrentStep(runtime: MissionRuntime): string {
  // Return the active step name while the mission is running.
  if (runtime.state !== "Running") {
    return "";
  }
  return runtime.steps[runtime.stepIndex] ?? "";
}

export class FleetRegistry {
  private fleets = new Map<string, string[]>();

  register(name: string, members: string[]): void {
    // Store a fleet name and its member robot identifiers.
    this.fleets.set(name, members);
  }

  members(name: string): string[] | undefined {
    // Look up fleet members by fleet name.
    return this.fleets.get(name);
  }

  names(): string[] {
    // Return all declared fleet names.
    return [...this.fleets.keys()];
  }

  clone(): FleetRegistry {
    // Clone fleet registrations for env re-binding after robot setup.
    const copy = new FleetRegistry();
    for (const [name, members] of this.fleets) {
      copy.register(name, [...members]);
    }
    return copy;
  }
}

export class ProgramSafetyZoneRegistry {
  private zones = new Map<string, number>();

  register(name: string, maxSpeedMps: number): void {
    // Register a zone-specific maximum speed in meters per second.
    this.zones.set(name, maxSpeedMps);
  }

  maxSpeedFor(zoneName: string): number | undefined {
    // Resolve the configured speed cap for a named zone.
    return this.zones.get(zoneName);
  }

  speedCaps(): Map<string, number> {
    // Return all registered zone speed caps.
    return new Map(this.zones);
  }
}

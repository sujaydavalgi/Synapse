/**
 * index module (simulator/index.ts).
 * @module
 */

import type {
  MotionCommand,
  PoseValue,
  RobotBackend,
  RobotState,
  RuntimeValue,
} from "../runtime/interpreter.js";
import { createSimHal, type HalBackend } from "../hal/index.js";

export type Obstacle = { x: number; y: number; radius: number };

export type SimulatorConfig = {
  obstacles?: Obstacle[];
  initialPose?: { x: number; y: number; theta: number; z?: number };
  lidarRange?: number;
  simulationSteps?: number;
};

type PublishedMessage = { topic: string; messageType: string; value: RuntimeValue };

export class Simulator implements RobotBackend {
  private pose: { x: number; y: number; theta: number; z: number };
  private velocity = { linear: 0, angular: 0 };
  private emergencyStop = false;
  private obstacles: Obstacle[];
  private lidarRange: number;
  private armPosition = { x: 0, y: 0, z: 0.5 };
  private gripperClosed = false;
  private thrust = 0;
  private eventLog: string[] = [];
  private published: PublishedMessage[] = [];
  private followQueue: PoseValue[] = [];
  private serviceLog: string[] = [];
  private actionLog: string[] = [];
  private hal: HalBackend = createSimHal();

  constructor(config: SimulatorConfig = {}) {
    this.pose = {
      x: config.initialPose?.x ?? 0,
      y: config.initialPose?.y ?? 0,
      theta: config.initialPose?.theta ?? 0,
      z: config.initialPose?.z ?? 0,
    };
    this.obstacles = config.obstacles ?? [
      { x: 2, y: 0, radius: 0.3 },
      { x: -1, y: 1.5, radius: 0.25 },
    ];
    this.lidarRange = config.lidarRange ?? 10;
  }

  readSensor(_sensorName: string, sensorType: string, _topic?: string | null): RuntimeValue {
    // ReadSensor.
    //
    // Parameters:
    // - `_sensorName` — input value
    // - `sensorType` — input value
    // - `_topic?` — optional input
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // - `_topic?` — optional parameter
    //
    // Example:

    // const result = readSensor(_sensorName, sensorType, _topic?);

    switch (sensorType) {
      case "Lidar":
        return { kind: "scan", nearestDistance: this.simulateLidar() };
      case "IMU":
        return {
          kind: "object",
          typeName: "IMUReading",
          fields: {
            roll: { kind: "number", value: 0, unit: "rad" },
            pitch: { kind: "number", value: 0, unit: "rad" },
            yaw: { kind: "number", value: this.pose.theta, unit: "rad" },
          },
        };
      case "AltitudeSensor":
        return { kind: "number", value: this.pose.z, unit: "m" };
      case "GPS":
      case "GNSS":
        return {
          kind: "object",
          typeName: "GpsFix",
          fields: {
            lat: { kind: "number", value: this.pose.x, unit: "none" },
            lon: { kind: "number", value: this.pose.y, unit: "none" },
            altitude: { kind: "number", value: this.pose.z, unit: "m" },
            fix_quality: { kind: "number", value: 1.0, unit: "none" },
          },
        };
      case "ForceTorque":
        return {
          kind: "object",
          typeName: "ForceTorqueReading",
          fields: {
            force: { kind: "number", value: this.gripperClosed ? 5.0 : 0, unit: "none" },
          },
        };
      case "Camera":
        return {
          kind: "object",
          typeName: "CameraFrame",
          fields: {
            width: { kind: "number", value: 640, unit: "none" },
            height: { kind: "number", value: 480, unit: "none" },
          },
        };
      default:
        return { kind: "void" };
    }
  }

  publishTopic(topicPath: string, messageType: string, value: RuntimeValue): void {
    // PublishTopic.
    //
    // Parameters:
    // - `topicPath` — input value
    // - `messageType` — input value
    // - `value` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = publishTopic(topicPath, messageType, value);

    this.published.push({ topic: topicPath, messageType, value });
    if (value.kind === "velocity") {
      this.velocity = { linear: value.linear, angular: value.angular };
    }
    this.eventLog.push(`publish(${topicPath}, ${messageType})`);
  }

  callService(serviceName: string, serviceType: string): RuntimeValue {
    // CallService.
    //
    // Parameters:
    // - `serviceName` — input value
    // - `serviceType` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = callService(serviceName, serviceType);

    this.serviceLog.push(`${serviceName}:${serviceType}`);
    this.eventLog.push(`service(${serviceName})`);
    return { kind: "bool", value: true };
  }

  sendAction(actionName: string, actionType: string, goal: RuntimeValue): RuntimeValue {
    // SendAction.
    //
    // Parameters:
    // - `actionName` — input value
    // - `actionType` — input value
    // - `goal` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = sendAction(actionName, actionType, goal);

    this.actionLog.push(`${actionName}:${actionType}`);
    this.eventLog.push(`action(${actionName})`);
    if (goal.kind === "pose") {
      this.pose = { x: goal.x, y: goal.y, theta: goal.theta, z: goal.z };
    }
    if (goal.kind === "trajectory" && goal.waypoints.length > 0) {
      this.followQueue = [...goal.waypoints];
    }
    return { kind: "bool", value: true };
  }

  getPublishedTopics(): PublishedMessage[] {
    // GetPublishedTopics.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // PublishedMessage[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = getPublishedTopics();

    return [...this.published];
  }

  executeMotion(cmd: MotionCommand): void {
    // ExecuteMotion.
    //
    // Parameters:
    // - `cmd` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeMotion(cmd);

    if (this.emergencyStop && cmd.kind !== "stop") {
      this.velocity = { linear: 0, angular: 0 };
      return;
    }

    switch (cmd.kind) {
      case "drive":
        this.velocity = { linear: cmd.linear, angular: cmd.angular };
        this.eventLog.push(`drive(${cmd.linear.toFixed(2)} m/s, ${cmd.angular.toFixed(2)} rad/s)`);
        break;
      case "follow":
        this.followQueue = [...cmd.waypoints];
        this.eventLog.push(`follow(${cmd.waypoints.length} waypoints)`);
        break;
      case "stop":
        this.velocity = { linear: 0, angular: 0 };
        this.followQueue = [];
        this.eventLog.push("stop()");
        break;
      case "move_to":
        this.armPosition = { x: cmd.x, y: cmd.y, z: cmd.z };
        this.eventLog.push(`move_to(${cmd.x}, ${cmd.y}, ${cmd.z})`);
        break;
      case "grip":
        this.gripperClosed = true;
        this.eventLog.push("grip()");
        break;
      case "release":
        this.gripperClosed = false;
        this.eventLog.push("release()");
        break;
      case "open":
        this.gripperClosed = false;
        this.eventLog.push("open()");
        break;
      case "set_thrust":
        this.thrust = cmd.thrust;
        this.eventLog.push(`set_thrust(${cmd.thrust})`);
        break;
      case "hover":
        this.thrust = 0.5;
        this.velocity = { linear: 0, angular: 0 };
        this.eventLog.push("hover()");
        break;
    }
  }

  tick(dtMs: number): void {
    // Tick.
    //
    // Parameters:
    // - `dtMs` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = tick(dtMs);

    if (this.emergencyStop) {
      this.velocity = { linear: 0, angular: 0 };
      return;
    }

    const dt = dtMs / 1000;

    if (this.followQueue.length > 0) {
      const target = this.followQueue[0];
      const dx = target.x - this.pose.x;
      const dy = target.y - this.pose.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < 0.05) {
        this.followQueue.shift();
        this.pose = { ...this.pose, x: target.x, y: target.y, theta: target.theta };
      } else {
        const speed = 0.5;
        this.pose.x += (dx / dist) * speed * dt;
        this.pose.y += (dy / dist) * speed * dt;
        this.pose.theta = Math.atan2(dy, dx);
        this.velocity = { linear: speed, angular: 0 };
      }
      return;
    }

    if (this.thrust > 0) {
      const climbRate = (this.thrust - 0.5) * 2;
      this.pose.z = Math.max(0, this.pose.z + climbRate * dt);
    }

    const newTheta = this.pose.theta + this.velocity.angular * dt;
    const newX = this.pose.x + this.velocity.linear * Math.cos(this.pose.theta) * dt;
    const newY = this.pose.y + this.velocity.linear * Math.sin(this.pose.theta) * dt;

    this.pose = { ...this.pose, x: newX, y: newY, theta: newTheta };
  }

  getState(): RobotState {
    // GetState.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // RobotState.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = getState();

    return {
      pose: { ...this.pose },
      velocity: { ...this.velocity },
      emergencyStop: this.emergencyStop,
    };
  }

  setEmergencyStop(value: boolean): void {
    // SetEmergencyStop.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = setEmergencyStop(value);

    this.emergencyStop = value;
    if (value) {
      this.velocity = { linear: 0, angular: 0 };
      this.followQueue = [];
    }
  }

  getHal(): HalBackend {
    // GetHal.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HalBackend.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = getHal();

    return this.hal;
  }

  getEventLog(): string[] {
    // GetEventLog.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // string[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = getEventLog();

    return [...this.eventLog];
  }

  getArmPosition(): {
    // GetArmPosition.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // .
    //
    // Options:
    // None.
    //
    // Example:

 // const result = getArmPosition();
 x: number; y: number; z: number } {
    return { ...this.armPosition };
  }

  getServiceLog(): string[] {
    return [...this.serviceLog];
  }

  getActionLog(): string[] {
    return [...this.actionLog];
  }

  private simulateLidar(): number {    // Compute nearest for the following logic.
    let nearest = this.lidarRange;

    // Process each obstacle.
    for (const obs of this.obstacles) {
      const dx = obs.x - this.pose.x;
      const dy = obs.y - this.pose.y;
      const dist = Math.sqrt(dx * dx + dy * dy) - obs.radius;

      // continue when dist > 0 && dist < nearest.
      if (dist > 0 && dist < nearest) {
        nearest = dist;
      }
    }
    const wallDist = 5 - Math.abs(this.pose.x);

    // continue when wallDist > 0 && wallDist < nearest.
    if (wallDist > 0 && wallDist < nearest) nearest = wallDist;
    return Math.max(0.01, nearest);
}
}

export function createDefaultSimulator(config?: SimulatorConfig): Simulator {
  // CreateDefaultSimulator.
  //
  // Parameters:
  // - `config?` — optional input
  //
  // Returns:
  // `Simulator`.
  //
  // Options:
  // - `config?` — optional parameter
  //
  // Example:

  // const result = createDefaultSimulator(config?);
  return new Simulator(config);
}

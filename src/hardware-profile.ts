/**
 * Hardware profile registry and simulation fault application for TS verify.
 * @module
 */

import type { Program } from "./ast/nodes.js";
import type { HardwareDecl } from "./foundations.js";

export type HardwareProfile = {
  name: string;
  cpu: string | null;
  memoryMb: number | null;
  storageMb: number | null;
  gpuTops: number | null;
  gpuRequired: boolean;
  sensors: string[];
  actuators: string[];
  connectivity: string[];
  batteryWh: number | null;
  networkBandwidthMbps: number | null;
  networkLatencyMs: number | null;
  packetLossPct: number | null;
  minControlPeriodMs: number;
  powerDrawW: number;
};

function profile(
  name: string,
  cpu: string,
  memoryMb: number,
  storageMb: number,
  gpuTops: number | null,
  gpuRequired: boolean,
  sensors: string[],
  actuators: string[],
  connectivity: string[],
  batteryWh: number | null,
  networkBandwidthMbps: number | null,
  networkLatencyMs: number | null,
  minControlPeriodMs: number,
  powerDrawW: number,
): HardwareProfile {
  return {
    name,
    cpu,
    memoryMb,
    storageMb,
    gpuTops,
    gpuRequired,
    sensors,
    actuators,
    connectivity,
    batteryWh,
    networkBandwidthMbps,
    networkLatencyMs,
    packetLossPct: null,
    minControlPeriodMs,
    powerDrawW,
  };
}

export function builtinProfiles(): Map<string, HardwareProfile> {
  // Built-in hardware profiles mirrored from Rust `hardware::builtin_profiles`.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // Map of profile name to profile definition.
  //
  // Options:
  // None.
  //
  // Example:
  // const profiles = builtinProfiles();

  return new Map([
    [
      "RoverV1",
      profile(
        "RoverV1",
        "CortexA78",
        4096,
        32768,
        null,
        false,
        ["Camera", "Lidar", "IMU"],
        ["DifferentialDrive"],
        ["WiFi", "Ethernet"],
        100,
        50,
        20,
        10,
        15,
      ),
    ],
    [
      "RoverV2",
      profile(
        "RoverV2",
        "CortexA78",
        8192,
        65536,
        1,
        false,
        ["Camera", "Lidar", "IMU", "GPS"],
        ["DifferentialDrive", "RoboticArm"],
        ["WiFi6", "Bluetooth5", "LTE", "GPS"],
        150,
        100,
        15,
        8,
        20,
      ),
    ],
    [
      "JetsonOrin",
      profile(
        "JetsonOrin",
        "CortexA78AE",
        8192,
        131072,
        275,
        true,
        ["Camera", "Lidar", "IMU"],
        ["DifferentialDrive"],
        ["Ethernet", "WiFi6", "FiveG"],
        null,
        1000,
        5,
        5,
        25,
      ),
    ],
    [
      "RaspberryPi5",
      profile(
        "RaspberryPi5",
        "CortexA76",
        8192,
        65536,
        null,
        false,
        ["Camera", "IMU"],
        ["DifferentialDrive"],
        ["WiFi", "Bluetooth", "Ethernet"],
        null,
        100,
        30,
        15,
        8,
      ),
    ],
    [
      "ESP32",
      profile(
        "ESP32",
        "Xtensa",
        4,
        4,
        null,
        false,
        ["IMU"],
        ["DifferentialDrive"],
        ["WiFi", "BLE"],
        5,
        10,
        100,
        50,
        2,
      ),
    ],
  ]);
}

export function hardwareProfileFromDecl(decl: HardwareDecl): HardwareProfile {
  // Convert a parsed hardware block into a verification profile.
  //
  // Parameters:
  // - `decl` — AST hardware declaration
  //
  // Returns:
  // Normalized hardware profile.
  //
  // Options:
  // None.
  //
  // Example:
  // const profile = hardwareProfileFromDecl(decl);

  return {
    name: decl.name,
    cpu: decl.cpu,
    memoryMb: decl.memoryMb,
    storageMb: decl.storageMb,
    gpuTops: decl.gpuTops,
    gpuRequired: decl.gpuRequired,
    sensors: [...decl.sensors],
    actuators: [...decl.actuators],
    connectivity: [...(decl.connectivity ?? [])],
    batteryWh: decl.batteryWh,
    networkBandwidthMbps: decl.networkBandwidthMbps,
    networkLatencyMs: decl.networkLatencyMs,
    packetLossPct: null,
    minControlPeriodMs: decl.minControlPeriodMs ?? 20,
    powerDrawW: decl.powerDrawW ?? 10,
  };
}

export function buildProfileRegistry(program: Program): Map<string, HardwareProfile> {
  // Merge built-in and program-declared hardware profiles.
  //
  // Parameters:
  // - `program` — parsed program
  //
  // Returns:
  // Profile registry keyed by name.
  //
  // Options:
  // None.
  //
  // Example:
  // const registry = buildProfileRegistry(program);

  const registry = builtinProfiles();
  for (const decl of program.hardwareProfiles) {
    registry.set(decl.name, hardwareProfileFromDecl(decl));
  }
  return registry;
}

export function applyFault(profile: HardwareProfile, faultType: string): HardwareProfile {
  // Apply a simulate_compatibility fault to a hardware profile copy.
  //
  // Parameters:
  // - `profile` — base hardware profile
  // - `faultType` — fault identifier
  //
  // Returns:
  // Profile after fault effects.
  //
  // Options:
  // None.
  //
  // Example:
  // const degraded = applyFault(profile, "WeakWifi");

  const next = { ...profile, sensors: [...profile.sensors], actuators: [...profile.actuators], connectivity: [...profile.connectivity] };
  switch (faultType) {
    case "CameraFailure":
      next.sensors = next.sensors.filter((s) => s !== "Camera");
      break;
    case "LidarFailure":
      next.sensors = next.sensors.filter((s) => s !== "Lidar");
      break;
    case "BatteryDegradation":
      if (next.batteryWh != null) next.batteryWh *= 0.5;
      break;
    case "NetworkOutage":
      next.networkBandwidthMbps = 0;
      next.networkLatencyMs = 10_000;
      break;
    case "ImuFailure":
      next.sensors = next.sensors.filter((s) => s !== "IMU");
      break;
    case "GpsFailure":
    case "GPSLost":
      next.sensors = next.sensors.filter((s) => s !== "GPS" && s !== "GNSS");
      next.connectivity = next.connectivity.filter((c) => c !== "GPS" && c !== "GNSS");
      break;
    case "GpsDrift":
    case "GpsSpoofing":
      break;
    case "WeakWifi":
      next.networkBandwidthMbps = 1;
      next.networkLatencyMs = 500;
      break;
    case "LteOutage":
      next.networkBandwidthMbps = 0;
      next.networkLatencyMs = 10_000;
      next.connectivity = next.connectivity.filter(
        (c) => !["LTE", "FourG", "4G", "FiveG", "5G"].includes(c),
      );
      break;
    case "SatelliteOutage":
      next.networkBandwidthMbps = 0;
      next.networkLatencyMs = 10_000;
      next.connectivity = next.connectivity.filter((c) => c !== "Satellite");
      break;
    case "NetworkLatencySpike":
    case "LatencySpike":
      next.networkLatencyMs = 2000;
      break;
    case "FiveGHandoff":
      next.networkLatencyMs = 150;
      break;
    case "BluetoothDisconnect":
      next.connectivity = next.connectivity.filter(
        (c) => !["Bluetooth", "Bluetooth5", "BLE"].includes(c),
      );
      break;
    case "PacketLoss":
      next.packetLossPct = 10;
      break;
    default:
      break;
  }
  return next;
}

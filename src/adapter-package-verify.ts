/**
 * Validate package `[adapter]` sections against framework registry metadata.
 * @module
 */

import { readFileSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";

export type AdapterMetadata = {
  provides: string[];
  requires: string[];
};

export type AdapterVerifySeverity = "pass" | "warning" | "error";

export type AdapterVerifyIssue = {
  severity: AdapterVerifySeverity;
  message: string;
};

const NAV2_METADATA: AdapterMetadata = {
  provides: ["Nav2Adapter", "NavigationGoal", "CostMap", "navigate"],
  requires: ["topic.publish", "ros2.bridge", "actuator.drive"],
};

const CARTOGRAPHER_METADATA: AdapterMetadata = {
  provides: ["CartographerSlam", "OccupancyGrid", "PoseGraph", "slam.localize", "slam.map"],
  requires: ["topic.publish", "sensor.read", "lidar.read"],
};

const RTABMAP_METADATA: AdapterMetadata = {
  provides: ["RtabmapSlam", "LoopClosure", "VisualOdometry", "slam.localize", "slam.map"],
  requires: ["topic.publish", "sensor.read", "camera.read"],
};

const SLAM_METADATA: AdapterMetadata = {
  provides: ["SlamAdapter", "slam.localize", "slam.map"],
  requires: ["topic.publish", "sensor.read"],
};

export function adapterMetadataForImport(importPath: string): AdapterMetadata | undefined {
  // Resolve expected adapter metadata for a framework import path.
  switch (importPath) {
    case "navigation.nav2":
      return NAV2_METADATA;
    case "navigation.cartographer":
      return CARTOGRAPHER_METADATA;
    case "navigation.rtabmap":
      return RTABMAP_METADATA;
    case "navigation.slam":
      return SLAM_METADATA;
    default:
      return undefined;
  }
}

export function adapterMetadataForPackage(packageName: string): AdapterMetadata | undefined {
  // Resolve expected adapter metadata for a registry package name.
  switch (packageName) {
    case "spanda-nav2":
      return NAV2_METADATA;
    case "spanda-cartographer":
      return CARTOGRAPHER_METADATA;
    case "spanda-rtabmap":
      return RTABMAP_METADATA;
    case "spanda-slam":
      return SLAM_METADATA;
    default:
      return undefined;
  }
}

function missingProvides(expected: AdapterMetadata, actual: AdapterMetadata): string[] {
  return expected.provides.filter((symbol) => !actual.provides.includes(symbol));
}

function missingRequires(expected: AdapterMetadata, actual: AdapterMetadata): string[] {
  return expected.requires.filter((symbol) => !actual.requires.includes(symbol));
}

export function verifyManifestAdapter(
  packageName: string,
  actual: AdapterMetadata,
  expected: AdapterMetadata,
): AdapterVerifyIssue[] {
  // Check declared provides/requires against registry adapter metadata.
  const issues: AdapterVerifyIssue[] = [];
  if (actual.provides.length === 0 && actual.requires.length === 0) {
    issues.push({
      severity: "error",
      message: `Package '${packageName}' missing [adapter] provides/requires for production adapter scaffolding`,
    });
    return issues;
  }

  const missingProvideSymbols = missingProvides(expected, actual);
  if (missingProvideSymbols.length === 0) {
    issues.push({
      severity: "pass",
      message: `Package '${packageName}' adapter provides cover expected symbols`,
    });
  } else {
    issues.push({
      severity: "error",
      message: `Package '${packageName}' adapter missing provides: ${missingProvideSymbols.join(", ")}`,
    });
  }

  const missingRequireSymbols = missingRequires(expected, actual);
  if (missingRequireSymbols.length === 0) {
    issues.push({
      severity: "pass",
      message: `Package '${packageName}' adapter requires cover expected runtime capabilities`,
    });
  } else {
    issues.push({
      severity: "warning",
      message: `Package '${packageName}' adapter missing recommended requires: ${missingRequireSymbols.join(", ")}`,
    });
  }

  return issues;
}

export function adapterVerifyOk(issues: AdapterVerifyIssue[]): boolean {
  return !issues.some((issue) => issue.severity === "error");
}

export type AdapterManifestSection = {
  packageName: string;
  adapter: AdapterMetadata;
};

function parseTomlStringArray(line: string): string[] {
  const match = line.match(/=\s*\[(.*)\]/);
  if (!match?.[1]) return [];
  return match[1]
    .split(",")
    .map((part) => part.trim().replace(/^"|"$/g, ""))
    .filter(Boolean);
}

export function readAdapterManifestSection(projectRoot: string): AdapterManifestSection {
  // Read package name and adapter provides/requires from spanda.toml.
  const manifestPath = join(resolve(projectRoot), "spanda.toml");
  if (!existsSync(manifestPath)) {
    throw new Error(`Missing spanda.toml in ${projectRoot}`);
  }
  const text = readFileSync(manifestPath, "utf-8");
  let packageName = "unknown";
  let inAdapter = false;
  const adapter: AdapterMetadata = { provides: [], requires: [] };
  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();
    if (line.startsWith("[package]")) {
      inAdapter = false;
      continue;
    }
    if (line.startsWith("[adapter]")) {
      inAdapter = true;
      continue;
    }
    if (line.startsWith("[") && !line.startsWith("[adapter]")) {
      inAdapter = false;
      continue;
    }
    if (!inAdapter && line.startsWith("name")) {
      const nameMatch = line.match(/=\s*"([^"]+)"/);
      if (nameMatch?.[1]) packageName = nameMatch[1];
      continue;
    }
    if (inAdapter && line.startsWith("provides")) {
      adapter.provides = parseTomlStringArray(line);
    }
    if (inAdapter && line.startsWith("requires")) {
      adapter.requires = parseTomlStringArray(line);
    }
  }
  return { packageName, adapter };
}

export function verifyAdapterPackage(
  projectRoot: string,
  importPath?: string,
  packageName?: string,
): AdapterVerifyIssue[] {
  // Resolve expected adapter metadata and validate the manifest adapter section.
  const expected =
    (importPath ? adapterMetadataForImport(importPath) : undefined) ??
    (packageName ? adapterMetadataForPackage(packageName) : undefined);
  if (!expected) {
    throw new Error("No adapter metadata registered for requested import/package");
  }
  const manifest = readAdapterManifestSection(projectRoot);
  return verifyManifestAdapter(manifest.packageName, manifest.adapter, expected);
}

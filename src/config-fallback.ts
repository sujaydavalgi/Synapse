/**
 * TypeScript fallback for config commands when the native CLI is unavailable.
 * @module
 */

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isCliAvailable, runNativeCli } from "./rust-bridge.js";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const MANIFEST = "spanda.toml";

type ConfigCommandResult = {
  exitCode: number;
  stdout: string;
  stderr: string;
};

function buildCliArgs(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
): string[] {
  const args = [command, ...positional];
  for (const [key, value] of flags) {
    if (value === true) {
      args.push(`--${key}`);
    } else if (typeof value === "string") {
      args.push(`--${key}`, value);
    }
  }
  return args;
}

function runCargoCli(args: string[]): ConfigCommandResult {
  const result = spawnSync("cargo", ["run", "-q", "-p", "spanda", "--", ...args], {
    cwd: repoRoot,
    encoding: "utf-8",
  });
  return {
    exitCode: result.status ?? 1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function findProjectRoot(start: string): string | null {
  let dir = resolve(start);
  for (let depth = 0; depth < 32; depth += 1) {
    if (existsSync(join(dir, MANIFEST))) {
      return dir;
    }
    const parent = dirname(dir);
    if (parent === dir) {
      break;
    }
    dir = parent;
  }
  return null;
}

function parseTomlSections(content: string): Record<string, Record<string, string>> {
  const sections: Record<string, Record<string, string>> = {};
  let current = "project";
  sections[current] = {};
  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    const section = line.match(/^\[([^\]]+)\]$/);
    if (section) {
      current = section[1] ?? "project";
      sections[current] = sections[current] ?? {};
      continue;
    }
    const kv = line.match(/^([A-Za-z0-9_.-]+)\s*=\s*"([^"]*)"/);
    if (kv) {
      sections[current] = sections[current] ?? {};
      sections[current][kv[1] ?? ""] = kv[2] ?? "";
    }
  }
  return sections;
}

function fallbackValidate(projectRoot: string, asJson: boolean): ConfigCommandResult {
  const manifestPath = join(projectRoot, MANIFEST);
  const content = readFileSync(manifestPath, "utf-8");
  const sections = parseTomlSections(content);
  const configSection = sections.config ?? {};
  const extendsSection = sections.extends ?? {};
  const missing: string[] = [];
  for (const value of Object.values(configSection)) {
    const candidate = join(projectRoot, value);
    if (!existsSync(candidate)) {
      missing.push(value);
    }
  }
  for (const value of Object.values(extendsSection)) {
    const candidate = join(projectRoot, value);
    if (!existsSync(candidate)) {
      missing.push(value);
    }
  }
  const ok = missing.length === 0;
  if (asJson) {
    return {
      exitCode: ok ? 0 : 1,
      stdout: `${JSON.stringify({ ok, project_root: projectRoot, missing }, null, 2)}\n`,
      stderr: "",
    };
  }
  if (ok) {
    return {
      exitCode: 0,
      stdout: `Configuration valid (fallback): ${projectRoot}\n`,
      stderr: "",
    };
  }
  return {
    exitCode: 1,
    stdout: "",
    stderr: `Missing configuration files:\n${missing.map((m) => `  - ${m}`).join("\n")}\n`,
  };
}

function fallbackResolve(projectRoot: string, asJson: boolean): ConfigCommandResult {
  const manifestPath = join(projectRoot, MANIFEST);
  const content = readFileSync(manifestPath, "utf-8");
  const sections = parseTomlSections(content);
  const payload = {
    project_root: projectRoot,
    manifest: sections.project ?? {},
    config_fragments: sections.config ?? {},
    extends: sections.extends ?? {},
    note: "TypeScript fallback resolver (install/build native CLI for full merge)",
  };
  if (asJson) {
    return {
      exitCode: 0,
      stdout: `${JSON.stringify(payload, null, 2)}\n`,
      stderr: "",
    };
  }
  return {
    exitCode: 0,
    stdout: `Resolved ${projectRoot} (fallback — build native CLI for full cascade)\n`,
    stderr: "",
  };
}

function runConfigFallback(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
): ConfigCommandResult {
  const configFlag = flags.get("config");
  const start =
    typeof configFlag === "string"
      ? configFlag
      : positional[0]
        ? resolve(positional[0])
        : process.cwd();
  const projectRoot = findProjectRoot(start);
  if (!projectRoot) {
    return {
      exitCode: 1,
      stdout: "",
      stderr: `No ${MANIFEST} found near ${start}\n`,
    };
  }
  const sub = positional[0] ?? "";
  const asJson = flags.get("json") === true;
  if (command === "config" && sub === "validate") {
    return fallbackValidate(projectRoot, asJson);
  }
  if (command === "config" && sub === "resolve") {
    return fallbackResolve(projectRoot, asJson);
  }
  return {
    exitCode: 1,
    stdout: "",
    stderr:
      "Configuration command requires the native Spanda CLI (npm run build:rust) or cargo run -p spanda.\n",
  };
}

/**
 * Run a config/device-tree/map command via native CLI, cargo, or TS fallback.
 */
export function runConfigCommand(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
): ConfigCommandResult {
  const args = buildCliArgs(command, positional, flags);
  if (isCliAvailable()) {
    const result = runNativeCli(args);
    return {
      exitCode: result.status ?? 1,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
    };
  }
  const cargo = runCargoCli(args);
  if (cargo.exitCode === 0 || cargo.stdout || !cargo.stderr.includes("could not find")) {
    return cargo;
  }
  return runConfigFallback(command, positional, flags);
}

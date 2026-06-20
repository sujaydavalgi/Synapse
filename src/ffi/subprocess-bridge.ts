import { spawnSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import type { ExternFnDecl } from "../foundations.js";
import type { RuntimeValue } from "../runtime/interpreter.js";

type BridgeResponse = {
  ok: boolean;
  result?: unknown;
  error?: string;
};

function runtimeValueToJson(value: RuntimeValue): unknown {
  switch (value.kind) {
    case "number":
      return value.value;
    case "bool":
      return value.value;
    case "string":
      return value.value;
    case "void":
      return null;
    default:
      return String(value);
  }
}

function jsonToRuntimeValue(value: unknown): RuntimeValue {
  if (typeof value === "number") {
    return { kind: "number", value, unit: "none" };
  }
  if (typeof value === "boolean") {
    return { kind: "bool", value };
  }
  if (typeof value === "string") {
    return { kind: "string", value };
  }
  return { kind: "void" };
}

function repoRoot(): string {
  return resolve(import.meta.dirname, "../..");
}

export function pythonBridgeScriptPath(): string | null {
  const env = process.env.SPANDA_PYTHON_BRIDGE;
  if (env && existsSync(env)) return env;
  const candidates = [
    join(process.cwd(), "scripts/spanda_python_bridge.py"),
    join(repoRoot(), "scripts/spanda_python_bridge.py"),
  ];
  return candidates.find((p) => existsSync(p)) ?? null;
}

export function cppBridgeBinaryPath(): string | null {
  const env = process.env.SPANDA_CPP_BRIDGE;
  if (env && existsSync(env)) return env;
  const candidates = [
    ...cargoCppBridgePaths(),
    join(process.cwd(), "scripts/spanda_cpp_bridge"),
    join(repoRoot(), "scripts/spanda_cpp_bridge"),
  ];
  return candidates.find((p) => existsSync(p)) ?? null;
}

function cargoCppBridgePaths(): string[] {
  const roots = [
    join(repoRoot(), "target/debug/build"),
    join(repoRoot(), "target/release/build"),
  ];
  const paths: string[] = [];
  for (const root of roots) {
    if (!existsSync(root)) continue;
    for (const dir of readdirSync(root)) {
      if (!dir.startsWith("spanda-core-")) continue;
      const bin = join(root, dir, "out/spanda_cpp_bridge");
      if (existsSync(bin)) paths.push(bin);
    }
  }
  return paths;
}

function pythonCommand(): string | null {
  for (const cmd of ["python3", "python"]) {
    const result = spawnSync(cmd, ["-c", "import sys"], { stdio: "ignore" });
    if (result.status === 0) return cmd;
  }
  return null;
}

function callSubprocessBridge(
  label: string,
  executable: string,
  extraArgs: string[],
  decl: ExternFnDecl,
  args: RuntimeValue[],
  line: number,
): RuntimeValue {
  const request = JSON.stringify({
    fn: decl.name,
    args: args.map(runtimeValueToJson),
  });
  const result = spawnSync(executable, extraArgs, {
    input: `${request}\n`,
    encoding: "utf8",
  });
  if (result.error) {
    throw new Error(`${label} bridge failed: ${result.error.message} (line ${line})`);
  }
  if (result.status !== 0) {
    throw new Error(
      `${label} bridge exited with ${result.status}: ${result.stderr?.trim() || "unknown error"} (line ${line})`,
    );
  }
  let resp: BridgeResponse;
  try {
    resp = JSON.parse(result.stdout.trim()) as BridgeResponse;
  } catch (err) {
    throw new Error(
      `Invalid ${label} bridge response: ${err instanceof Error ? err.message : err} (line ${line})`,
    );
  }
  if (!resp.ok) {
    throw new Error(resp.error ?? `${label} bridge call failed (line ${line})`);
  }
  return jsonToRuntimeValue(resp.result ?? null);
}

export function callExternBridge(
  decl: ExternFnDecl,
  args: RuntimeValue[],
): RuntimeValue {
  const line = decl.span.start.line;
  if (decl.bridge === "python") {
    const script = pythonBridgeScriptPath();
    if (!script) {
      throw new Error(
        `Python bridge script not found — set SPANDA_PYTHON_BRIDGE or run from repo root (line ${line})`,
      );
    }
    const python = pythonCommand();
    if (!python) {
      throw new Error(
        `Python interpreter not found (install python3 for extern python fn) (line ${line})`,
      );
    }
    return callSubprocessBridge("Python", python, [script], decl, args, line);
  }
  if (decl.bridge === "cpp") {
    const binary = cppBridgeBinaryPath();
    if (!binary) {
      throw new Error(
        `C++ bridge binary not found — set SPANDA_CPP_BRIDGE or build spanda-core (line ${line})`,
      );
    }
    return callSubprocessBridge("C++", binary, [], decl, args, line);
  }
  throw new Error(`No native binding for extern fn '${decl.name}' (line ${line})`);
}

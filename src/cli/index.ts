#!/usr/bin/env node
/**
 * Spanda CLI entry point: check, run, verify, format, lint, doc, codegen, deploy, debug, and package commands.
 * @module
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { compileFile, run } from "../compile.js";
import { createDefaultSimulator } from "../simulator/index.js";
import { LexerError } from "../lexer/index.js";
import { ParseError } from "../parser/index.js";
import { TypeCheckError } from "../types/index.js";
import { RuntimeError } from "../runtime/index.js";
import {
  isCliAvailable,
  runNativeCli,
  fmtViaCli,
  lintViaCli,
  docViaCli,
  codegenViaCli,
  deployViaCli,
  debugViaCli,
  securityCheckViaCli,
  securityAuditViaCli,
  type VerifyResult,
} from "../rust-bridge.js";
import { securityCheck, securityAudit, reportHasErrors } from "../security/index.js";
import { runOperationalCommand } from "../operational.js";
import {
  applyRollout,
  buildDeployPlan,
  defaultRolloutOptions,
  defaultStatePath,
  emptyDeployState,
  loadDeployState,
  planRollout,
  rollbackTargets,
  serializeDeployState,
  validateRolloutCertification,
  type DeployState,
  type RolloutStrategy,
} from "../deploy-service.js";
import {
  buildDeployBundle,
  signDeployBundle,
  type DeployArtifactBundle,
} from "../deploy-bundle.js";
import {
  agentHealth,
  agentReadiness,
  defaultAgentsPath,
  executeRemoteRollback,
  executeRemoteRollout,
  lookupAgent,
  readAgentRegistryFromDisk,
  registerAgent,
  writeAgentRegistryToDisk,
} from "../deploy-remote.js";
import { startDeployAgentServer } from "../deploy-agent.js";
import { buildCertificationProof } from "../certify-prover.js";
import { defaultFleetMeshUrl } from "../fleet-mesh.js";
import { runTelemetryCli } from "../telemetry-cli.js";
import {
  defaultFleetAgentsPath,
  fleetAgentHealth,
  fleetAgentReadiness,
  lookupFleetAgent,
  readFleetAgentRegistryFromDisk,
  registerFleetAgent,
  writeFleetAgentRegistryToDisk,
} from "../fleet-remote.js";
import { startFleetAgentServer } from "../fleet-agent.js";
import {
  orchestrateFleets,
  orchestrateFleetsMesh,
  orchestrateFleetsRemote,
} from "../fleet-orchestrator.js";
import {
  adapterVerifyOk,
  readAdapterManifestSection,
  verifyAdapterPackage,
} from "../adapter-package-verify.js";
import {
  coordinateSwarms,
  coordinateSwarmsMesh,
  readSwarmStateFromDisk,
  writeSwarmStateToDisk,
} from "../swarm-coordinator.js";

const USAGE = `Spanda Programming Language — the pulse of autonomous intelligence

Usage:
  spanda check [--json] <file.sd>
  spanda verify [--json] [--target <Profile>] [--all-targets] [--simulate] [--strict-certify] <file.sd>
  spanda certify prove [--json] [--strict] [--out <file.json>] <file.sd>
  spanda compatibility [flags] <file.sd>     Alias for verify
  spanda run [--json] [--verbose] [--secure] [--inject-security-faults] [--enforce-certify] [--persist-telemetry] <file.sd>
  spanda sim [--json] [--inject-security-faults] [--enforce-certify] [--persist-telemetry] <file.sd>
  spanda security check [--json] <file.sd>
  spanda security audit [--json] <file.sd>
  spanda telemetry list|latest|heartbeats|devices|stats|export|prometheus|otlp|serve|sessions|replay|info [flags]
  spanda fmt [--json] <file.sd>
  spanda lint [--json] <file.sd>
  spanda doc [--json] [--out <file.md>] <file.sd>
  spanda codegen [--target native|wasm|esp32] [--out <file>] <file.sd>
  spanda deploy plan [--json] [--version <ver>] <file.sd>
  spanda deploy rollout [--json] [--remote] [--require-certify] [--strategy all|canary|staged] [--canary-percent N] [--version <ver>] [--dry-run] <file.sd>
  spanda deploy rollback [--json] [--remote] <file.sd>
  spanda deploy status [--json]
  spanda deploy agent start [--bind <addr>] [--target <Robot@Hardware>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>] [--require-hash] [--require-certify]
  spanda deploy agent register <Robot@Hardware> <http(s)://host:port> [--token <t>]
  spanda deploy agent list [--json]
  spanda deploy agent readiness <Robot@Hardware> [--runtime] [--inject-health-faults] [--json]
  spanda deploy --target wasm [--out <file.json>] <file.sd>
  spanda fleet run [--json] [--trace-*] [--persist-telemetry] <file.sd>
  spanda fleet orchestrate [--json] [--remote] [--mesh-url <url>] [--mesh-token <t>] <file.sd>
  spanda fleet mesh start [--bind <addr>] [--token <t>]
  spanda fleet agent start [--bind <addr>] [--robot <name>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]
  spanda fleet agent register <RobotName> <http(s)://host:port> [--token <t>]
  spanda fleet agent list [--json]
  spanda fleet agent readiness <RobotName> [--runtime] [--inject-health-faults] [--json]
  spanda swarm coordinate [--json] [--mesh-url <url>] [--mesh-token <t>] <file.sd>
  spanda debug [--break <line>] <file.sd>
  spanda ir [--json] <file.sd>
  spanda llvm-ir [--out <file.ll>] [--target-triple <triple>] <file.sd>
  spanda compile-native [--out <binary>] [--target-triple <triple>] <file.sd>

Package commands (require native CLI: npm run build:rust):
  spanda init [name] [--description <text>]
  spanda build [--project <dir>]
  spanda test [--project <dir>]
  spanda add <package> [--version <ver>] [--path <dir>] [--git <url>]
  spanda remove <package>
  spanda install [--project <dir>]
  spanda publish [--project <dir>]
  spanda verify-adapter [--project <dir>] [--import <path>] [--package <name>]
  spanda registry search <query>

Examples:
  spanda check examples/rover.sd
  spanda verify examples/hardware/rover_deploy.sd --all-targets
  spanda run examples/rover.sd
  spanda fmt examples/rover.sd
`;

type ParsedArgs = {
  command: string;
  positional: string[];
  json: boolean;
  verbose: boolean;
  flags: Map<string, string | boolean>;
};

function parseArgs(argv: string[]): ParsedArgs {
  // Description:
  //     ParseArgs.
  //
  // Inputs:
  //     argv: string[]
  //         Caller-supplied argv.
  //
  // Outputs:
  //     result: ParsedArgs
  //         Return value from `parseArgs`.
  //
  // Example:
  //     const result = parseArgs(argv);
  // Description:
  //     ParseArgs.
  //
  // Inputs:
  //     argv: string[]
  //         Caller-supplied argv.
  //
  // Outputs:
  //     result: ParsedArgs
  //         Return value from `parseArgs`.
  //
  // Example:
  //     const result = parseArgs(argv);

  // const result = parseArgs(argv);
  const positional: string[] = [];
  const flags = new Map<string, string | boolean>();
  let json = false;
  let verbose = false;

  // Loop with index variable i.
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]!;

    // continue when arg equals "--json".
    if (arg === "--json") {
      json = true;

    // Otherwise, continue when arg equals "--verbose".
    } else if (arg === "--verbose") {
      verbose = true;

    // Otherwise, continue when arg.startsWith("--").
    } else if (arg.startsWith("--")) {
      const key = arg.slice(2);
      const next = argv[i + 1];

      // continue when next && !next.startsWith("-").
      if (next && !next.startsWith("-")) {
        flags.set(key, next);
        i++;

      // Handle any remaining cases.
      } else {
        flags.set(key, true);
      }

    // Otherwise, continue when length equals 2.
    } else if (arg.startsWith("-") && arg.length === 2) {
      flags.set(arg.slice(1), true);

    // Handle any remaining cases.
    } else {
      positional.push(arg);
    }
  }
  const command = positional.shift() ?? "";
  return { command, positional, json, verbose, flags };
}

function requireNative(message: string): void {
  // Description:
  //     RequireNative.
  //
  // Inputs:
  //     message: string
  //         Caller-supplied message.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = requireNative(message);
  // Description:
  //     RequireNative.
  //
  // Inputs:
  //     message: string
  //         Caller-supplied message.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = requireNative(message);

  // const result = requireNative(message);
  if (!isCliAvailable()) {
    console.error(`Error: ${message}`);
    console.error("Build the native CLI: npm run build:rust");
    process.exit(1);
  }
}

function flagStr(flags: Map<string, string | boolean>, key: string): string | undefined {
  // Description:
  //     FlagStr.
  //
  // Inputs:
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     key: string
  //         Caller-supplied key.
  //
  // Outputs:
  //     result: string | undefined
  //         Return value from `flagStr`.
  //
  // Example:
  //     const result = flagStr(flags, key);
  // Description:
  //     FlagStr.
  //
  // Inputs:
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     key: string
  //         Caller-supplied key.
  //
  // Outputs:
  //     result: string | undefined
  //         Return value from `flagStr`.
  //
  // Example:
  //     const result = flagStr(flags, key);

  // const result = flagStr(flags, key);
  const v = flags.get(key);
  return typeof v === "string" ? v : undefined;
}

function flagBool(flags: Map<string, string | boolean>, key: string): boolean {
  // Description:
  //     FlagBool.
  //
  // Inputs:
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     key: string
  //         Caller-supplied key.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `flagBool`.
  //
  // Example:
  //     const result = flagBool(flags, key);
  // Description:
  //     FlagBool.
  //
  // Inputs:
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     key: string
  //         Caller-supplied key.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `flagBool`.
  //
  // Example:
  //     const result = flagBool(flags, key);

  // const result = flagBool(flags, key);
  return flags.get(key) === true;
}

async function main(): Promise<void> {
  // Description:
  //     Main.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `main`.
  //
  // Example:
  //     const result = main();
  // Description:
  //     Main.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `main`.
  //
  // Example:
  //     const result = main();

  // const result = main();
  const parsed = parseArgs(process.argv.slice(2));
  const { command, positional, json, verbose, flags } = parsed;

  // continue when !command || command equals "help" || command === "--help" || command === "-h".
  if (!command || command === "help" || command === "--help" || command === "-h") {
    console.log(USAGE);
    process.exit(0);
  }

  // Try the operation and handle failures below.
  try {

    // Branch on command.
    switch (command) {
      case "check":
        handleCheck(positional[0], json);
        break;
      case "verify":
      case "compatibility":
        if (positional[0] === "mission") {
          handleReadinessNative("verify", positional, flags);
        } else {
          handleVerify(positional[0], json, flags);
        }
        break;
      case "certify":
        handleCertify(positional, flags, json);
        break;
      case "run":
      case "sim":
        handleRun(positional[0], command === "sim", json, verbose, flags);
        break;
      case "security":
        handleSecurity(positional, json);
        break;
      case "telemetry":
        handleTelemetry(positional, flags, json);
        break;
      case "fmt":
        handleFmt(positional[0], json);
        break;
      case "lint":
        handleLint(positional[0], json);
        break;
      case "doc":
        handleDoc(positional[0], json, flagStr(flags, "out"));
        break;
      case "codegen":
        handleCodegen(positional[0], flagStr(flags, "target"), flagStr(flags, "out"));
        break;
      case "deploy":
        await handleDeploy(positional, flags, json);
        break;
      case "fleet":
        handleFleet(positional, flags, json);
        break;
      case "swarm":
        void handleSwarm(positional, flags, json);
        break;
      case "twin":
        if (positional[0] === "readiness") {
          handleReadinessNative("twin", positional, flags);
        } else {
          requireNative("Twin export requires the native Rust CLI.");
          const result = runNativeCli(["twin", ...positional]);
          if (result.stdout) process.stdout.write(result.stdout);
          if (result.stderr) process.stderr.write(result.stderr);
          process.exit(result.status ?? 1);
        }
        break;
      case "debug":
        handleDebug(positional[0], flags);
        break;
      case "ir":
        handleIr(positional[0], json);
        break;
      case "llvm-ir":
        handleNativeCodegen("llvm-ir", positional[0], flags);
        break;
      case "compile-native":
        handleNativeCodegen("compile-native", positional[0], flags);
        break;
      case "init":
      case "build":
      case "test":
      case "add":
      case "remove":
      case "install":
      case "publish":
      case "verify-adapter":
        handlePackage(command, positional, flags, json);
        break;
      case "registry":
        handleRegistry(positional, json);
        break;
      case "readiness":
      case "analyze-failure":
      case "safety-report":
      case "diagnose":
      case "audit":
      case "verify-fleet":
      case "verify-approval":
      case "assure":
      case "anomaly":
      case "prognostics":
      case "mission":
      case "resilience":
      case "mitigation":
      case "state":
      case "heal":
      case "recover":
      case "recovery-report":
      case "recovery":
        handleReadinessNative(command, positional, flags);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        console.log(USAGE);
        process.exit(1);
    }
  } catch (err) {

    // continue when json.
    if (json) {
      console.log(JSON.stringify({ ok: false, error: String(err) }));

    // Handle any remaining cases.
    } else {
      printError(err);
    }
    process.exit(1);
  }
}

function absPath(filePath: string | undefined): string {
  // Description:
  //     AbsPath.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //
  // Outputs:
  //     result: string
  //         Return value from `absPath`.
  //
  // Example:
  //     const result = absPath(filePath);
  // Description:
  //     AbsPath.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //
  // Outputs:
  //     result: string
  //         Return value from `absPath`.
  //
  // Example:
  //     const result = absPath(filePath);

  // const result = absPath(filePath);
  if (!filePath) {
    console.error("Error: missing file path");
    console.log(USAGE);
    process.exit(1);
  }
  return resolve(filePath);
}

function handleCheck(filePath: string | undefined, json: boolean): void {
  // Description:
  //     HandleCheck.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleCheck(filePath, json);
  // Description:
  //     HandleCheck.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleCheck(filePath, json);

  // const result = handleCheck(filePath, json);
  const abs = absPath(filePath);

  // continue when isCliAvailable().
  if (isCliAvailable()) {
    const result = runNativeCli(json ? ["check", "--json", abs] : ["check", abs]);

    // continue when json.
    if (json) {
      console.log(result.stdout ?? "");

    // Handle any remaining cases.
    } else {
      process.stdout.write(result.stdout ?? "");
      process.stderr.write(result.stderr ?? "");
    }
    process.exit(result.status === 0 ? 0 : 1);
  }
  compileFile(abs);

  // continue when json.
  if (json) {
    console.log(JSON.stringify({ ok: true, diagnostics: [] }));

  // Handle any remaining cases.
  } else {
    console.log(`✓ ${filePath} — no type errors`);
  }
}

function handleCertify(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  // Description:
  //     HandleCertify.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleCertify(positional, flags, json);

  // Emit structured certification proof artifacts for audit workflows.
  const sub = positional[0];
  if (sub !== "prove") {
    console.error("Usage: spanda certify prove [--json] [--strict] [--out <file.json>] <file.sd>");
    process.exit(1);
  }
  const { abs, program } = compileProgramOrExit(positional[1] ?? "");
  const strict = flagBool(flags, "strict");
  const report = buildCertificationProof(program, abs, strict);
  const payload = JSON.stringify(report, null, 2);
  const out = flagStr(flags, "out");
  if (out) {
    writeFileSync(resolve(out), payload);
    if (!json) console.log(`✓ Wrote certification proof to ${out}`);
  }
  if (json) {
    console.log(payload);
  } else if (!out) {
    console.log(`Certification proof for ${abs}`);
    console.log(`  Status: ${report.passed ? "PASSED" : "FAILED"}`);
    console.log(`  ${report.summary}`);
    for (const item of report.checklist) {
      const icon = item.severity === "pass" ? "✓" : item.severity === "warning" ? "⚠" : "✗";
      console.log(`  ${icon} [${item.category}] ${item.message}`);
    }
  }
  if (!report.passed) process.exit(1);
}

function handleVerify(filePath: string | undefined, json: boolean, flags: Map<string, string | boolean>): void {
  // Description:
  //     HandleVerify.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleVerify(filePath, json, flags);
  // Description:
  //     HandleVerify.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleVerify(filePath, json, flags);

  // const result = handleVerify(filePath, json, flags);
  requireNative("Hardware verification requires the native Rust CLI.");
  const abs = absPath(filePath);
  const extra: string[] = [];
  const target = flagStr(flags, "target");

  // continue when target) extra.push("--target", target.
  if (target) extra.push("--target", target);

  // continue when flagBool(flags, "all-targets")) extra.push("--all-targets".
  if (flagBool(flags, "all-targets")) extra.push("--all-targets");

  // continue when flagBool(flags, "simulate")) extra.push("--simulate".
  if (flagBool(flags, "simulate")) extra.push("--simulate");

  // continue when flagBool(flags, "strict-certify")) extra.push("--strict-certify".
  if (flagBool(flags, "strict-certify")) extra.push("--strict-certify");

  // continue when json) extra.push("--json".
  if (json) extra.push("--json");
  const result = runNativeCli(["verify", abs, ...extra]);

  // continue when json.
  if (json) {
    console.log(result.stdout ?? "");

  // Handle any remaining cases.
  } else {
    printVerifyHuman(JSON.parse(result.stdout || "{}") as VerifyResult, filePath!);
  }
  process.exit(result.status === 0 ? 0 : 1);
}

function printVerifyHuman(result: VerifyResult, filePath: string): void {
  // Description:
  //     PrintVerifyHuman.
  //
  // Inputs:
  //     result: VerifyResult
  //         Caller-supplied result.
  //     filePath: string
  //         Caller-supplied filePath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = printVerifyHuman(result, filePath);
  // Description:
  //     PrintVerifyHuman.
  //
  // Inputs:
  //     result: VerifyResult
  //         Caller-supplied result.
  //     filePath: string
  //         Caller-supplied filePath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = printVerifyHuman(result, filePath);

  // const result = printVerifyHuman(result, filePath);
  const compatible = result.compatible ?? result.ok;
  console.log(`\nHardware compatibility: ${filePath}`);

  // continue when result.target) console.log(`  Target: ${result.target}`.
  if (result.target) console.log(`  Target: ${result.target}`);
  console.log(`  Status: ${compatible ? "COMPATIBLE" : "INCOMPATIBLE"}\n`);

  // Handle each entry in items.
  for (const item of result.items) {
    const icon = item.severity === "pass" ? "✓" : item.severity === "warning" ? "⚠" : "✗";
    console.log(`  ${icon} [${item.category}] ${item.message}`);
  }

  // continue when result.matrix?.cells.length.
  if (result.matrix?.cells.length) {
    console.log("\n  Compatibility matrix:");

    // Process each cell.
    for (const cell of result.matrix.cells) {
      console.log(`    ${cell.robot} × ${cell.target}: ${cell.compatible ? "ok" : "fail"}`);
    }
  }
  console.log();
}

function handleSecurity(positional: string[], json: boolean): void {
  // Description:
  //     HandleSecurity.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleSecurity(positional, json);

  // Run spanda security check or audit on a source file.
  const sub = positional[0];
  const filePath = positional[1];
  if (!sub || !filePath || (sub !== "check" && sub !== "audit")) {
    console.error("Usage: spanda security check|audit [--json] <file.sd>");
    process.exit(1);
  }
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf8");
  if (isCliAvailable()) {
    const result = sub === "audit" ? securityAuditViaCli(source) : securityCheckViaCli(source);
    if (json) {
      console.log(JSON.stringify(result));
    } else if (result.findings.length === 0) {
      console.log(`✓ ${filePath} — no security ${sub} findings`);
    } else {
      for (const f of result.findings) {
        console.log(`${f.severity}: ${f.message} (${f.line}:${f.column})`);
      }
    }
    process.exit(reportHasErrors(result as import("../security/validate.js").SecurityReport) ? 1 : 0);
  }
  const report = sub === "audit" ? securityAudit(source) : securityCheck(source);
  if (json) {
    console.log(JSON.stringify(report));
  } else if (report.findings.length === 0) {
    console.log(`✓ ${filePath} — no security ${sub} findings`);
  } else {
    for (const f of report.findings) {
      console.log(`${f.severity}: ${f.message} (${f.line}:${f.column})`);
    }
  }
  process.exit(reportHasErrors(report) ? 1 : 0);
}

function handleTelemetry(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  const sub = positional[0];
  if (!sub) {
    console.error(
      "Usage: spanda telemetry list|latest|heartbeats|devices|stats|export|prometheus|otlp|serve|sessions|replay|info [flags]",
    );
    process.exit(1);
  }
  const args = [...positional.slice(1)];
  if (json) {
    args.push("--json");
  }
  for (const [key, value] of flags.entries()) {
    if (key === "json") {
      continue;
    }
    args.push(`--${key}`);
    if (typeof value === "string") {
      args.push(value);
    }
  }
  const nativeArgs = ["telemetry", sub, ...args];
  const native = runNativeCli(nativeArgs);
  if (native.status === 0) {
    if (native.stdout) {
      process.stdout.write(native.stdout);
    }
    if (native.stderr) {
      process.stderr.write(native.stderr);
    }
    process.exit(0);
  }
  process.exit(runTelemetryCli(sub, args));
}

function handleRun(
  filePath: string | undefined,
  verbose: boolean,
  json: boolean,
  extraVerbose: boolean,
  flags: Map<string, string | boolean>,
): void {
  // Description:
  //     HandleRun.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     verbose: boolean
  //         Caller-supplied verbose.
  //     json: boolean
  //         Caller-supplied json.
  //     extraVerbose: boolean
  //         Caller-supplied extraVerbose.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleRun(filePath, verbose, json, extraVerbose, flags);
  // Description:
  //     HandleRun.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     verbose: boolean
  //         Caller-supplied verbose.
  //     json: boolean
  //         Caller-supplied json.
  //     extraVerbose: boolean
  //         Caller-supplied extraVerbose.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleRun(filePath, verbose, json, extraVerbose, flags);

  // const result = handleRun(filePath, verbose, json, extraVerbose);
  const abs = absPath(filePath);
  const showVerbose = verbose || extraVerbose;

  // continue when isCliAvailable() && json.
  if (isCliAvailable() && json) {
    const args = ["run", "--json", abs];
    if (showVerbose) args.push("--verbose");
    if (flags.get("secure")) args.push("--secure");
    if (flags.get("inject-security-faults")) args.push("--inject-security-faults");
    if (flags.get("enforce-certify")) args.push("--enforce-certify");
    if (flagBool(flags, "persist-telemetry")) args.push("--persist-telemetry");
    const result = runNativeCli(args);
    console.log(result.stdout ?? "");
    process.exit(result.status === 0 ? 0 : 1);
  }
  runSimulation(abs, filePath!, showVerbose, flags);
}

function runSimulation(
  absPath: string,
  displayPath: string,
  verbose: boolean,
  flags: Map<string, string | boolean>,
): void {
  // Description:
  //     RunSimulation.
  //
  // Inputs:
  //     absPath: string
  //         Caller-supplied absPath.
  //     displayPath: string
  //         Caller-supplied displayPath.
  //     verbose: boolean
  //         Caller-supplied verbose.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = runSimulation(absPath, displayPath, verbose, flags);
  // Description:
  //     RunSimulation.
  //
  // Inputs:
  //     absPath: string
  //         Caller-supplied absPath.
  //     displayPath: string
  //         Caller-supplied displayPath.
  //     verbose: boolean
  //         Caller-supplied verbose.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = runSimulation(absPath, displayPath, verbose, flags);

  // const result = runSimulation(absPath, displayPath, verbose);
  const { program } = compileFile(absPath);
  const robot = program.robots[0];

  // continue when robot is falsy.
  if (!robot) {
    console.error("No robot defined in program");
    process.exit(1);
  }
  const sim = createDefaultSimulator();
  const logs: string[] = [];
  console.log(`\n🤖 Running robot "${robot.name}" from ${displayPath}\n`);
  const state = run(program, {
    backend: sim,
    maxLoopIterations: verbose ? 20 : 10,
    onLog: (msg) => logs.push(msg),
    onMotionBlocked: (reason) => logs.push(`⚠ BLOCKED: ${reason}`),
    secure: flags.get("secure") === true,
    injectSecurityFaults: flags.get("inject-security-faults") === true,
    enforceCertify: flags.get("enforce-certify") === true,
    persistTelemetry: flagBool(flags, "persist-telemetry"),
  });
  console.log("── Final State ──");
  console.log(`  Pose:     x=${state.pose.x.toFixed(3)} m, y=${state.pose.y.toFixed(3)} m, θ=${state.pose.theta.toFixed(3)} rad`);

  // continue when z differs from undefined.
  if (state.pose.z !== undefined) {
    console.log(`  Altitude: z=${state.pose.z.toFixed(3)} m`);
  }
  console.log(`  Velocity: linear=${state.velocity.linear.toFixed(3)} m/s, angular=${state.velocity.angular.toFixed(3)} rad/s`);
  console.log(`  E-stop:   ${state.emergencyStop ? "ACTIVE" : "off"}`);

  // continue when verbose.
  if (verbose) {
    console.log("\n── Simulation Log ──");

    // Iterate over getEventLog.
    for (const event of sim.getEventLog()) {
      console.log(`  ${event}`);
    }

    // continue when logs.length > 0.
    if (logs.length > 0) {
      console.log("\n── Runtime Log ──");

      // Process each log.
      for (const log of logs) {
        console.log(`  ${log}`);
      }
    }
  }
  console.log("\n✓ Simulation complete\n");
}

function handleFmt(filePath: string | undefined, json: boolean): void {
  // Description:
  //     HandleFmt.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleFmt(filePath, json);
  // Description:
  //     HandleFmt.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleFmt(filePath, json);

  // const result = handleFmt(filePath, json);
  requireNative("Formatting requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const result = fmtViaCli(source);

  // continue when json.
  if (json) {
    console.log(JSON.stringify(result));

  // Otherwise, continue when result.changed.
  } else if (result.changed) {
    writeFileSync(abs, result.formatted);
    console.log(`✓ Formatted ${filePath}`);

  // Handle any remaining cases.
  } else {
    console.log(`✓ ${filePath} — already formatted`);
  }
  process.exit(result.ok ? 0 : 1);
}

function handleLint(filePath: string | undefined, json: boolean): void {
  // Description:
  //     HandleLint.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleLint(filePath, json);
  // Description:
  //     HandleLint.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleLint(filePath, json);

  // const result = handleLint(filePath, json);
  requireNative("Linting requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const result = lintViaCli(source);

  // continue when json.
  if (json) {
    console.log(JSON.stringify(result));

  // Handle any remaining cases.
  } else {

    // continue when result.ok.
    if (result.ok) {
      console.log(`✓ ${filePath} — no lint issues`);

    // Handle any remaining cases.
    } else {
      console.error(`Lint issues in ${filePath}:`);

      // Process each issue.
      for (const issue of result.issues) {
        console.error(`  [${issue.line}:${issue.column}] ${issue.severity}: ${issue.message} (${issue.rule})`);
      }
    }
  }
  process.exit(result.ok ? 0 : 1);
}

function handleDoc(filePath: string | undefined, json: boolean, outPath: string | undefined): void {
  // Description:
  //     HandleDoc.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //     outPath: string | undefined
  //         Caller-supplied outPath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleDoc(filePath, json, outPath);
  // Description:
  //     HandleDoc.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //     outPath: string | undefined
  //         Caller-supplied outPath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleDoc(filePath, json, outPath);

  // const result = handleDoc(filePath, json, outPath);
  requireNative("Documentation generation requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const result = docViaCli(source);

  // continue when json.
  if (json) {
    console.log(JSON.stringify(result));

  // Otherwise, continue when outPath.
  } else if (outPath) {
    writeFileSync(resolve(outPath), result.markdown);
    console.log(`✓ Wrote documentation to ${outPath}`);

  // Handle any remaining cases.
  } else {
    console.log(result.markdown);
  }
  process.exit(result.ok ? 0 : 1);
}

function handleCodegen(filePath: string | undefined, target: string | undefined, outPath: string | undefined): void {
  // Description:
  //     HandleCodegen.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     target: string | undefined
  //         Caller-supplied target.
  //     outPath: string | undefined
  //         Caller-supplied outPath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleCodegen(filePath, target, outPath);
  // Description:
  //     HandleCodegen.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     target: string | undefined
  //         Caller-supplied target.
  //     outPath: string | undefined
  //         Caller-supplied outPath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleCodegen(filePath, target, outPath);

  // const result = handleCodegen(filePath, target, outPath);
  requireNative("Codegen requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const t = (target ?? "native") as "native" | "wasm" | "esp32";
  const output = codegenViaCli(source, t);

  // continue when outPath.
  if (outPath) {
    writeFileSync(resolve(outPath), output);
    console.log(`✓ Wrote codegen output to ${outPath}`);

  // Handle any remaining cases.
  } else {
    console.log(output);
  }
}

function deployStateFilePath(): string {
  // Description:
  //     DeployStateFilePath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string
  //         Return value from `deployStateFilePath`.
  //
  // Example:
  //     const result = deployStateFilePath();

  // Resolve the on-disk OTA deploy state file path.
  return process.env.SPANDA_DEPLOY_STATE ?? defaultStatePath();
}

function readDeployStateFromDisk(): DeployState {
  // Description:
  //     ReadDeployStateFromDisk.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: DeployState
  //         Return value from `readDeployStateFromDisk`.
  //
  // Example:
  //     const result = readDeployStateFromDisk();

  // Load persisted OTA deploy state, or return an empty record when missing.
  const path = deployStateFilePath();
  if (!existsSync(path)) {
    return emptyDeployState();
  }
  return loadDeployState(readFileSync(path, "utf-8"));
}

function writeDeployStateToDisk(state: DeployState): void {
  // Description:
  //     WriteDeployStateToDisk.
  //
  // Inputs:
  //     state: DeployState
  //         Caller-supplied state.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = writeDeployStateToDisk(state);

  // Persist OTA deploy state, creating parent directories when needed.
  const path = resolve(deployStateFilePath());
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, serializeDeployState(state));
}

function compileProgramOrExit(filePath: string) {
  // Description:
  //     CompileProgramOrExit.
  //
  // Inputs:
  //     filePath: string
  //         Caller-supplied filePath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = compileProgramOrExit(filePath);

  // Compile a Spanda source file and exit with a CLI error on failure.
  const abs = absPath(filePath);
  try {
    const result = compileFile(abs, "typescript");
    return { abs, program: result.program };
  } catch (err) {
    console.error(`Error compiling ${abs}: ${String(err)}`);
    process.exit(1);
  }
}

function agentsRegistryPath(): string {
  // Description:
  //     AgentsRegistryPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string
  //         Return value from `agentsRegistryPath`.
  //
  // Example:

  //     const result = agentsRegistryPath();

  return process.env.SPANDA_DEPLOY_AGENTS ?? defaultAgentsPath();
}

async function handleDeployOta(
  subcommand: string,
  args: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): Promise<void> {
  // Description:
  //     HandleDeployOta.
  //
  // Inputs:
  //     subcommand: string
  //         Caller-supplied subcommand.
  //     args: string[]
  //         Caller-supplied args.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `handleDeployOta`.
  //
  // Example:
  //     const result = handleDeployOta(subcommand, args, flags, json);
  // Description:
  //     HandleDeployOta.
  //
  // Inputs:
  //     subcommand: string
  //         Caller-supplied subcommand.
  //     args: string[]
  //         Caller-supplied args.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `handleDeployOta`.
  //
  // Example:

  //     const result = handleDeployOta(subcommand, args, flags, json);

  if (subcommand === "status") {
    const state = readDeployStateFromDisk();
    const path = deployStateFilePath();
    if (json) {
      console.log(JSON.stringify(state, null, 2));
      return;
    }
    console.log(`Deploy state (${path})`);
    const entries = Object.entries(state.currentVersion);
    if (entries.length === 0) {
      console.log("  (no deployments recorded)");
      return;
    }
    for (const [key, ver] of entries) {
      const prev = state.previousVersion[key] ?? "-";
      console.log(`  ${key}: ${ver} (previous: ${prev})`);
    }
    return;
  }

  const filePath = args.find((arg) => !arg.startsWith("-"));
  const { abs, program } = compileProgramOrExit(filePath ?? "");
  const version = flagStr(flags, "version") ?? "1.0.0";
  const plan = buildDeployPlan(program, abs, version);
  let bundle: DeployArtifactBundle = buildDeployBundle(plan);
  const signKey = flagStr(flags, "sign-key") ?? process.env.SPANDA_DEPLOY_SIGN_KEY;
  if (signKey) {
    bundle = await signDeployBundle(bundle, signKey);
  }

  if (subcommand === "plan") {
    const bundleOut = flagStr(flags, "bundle-out");
    if (bundleOut) {
      writeFileSync(resolve(bundleOut), JSON.stringify(bundle, null, 2));
      console.log(`Wrote signed deploy bundle to ${bundleOut}`);
    }
    if (json) {
      console.log(JSON.stringify(bundle.signature ? bundle : plan, null, 2));
      return;
    }
    console.log(`Deploy plan for ${abs} (version ${version})`);
    for (const assignment of plan.assignments) {
      console.log(`  ${assignment.robotName} -> ${assignment.hardware}`);
    }
    if (plan.certifications.length > 0) {
      console.log(`  certifications: ${plan.certifications.join(", ")}`);
    }
    if (plan.certificationProof) {
      const proof = plan.certificationProof;
      const status = proof.passedStrict
        ? "passed (strict)"
        : proof.passed
          ? "passed (relaxed)"
          : "failed";
      console.log(`  certification_proof: ${status} — ${proof.summary}`);
    }
    if (plan.programHash) {
      console.log(`  program_hash: ${plan.programHash}`);
    }
    if (bundle.signature) {
      console.log(`  artifact_signature: ${bundle.signature}`);
    }
    return;
  }

  if (subcommand === "rollout") {
    const strategyRaw = flagStr(flags, "strategy") ?? "all";
    const strategy = strategyRaw as RolloutStrategy;
    if (!["all", "canary", "staged"].includes(strategy)) {
      console.error(`Unknown strategy '${strategyRaw}'`);
      process.exit(1);
    }
    const dryRun = flagBool(flags, "dry-run");
    const remote = flagBool(flags, "remote");
    const requireCertify = flagBool(flags, "require-certify");
    const canaryPercent = Number.parseInt(flagStr(flags, "canary-percent") ?? "10", 10);
    const options = {
      ...defaultRolloutOptions(),
      strategy,
      canaryPercent: Number.isFinite(canaryPercent) ? canaryPercent : 10,
      version,
      dryRun,
      requireCertify,
    };
    const certifyError = validateRolloutCertification(plan, options);
    if (certifyError) {
      console.error(certifyError);
      process.exit(1);
    }
    const registry = readAgentRegistryFromDisk(agentsRegistryPath());
    const result = remote
      ? await executeRemoteRollout(plan, options, registry, bundle)
      : planRollout(plan, options);
    if (!dryRun) {
      const state = readDeployStateFromDisk();
      applyRollout(state, result);
      try {
        writeDeployStateToDisk(state);
      } catch (err) {
        console.error(`Warning: could not save deploy state: ${String(err)}`);
      }
    }
    printRolloutResult(result, json);
    return;
  }

  if (subcommand === "rollback") {
    const remote = flagBool(flags, "remote");
    const state = readDeployStateFromDisk();
    const rollbackPlan = buildDeployPlan(program, abs, "rollback");
    const registry = readAgentRegistryFromDisk(agentsRegistryPath());
    const result = remote
      ? await executeRemoteRollback(rollbackPlan, registry)
      : rollbackTargets(state, rollbackPlan);
    if (!remote) {
      try {
        writeDeployStateToDisk(state);
      } catch (err) {
        console.error(`Warning: could not save deploy state: ${String(err)}`);
      }
    } else {
      rollbackTargets(state, rollbackPlan);
      try {
        writeDeployStateToDisk(state);
      } catch (err) {
        console.error(`Warning: could not save deploy state: ${String(err)}`);
      }
    }
    printRolloutResult(result, json);
    return;
  }

  console.error(`Unknown deploy subcommand '${subcommand}'`);
  process.exit(1);
}

function printRolloutResult(
  result: ReturnType<typeof planRollout>,
  json: boolean,
): void {
  // Description:
  //     PrintRolloutResult.
  //
  // Inputs:
  //     result: ReturnType<typeof planRollout>
  //         Caller-supplied result.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = printRolloutResult(result, json);

  // Print rollout or rollback results as JSON or human-readable lines.
  if (json) {
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  console.log(
    `Rollout ${result.version} (${result.strategy}) — ${result.success ? "ok" : "failed"}`,
  );
  for (const step of result.steps) {
    console.log(
      `  ${step.robotName}@${step.hardware} -> ${step.status} v${step.version}`,
    );
  }
}

function handleDeployWasm(filePath: string | undefined, outPath: string | undefined): void {
  // Description:
  //     HandleDeployWasm.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     outPath: string | undefined
  //         Caller-supplied outPath.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleDeployWasm(filePath, outPath);

  // Emit a WASM deploy manifest using the native Rust CLI.
  requireNative("WASM deploy manifest requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const manifest = deployViaCli(source);

  // continue when outPath.
  if (outPath) {
    writeFileSync(resolve(outPath), manifest);
    console.log(`✓ Wrote WASM deploy manifest to ${outPath}`);

  // Handle any remaining cases.
  } else {
    console.log(manifest);
  }
}

function handleDeployAgent(subcommand: string | undefined, args: string[], json: boolean): void | Promise<void> {
  // Description:
  //     HandleDeployAgent.
  //
  // Inputs:
  //     subcommand: string | undefined
  //         Caller-supplied subcommand.
  //     args: string[]
  //         Caller-supplied args.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     result: void | Promise<void>
  //         Return value from `handleDeployAgent`.
  //
  // Example:

  //     const result = handleDeployAgent(subcommand, args, json);

  if (subcommand === "start") {
    let bind = "127.0.0.1:8765";
    let target = "";
    let token: string | undefined;
    let tlsCert: string | undefined;
    let tlsKey: string | undefined;
    let requireHash = false;
    let requireSignature = false;
    let requireCertify = false;
    let trustedPublicKey: string | undefined;
    for (let i = 0; i < args.length; i++) {
      if (args[i] === "--bind" && args[i + 1]) {
        bind = args[++i]!;
      } else if (args[i] === "--target" && args[i + 1]) {
        target = args[++i]!;
      } else if (args[i] === "--token" && args[i + 1]) {
        token = args[++i];
      } else if (args[i] === "--tls-cert" && args[i + 1]) {
        tlsCert = args[++i];
      } else if (args[i] === "--tls-key" && args[i + 1]) {
        tlsKey = args[++i];
      } else if (args[i] === "--require-hash") {
        requireHash = true;
      } else if (args[i] === "--require-certify") {
        requireCertify = true;
      } else if (args[i] === "--require-signature") {
        requireSignature = true;
      } else if (args[i] === "--trust-key" && args[i + 1]) {
        trustedPublicKey = args[++i];
      }
    }
    if (!target) {
      console.error("Missing --target Robot@Hardware");
      process.exit(1);
    }
    if ((tlsCert && !tlsKey) || (!tlsCert && tlsKey)) {
      console.error("Both --tls-cert and --tls-key are required for HTTPS agents");
      process.exit(1);
    }
    if (requireSignature && !trustedPublicKey) {
      console.error("Missing --trust-key when --require-signature is set");
      process.exit(1);
    }
    startDeployAgentServer({
      bind,
      target,
      token,
      tlsCert,
      tlsKey,
      requireHash,
      requireSignature,
      requireCertify,
      trustedPublicKey,
    });
    return;
  }

  if (subcommand === "register") {
    const positional = args.filter((arg) => !arg.startsWith("-") && arg !== args[args.indexOf("--token") + 1]);
    const target = positional[0];
    const url = positional[1];
    const tokenIdx = args.indexOf("--token");
    const token = tokenIdx >= 0 ? args[tokenIdx + 1] : undefined;
    if (!target || !url) {
      console.error("Usage: spanda deploy agent register <Robot@Hardware> <http(s)://host:port> [--token <t>]");
      process.exit(1);
    }
    const registry = readAgentRegistryFromDisk(agentsRegistryPath());
    try {
      writeAgentRegistryToDisk(registerAgent(registry, target, url, token), agentsRegistryPath());
      console.log(`Registered deploy agent in ${agentsRegistryPath()}`);
    } catch (err) {
      console.error(`Register failed: ${String(err)}`);
      process.exit(1);
    }
    return;
  }

  if (subcommand === "list") {
    const registry = readAgentRegistryFromDisk(agentsRegistryPath());
    if (json) {
      console.log(JSON.stringify(registry, null, 2));
      return;
    }
    console.log(`Deploy agents (${agentsRegistryPath()})`);
    if (registry.agents.length === 0) {
      console.log("  (no agents registered)");
      return;
    }
    return (async () => {
      for (const entry of registry.agents) {
        const healthy = await agentHealth(entry);
        console.log(`  ${entry.target} -> ${entry.url} (healthy=${healthy})`);
      }
    })();
  }

  if (subcommand === "readiness") {
    const target = args.find((arg) => !arg.startsWith("-") && arg !== args[args.indexOf("--token") + 1]);
    if (!target) {
      console.error("Usage: spanda deploy agent readiness <Robot@Hardware> [--runtime] [--inject-health-faults] [--json]");
      process.exit(1);
    }
    const registry = readAgentRegistryFromDisk(agentsRegistryPath());
    const entry = lookupAgent(registry, target);
    if (!entry) {
      console.error(`No deploy agent registered for target ${target}`);
      process.exit(1);
    }
    return (async () => {
      try {
        const runtime = args.includes("--runtime") || args.includes("--inject-health-faults");
        const inject = args.includes("--inject-health-faults");
        const body = await agentReadiness(entry, runtime, inject);
        if (json) {
          console.log(JSON.stringify(body, null, 2));
        } else {
          const readiness = body.readiness as { mission_ready?: boolean; score?: { total?: number } } | undefined;
          const missionReady = body.mission_ready ?? readiness?.mission_ready;
          const score = readiness?.score?.total ?? 0;
          console.log(`Agent readiness for ${target}`);
          console.log(`Mission Ready: ${missionReady ? "YES" : "NO"}`);
          console.log(`Score: ${score}/100`);
        }
        const missionReady = body.mission_ready ?? (body.readiness as { mission_ready?: boolean } | undefined)?.mission_ready;
        process.exit(missionReady ? 0 : 1);
      } catch (err) {
        console.error(String(err));
        process.exit(1);
      }
    })();
  }

  console.error("Usage: spanda deploy agent start|register|list|readiness");
  process.exit(1);
}

async function handleDeploy(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): Promise<void> {
  // Description:
  //     HandleDeploy.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `handleDeploy`.
  //
  // Example:
  //     const result = handleDeploy(positional, flags, json);
  // Description:
  //     HandleDeploy.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `handleDeploy`.
  //
  // Example:

  //     const result = handleDeploy(positional, flags, json);

  const sub = positional[0];
  if (sub === "agent") {
    await handleDeployAgent(positional[1], positional.slice(2), json);
    return;
  }
  if (sub === "plan" || sub === "rollout" || sub === "rollback" || sub === "status") {
    await handleDeployOta(sub, positional.slice(1), flags, json);
    return;
  }
  handleDeployWasm(positional[0], flagStr(flags, "out"));
}

function handleFleetOrchestrate(
  filePath: string | undefined,
  json: boolean,
  remote: boolean,
  flags: Map<string, string | boolean>,
): void | Promise<void> {
  // Description:
  //     HandleFleetOrchestrate.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //     remote: boolean
  //         Caller-supplied remote.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     result: void | Promise<void>
  //         Return value from `handleFleetOrchestrate`.
  //
  // Example:
  //     const result = handleFleetOrchestrate(filePath, json, remote, flags);

  // Coordinate fleet missions declared in a Spanda program.
  const meshUrl = flagStr(flags, "mesh-url") ?? (remote ? defaultFleetMeshUrl() : undefined);
  const meshToken = flagStr(flags, "mesh-token");
  const { abs, program } = compileProgramOrExit(filePath ?? "");
  const printResult = (result: Awaited<ReturnType<typeof orchestrateFleets>>) => {
    // Description:
    //     PrintResult.
    //
    // Inputs:
    //     result: Awaited<ReturnType<typeof orchestrateFleets>>
    //         Caller-supplied result.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     const result = printResult(result);

    if (json) {
      console.log(JSON.stringify(result, null, 2));
      return;
    }
    console.log(`Fleet orchestration for ${abs}`);
    for (const fleet of result.fleets) {
      console.log(`  fleet ${fleet.fleetName} (${fleet.coordinationMode})`);
      for (const member of fleet.members) {
        console.log(
          `    ${member.robotName} mission=${member.missionName ?? "null"} state=${member.missionState} step='${member.currentStep}' peer=${member.hasPeerLink}`,
        );
        for (const handoff of member.peerHandoffs ?? []) {
          console.log(`      handoff: ${handoff}`);
        }
      }
      for (const message of fleet.peerMessages ?? []) {
        console.log(`    peer: ${message}`);
      }
      for (const delivery of fleet.peerDeliveries ?? []) {
        console.log(
          `    mesh: ${delivery.fromRobot} -> ${delivery.toRobot} topic=${delivery.topic} step=${delivery.step} delivered=${delivery.delivered}`,
        );
      }
      if (remote || meshUrl) {
        console.log(`    remote: relayed=${fleet.remoteRelayed ?? 0} failed=${fleet.remoteFailed ?? 0}`);
      }
    }
  };

  if (meshUrl) {
    return orchestrateFleetsMesh(program, abs, meshUrl, meshToken).then(printResult);
  }
  if (remote) {
    const registry = readFleetAgentRegistry
  // Description:
  //     HandleFleetAgent.
  //
  // Inputs:
  //     subcommand: string | undefined
  //         Caller-supplied subcommand.
  //     args: string[]
  //         Caller-supplied args.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     result: void | Promise<void>
  //         Return value from `handleFleetAgent`.
  //
  // Example:

// const result = handleFleetAgent(subcommand, args, json);
FromDisk(defaultFleetAgentsPath());
    return orchestrateFleetsRemote(program, abs, registry).then(printResult);
  }
  printResult(orchestrateFleets(program, abs));
}

function handleFleetAgent(subcommand: string | undefined, args: string[], json: boolean): void | Promise<void> {
  // Description:
  //     HandleFleetAgent.
  //
  // Inputs:
  //     subcommand: string | undefined
  //         Caller-supplied subcommand.
  //     args: string[]
  //         Caller-supplied args.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     result: void | Promise<void>
  //         Return value from `handleFleetAgent`.
  //
  // Example:

  //     const result = handleFleetAgent(subcommand, args, json);

  if (subcommand === "start") {
    let bind = "127.0.0.1:8766";
    let robotName = "";
    let token: string | undefined;
    let tlsCert: string | undefined;
    let tlsKey: string | undefined;
    for (let i = 0; i < args.length; i++) {
      if (args[i] === "--bind" && args[i + 1]) {
        bind = args[++i]!;
      } else if (args[i] === "--robot" && args[i + 1]) {
        robotName = args[++i]!;
      } else if (args[i] === "--token" && args[i + 1]) {
        token = args[++i];
      } else if (args[i] === "--tls-cert" && args[i + 1]) {
        tlsCert = args[++i];
      } else if (args[i] === "--tls-key" && args[i + 1]) {
        tlsKey = args[++i];
      }
    }
    if (!robotName) {
      console.error("Missing --robot <RobotName>");
      process.exit(1);
    }
    if ((tlsCert && !tlsKey) || (!tlsCert && tlsKey)) {
      console.error("Both --tls-cert and --tls-key are required for HTTPS fleet agents");
      process.exit(1);
    }
    startFleetAgentServer({ bind, robotName, token, tlsCert, tlsKey });
    return;
  }

  if (subcommand === "register") {
    const positional = args.filter((arg) => !arg.startsWith("-") && arg !== args[args.indexOf("--token") + 1]);
    const robotName = positional[0];
    const url = positional[1];
    const tokenIdx = args.indexOf("--token");
    const token = tokenIdx >= 0 ? args[tokenIdx + 1] : undefined;
    if (!robotName || !url) {
      console.error("Usage: spanda fleet agent register <RobotName> <http(s)://host:port> [--token <t>]");
      process.exit(1);
    }
    const registry = readFleetAgentRegistryFromDisk(defaultFleetAgentsPath());
    try {
      writeFleetAgentRegistryToDisk(
        registerFleetAgent(registry, robotName, url, token),
        defaultFleetAgentsPath(),
      );
      console.log(`Registered fleet agent in ${defaultFleetAgentsPath()}`);
    } catch (err) {
      console.error(`Register failed: ${String(err)}`);
      process.exit(1);
    }
    return;
  }

  if (subcommand === "list") {
    const registry = readFleetAgentRegistryFromDisk(defaultFleetAgentsPath());
    if (json) {
      console.log(JSON.stringify(registry, null, 2));
      return;
    }
    console.log(`Fleet agents (${defaultFleetAgentsPath()})`);
    if (registry.agents.length === 0) {
      console.log("  (no agents registered)");
      return;
    }
    return (async () => {
      for (const entry of registry.agents) {
        const healthy = await fleetAgentHealth(entry);
        console.log(`  ${entry.robotName} -> ${entry.url} (healthy=${healthy})`);
      }
    })();
  }

  if (subcommand === "readiness") {
    const robot = args.find((arg) => !arg.startsWith("-") && arg !== args[args.indexOf("--token") + 1]);
    if (!robot) {
      console.error("Usage: spanda fleet agent readiness <RobotName> [--runtime] [--inject-health-faults] [--json]");
      process.exit(1);
    }
    const registry = readFleetAgentRegistryFromDisk(defaultFleetAgentsPath());
    const entry = lookupFleetAgent(registry, robot);
    if (!entry) {
      console.error(`No fleet agent registered for robot ${robot}`);
      process.exit(1);
    }
    return (async () => {
      try {
        const runtime = args.includes("--runtime") || args.includes("--inject-health-faults");
        const inject = args.includes("--inject-health-faults");
        const body = await fleetAgentReadiness(entry, runtime, inject);
        if (json) {
          console.log(JSON.stringify(body, null, 2));
        } else {
          const readiness = body.readiness as { mission_ready?: boolean; score?: { total?: number } } | undefined;
          const missionReady = body.mission_ready ?? readiness?.mission_ready;
          const score = readiness?.score?.total ?? 0;
          console.log(`Fleet agent readiness for ${robot}`);
          console.log(`Mission Ready: ${missionReady ? "YES" : "NO"}`);
          console.log(`Score: ${score}/100`);
        }
        const readiness = body.readiness as { mission_ready?: boolean } | undefined;
        const missionReady = body.mission_ready ?? readiness?.mission_ready;
        process.exit(missionReady ? 0 : 1);
      } catch (err) {
        conso
  // Description:
  //     HandleFleet.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:

// const result = handleFleet(positional, flags, json);
le.error(String(err));
        process.exit(1);
      }
    })();
  }

  console.error("Usage: spanda fleet agent start|register|list|readiness");
  process.exit(1);
}

function handleFleet(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  // Description:
  //     HandleFleet.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:

  //     const result = handleFleet(positional, flags, json);

  const sub = positional[0];
  if (sub === "readiness") {
    handleReadinessNative("fleet", positional, flags);
    return;
  }
  if (sub === "orchestrate") {
    void handleFleetOrchestrate(positional[1], json, flagBool(flags, "remote"), flags);
    return;
  }
  if (sub === "agent") {
    void handleFleetAgent(positional[1], positional.slice(2), json);
    return;
  }

  if (sub === "mesh") {
    if (positional[1] !== "start") {
      console.error("Usage: spanda fleet mesh start [--bind <addr>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]");
      process.exit(1);
    }
    requireNative("Fleet mesh start requires the native Rust CLI.");
    const args = ["fleet", "mesh", "start", ...positional.slice(2)];
    for (const [key, value] of flags) {
      if (value === true) {
        args.push(`--${key}`);
      } else if (typeof value === "string") {
        args.push(`--${key}`, value);
      }
    }
    const result = runNativeCli(args);
    if (json) {
      console.log(result.stdout ?? "");
    } else {
      process.stdout.write(result.stdout ?? "");
      process.stderr.write(result.stderr ?? "");
    }
    if (result.status !== 0) {
      process.exit(result.status ?? 1);
    }
    return;
  }

  if (sub === "run") {
    requireNative("Fleet run requires the native Rust CLI.");
    const abs = absPath(positional[1]);
    const args = ["fleet", "run", abs];
    if (json) args.push("--json");
    for (const [key, value] of flags) {
      if (value === true) {
        args.push(`--${key}`);
      } else if (typeof value === "string") {
        args.push(`--${key}`, value);
      }
    }
    const result = runNativeCli(args);
    if (json) {
      console.log(result.stdout ?? "");
    } else {
      process.stdout.write(result.stdout ?? "");
      process.stderr.write(result.stderr ?? "");
    }
    if (result.status !== 0) {
      process.exit(result.status ?? 1);
    }
    return;
  }

  console.error(`Unknown fleet subcommand '${sub ?? ""}'`);
  console.error("Usage: spanda fleet run [--json] [--trace-*] [--persist-telemetry] <file.sd>");
  console.error("       spanda fleet orchestrate [--json] [--remote] <file.sd>");
  console.error("       spanda fleet agent start|register|list");
  console.error("       spanda fleet mesh start [--bind <addr>] [--token <t>]");
  process.exit(1);
}

function handleIr(filePath: string | undefined, json: boolean): void {
  // Description:
  //     HandleIr.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleIr(filePath, json);

  // const result = handleIr(filePath, json);
  requireNative("Spanda IR lowering requires the native Rust CLI.");
  const abs = absPath(filePath);
  const args = ["ir", abs];

  // continue when json) args.push("--json".
  if (json) args.push("--json");
  const result = runNativeCli(args);

  // continue when json.
  if (json) {
    console.log(result.stdout ?? "");

  // Handle any remaining cases.
  } else {
    process.stdou
  // Description:
  //     HandleNativeCodegen.
  //
  // Inputs:
  //     command: "llvm-ir" | "compile-native"
  //         Caller-supplied command.
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:

// const result = handleNativeCodegen(command, filePath, flags);
t.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
  }
  process.exit(result.status === 0 ? 0 : 1);
}

function handleNativeCodegen(
  command: "llvm-ir" | "compile-native",
  filePath: string | undefined,
  flags: Map<string, string | boolean>,
): void {
  // Description:
  //     HandleNativeCodegen.
  //
  // Inputs:
  //     command: "llvm-ir" | "compile-native"
  //         Caller-supplied command.
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleNativeCodegen(command, filePath, flags);

  // const result = handleNativeCodegen(command, filePath, flags);
  requireNative(`${command} requires the native Rust CLI.`);
  const abs = absPath(filePath);
  const args: string[] = [command];
  const out = flagStr(flags, "out");

  // continue when out) args.push("--out", out.
  if (out) args.push("--out", out);
  const triple = flagStr(flags, "target-triple");

  // continue when triple) args.push("--target-triple", triple.
  if (triple) args.push("--target-triple", triple);
  args.p
  // Description:
  //     HandleDebug.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:

// const result = handleDebug(filePath, flags);
ush(abs);
  const result = runNativeCli(args);
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  process.exit(result.status === 0 ? 0 : 1);
}

function handleDebug(filePath: string | undefined, flags: Map<string, string | boolean>): void {
  // Description:
  //     HandleDebug.
  //
  // Inputs:
  //     filePath: string | undefined
  //         Caller-supplied filePath.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleDebug(filePath, flags);

  // const result = handleDebug(filePath, flags);
  requireNative("Debug requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const breakpoints: number[] = [];
  const br = flags.get("break");

  // continue when typeof br equals push.
  if (typeof br === "string") breakpoints.push(Number(br));
  const result = debugViaCli(source, breakpoints);

  // continue when length equals 0.
  if (result.pauses.length === 0) {
    console.log("✓ Debug session completed (no breakpoints hit)");

  // Handle any remaining cases.
  } else {
    console.log("Debug pauses:"
  // Description:
  //     HandleSwarm.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:

// const result = handleSwarm(positional, flags, json);
);

    // Process each pause.
    for (const p of result.pauses) {
      console.log(`  line ${p.line} — ${p.reason}`);
    }
  }
  process.exit(result.ok ? 0 : 1);
}

function handleSwarm(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  // Description:
  //     HandleSwarm.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleSwarm(positional, flags, json);

  // Route swarm subcommands to the experimental coordinator runtime.
  const sub = positional[0];
  if (sub !== "coordinate") {
    console.error("Usage: spanda swarm coordinate [--json] [--mesh-url <url>] [--mesh-token <t>] <file.sd>");
    process.exit(1);
  }
  const meshUrl = flagStr(flags, "mesh-url");
  const meshToken = flagStr(flags, "mesh-token");
  const run = async () => {
    // Description:
    //     Run.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     const result = run();

    const { abs, program } = compileProgramOrExit(positional[1] ?? "");
    const state = readSwarmStateFromDisk();
    const result = meshUrl
      ? await coordinateSwarmsMesh(program, abs, state, meshUrl, meshToken)
      : coordinateSwarms(program, abs, state);
    try {
      writeSwarmStateToDisk(state);
    } catch (err) {
      console.error(`Warning: could not save swarm state: ${String(err)}`);
    }
    if (json) {
      console.log(JSON.stringify(result, null, 2));
    } else {
      console.log(`Swarm coordination for ${abs}`);
      for (const swarm of result.swarms) {
        console.log(
          `  swarm ${swarm.swarmName} -> fleet ${swarm.fleetName} (${swarm.policy}, cursor=${swarm.roundRobinCursor})`,
        );
        if (swarm.activeMember) {
          console.log(`    active_member: ${swarm.activeMember}`);
        }
        for (const member of swarm.members) {
          console.log(
            `    ${member.robotName} mission=${member.missionName ?? "null"} state=${member.missionState} step='${member.currentStep}'`,
          );
        }
        for (const delivery of swarm.peerDeliveries) {
          co
  // Description:
  //     HandlePackage.
  //
  // Inputs:
  //     command: string
  //         Caller-supplied command.
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:

// const result = handlePackage(command, positional, flags, json);
nsole.log(`    follow: ${delivery.fromRobot} -> ${delivery.toRobot} step=${delivery.step}`);
        }
        if (meshUrl) {
          console.log(`    mesh: relayed=${swarm.remoteRelayed ?? 0} failed=${swarm.remoteFailed ?? 0}`);
        }
      }
    }
    process.exit(result.success ? 0 : 1);
  };
  void run();
}

function handlePackage(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  // Description:
  //     HandlePackage.
  //
  // Inputs:
  //     command: string
  //         Caller-supplied command.
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handlePackage(command, positional, flags, json);

  // const result = handlePackage(command, positional, flags, json);
  if (command === "verify-adapter" && !isCliAvailable()) {
    handleVerifyAdapterFallback(positional, flags, json);
    return;
  }
  requireNative("Package commands require the native Rust CLI.");
  const args = [command];

  // continue when json) args.push("--json".
  if (json) args.push("--json");
  const project = flagStr(flags, "project");

  // continue when project) args.push("--project", project.
  if (project) args.push("--project", project);
  const description = flagStr(flags, "description");

  // continue when description) args.push("--description", description.
  if (description) args.push("--description", description);
  const version = flagStr(flags, "version");

  // continue when version) args.push("--version", version.
  if (version) args.push("--version", version);
  const pathFlag = flagStr(flags, "path");

  // continue when pathFlag) args.push("--path", pathFlag.
  if (pathFlag) args.push("--path", pathFlag);
  // Description:
  //     HandleVerifyAdapterFallback.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:

  //     const result = handleVerifyAdapterFallback(positional, flags, json);
  const git = flagStr(flags, "git");

  // continue when git) args.push("--git", git.
  if (git) args.push("--git", git);
  args.push(...positional);
  const result = runNativeCli(args);
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  process.exit(result.status === 0 ? 0 : 1);
}

function handleVerifyAdapterFallback(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  // Description:
  //     HandleVerifyAdapterFallback.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleVerifyAdapterFallback(positional, flags, json);

  // Validate adapter package metadata without the native Rust CLI.
  const project = flagStr(flags, "project") ?? process.cwd();
  const importPath = flagStr(flags, "import");
  const packageName = flagStr(flags, "package");
  const resolvedImport = importPath ?? (packageName ? undefined : "navigation.nav2");
  try {
    const issues = verifyAdapterPackage(project, resolvedImport, packageName);
    if (json) {
      console.log(JSON.stringify({ ok: adapterVerifyOk(issues), issues }, null, 2));
    } else {
      for (const issue of issues) {
        const icon = issue.severity === "pass" ? "✓" : issue.severity === "warning" ? "⚠" : "✗";
        console.log(`  ${icon} ${issue.message}`);
      }
      if (!adapterVerifyOk(issues)) process.exit(1);
  // Description:
  //     HandleRegistry.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:

     // const result = handleRegistry(positional, json);
     const manifest = readAdapterManifestSection(project);
      console.log(`✓ Adapter package verification passed for ${manifest.packageName}`);
    }
  } catch (err) {
    if (json) {
      console.log(JSON.stringify({ ok: false, error: String(err) }));
    } else {
      console.error(`Adapter verify failed: ${String(err)}`);
    }
    process.exit(1);
  }
  void positional;
}

function handleRegistry(positional: string[], json: boolean): void {
  // Description:
  //     HandleRegistry.
  //
  // Inputs:
  //     positional: string[]
  //         Caller-supplied positional.
  //     json: boolean
  //         Caller-supplied json.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = handleRegistry(positional, json);

  // const result = handleRegistry(positional, json);
  requireNative("Registry commands require the native Rust CLI.");
  const sub = positional[0];

  // continue when sub equals "search".
  if (sub === "search") {
    const query = positional[1];

    // continue when query is falsy.
    if (!query) {
      console.error("Error: missing search query");
      process.exit(1);
    }
    const args = ["registry", "search", query];

    // continue when json) args.push("--json".
    if (json) args.push("--json");
    const result = runNativeCli(args);
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    process.exit(result.status === 0 ? 0 : 1);

  // Otherwise, continue when sub equals "info".
  } else if (sub === "info") {
    const pkg = positional[1];

    // continue when pkg is falsy.
    if (!pkg) {
      console.error("Error: missing package name");
      proce
  // Description:
  //     HandleReadinessNative.
  //
  // Inputs:
  //     command: string
  //         Caller-supplied command.
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:

// const result = handleReadinessNative(command, positional, flags);
ss.exit(1);
    }
    const result = runNativeCli(["registry", "info", pkg]);
    console.log(result.stdout ?? "");
    process.exit(result.status === 0 ? 0 : 1);

  // Handle any remaining cases.
  } else {
    console.error("Usage: spanda registry search <query> | spanda registry info <package>");
    process.exit(1);
  }
}

function handleReadinessNative(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
): void {
  // Description:
  //     HandleReadinessNative.
  //
  // Inputs:
  //     command: string
  //         Caller-supplied command.
  //     positional: string[]
  //         Caller-supplied positional.
  //     flags: Map<string, string | boolean>
  //         Caller-supplied flags.
  //
  // Outputs:
  //     None.
  //
  // Example:

  //     const result = handleReadinessNative(command, positional, flags);

  if (isCliAvailable()) {
    const args = [command, ...positional];
    for (const [key, value] of flags) {
      if (value === true) {
        args.push(`--${key}`);
      } else if (typeof value === "string") {
        args.push(`--${key}`, value);
      }
    }
    const res
  // Description:
  //     PrintError.
  //
  // Inputs:
  //     err: unknown
  //         Caller-supplied err.
  //
  // Outputs:
  //     None.
  //
  // Example:

// const result = printError(err);
ult = runNativeCli(args);
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    process.exit(result.status ?? 1);
  }

  try {
    const { exitCode, output } = runOperationalCommand(command, positional, flags);
    console.log(output);
    process.exit(exitCode);
  } catch (err) {
    console.error(String(err));
    process.exit(1);
  }
}

function printError(err: unknown): void {
  // Description:
  //     PrintError.
  //
  // Inputs:
  //     err: unknown
  //         Caller-supplied err.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = printError(err);

  // const result = printError(err);
  if (err instanceof LexerError) {
    console.error(`Lexer error [${err.line}:${err.column}]: ${err.message}`);

  // Otherwise, continue when err instanceof ParseError.
  } else if (err instanceof ParseError) {
    console.error(`Parse error [${err.line}:${err.column}]: ${err.message}`);

  // Otherwise, continue when err instanceof TypeCheckError.
  } else if (err instanceof TypeCheckError) {
    console.error("Type errors:");

    // Process each error.
    for (const e of err.errors) {
      console.error(`  [${e.line}:${e.column}] ${e.message}`);
    }

  // Otherwise, continue when err instanceof RuntimeError.
  } else if (err instanceof RuntimeError) {
    console.error(`Runtime error [line ${err.line}]: ${err.message}`);

  // Otherwise, continue when err instanceof Error.
  } else if (err instanceof Error) {
    console.error(`Error: ${err.message}`);

  // Handle any remaining cases.
  } else {
    console.error(String(err));
  }
}

main().catch((err) => {
  console.error(String(err));
  process.exit(1);
});

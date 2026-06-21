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
  type DeployState,
  type RolloutStrategy,
} from "../deploy-service.js";
import {
  agentHealth,
  defaultAgentsPath,
  executeRemoteRollback,
  executeRemoteRollout,
  readAgentRegistryFromDisk,
  registerAgent,
  writeAgentRegistryToDisk,
} from "../deploy-remote.js";
import { startDeployAgentServer } from "../deploy-agent.js";
import { orchestrateFleets } from "../fleet-orchestrator.js";

const USAGE = `Spanda Programming Language — the pulse of autonomous intelligence

Usage:
  spanda check [--json] <file.sd>
  spanda verify [--json] [--target <Profile>] [--all-targets] [--simulate] <file.sd>
  spanda compatibility [flags] <file.sd>     Alias for verify
  spanda run [--json] [--verbose] [--secure] [--inject-security-faults] <file.sd>
  spanda sim [--json] [--inject-security-faults] <file.sd>
  spanda security check [--json] <file.sd>
  spanda security audit [--json] <file.sd>
  spanda fmt [--json] <file.sd>
  spanda lint [--json] <file.sd>
  spanda doc [--json] [--out <file.md>] <file.sd>
  spanda codegen [--target native|wasm|esp32] [--out <file>] <file.sd>
  spanda deploy plan [--json] [--version <ver>] <file.sd>
  spanda deploy rollout [--json] [--remote] [--strategy all|canary|staged] [--canary-percent N] [--version <ver>] [--dry-run] <file.sd>
  spanda deploy rollback [--json] [--remote] <file.sd>
  spanda deploy status [--json]
  spanda deploy agent start [--bind <addr>] [--target <Robot@Hardware>] [--token <t>]
  spanda deploy agent register <Robot@Hardware> <http://host:port> [--token <t>]
  spanda deploy agent list [--json]
  spanda deploy --target wasm [--out <file.json>] <file.sd>
  spanda fleet run [--json] [--trace-*] <file.sd>
  spanda fleet orchestrate [--json] <file.sd>
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
  // ParseArgs.
  //
  // Parameters:
  // - `argv` — input value
  //
  // Returns:
  // `ParsedArgs`.
  //
  // Options:
  // None.
  //
  // Example:

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
  // RequireNative.
  //
  // Parameters:
  // - `message` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = requireNative(message);
  if (!isCliAvailable()) {
    console.error(`Error: ${message}`);
    console.error("Build the native CLI: npm run build:rust");
    process.exit(1);
  }
}

function flagStr(flags: Map<string, string | boolean>, key: string): string | undefined {
  // FlagStr.
  //
  // Parameters:
  // - `flags` — input value
  // - `key` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = flagStr(flags, key);
  const v = flags.get(key);
  return typeof v === "string" ? v : undefined;
}

function flagBool(flags: Map<string, string | boolean>, key: string): boolean {
  // FlagBool.
  //
  // Parameters:
  // - `flags` — input value
  // - `key` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = flagBool(flags, key);
  return flags.get(key) === true;
}

async function main(): Promise<void> {
  // Main.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
        handleVerify(positional[0], json, flags);
        break;
      case "run":
      case "sim":
        handleRun(positional[0], command === "sim", json, verbose, flags);
        break;
      case "security":
        handleSecurity(positional, json);
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
        handlePackage(command, positional, flags, json);
        break;
      case "registry":
        handleRegistry(positional, json);
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
  // AbsPath.
  //
  // Parameters:
  // - `filePath` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = absPath(filePath);
  if (!filePath) {
    console.error("Error: missing file path");
    console.log(USAGE);
    process.exit(1);
  }
  return resolve(filePath);
}

function handleCheck(filePath: string | undefined, json: boolean): void {
  // HandleCheck.
  //
  // Parameters:
  // - `filePath` — input value
  // - `json` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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

function handleVerify(filePath: string | undefined, json: boolean, flags: Map<string, string | boolean>): void {
  // HandleVerify.
  //
  // Parameters:
  // - `filePath` — input value
  // - `json` — input value
  // - `flags` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
  // PrintVerifyHuman.
  //
  // Parameters:
  // - `result` — input value
  // - `filePath` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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

function handleRun(
  filePath: string | undefined,
  verbose: boolean,
  json: boolean,
  extraVerbose: boolean,
  flags: Map<string, string | boolean>,
): void {
  // HandleRun.
  //
  // Parameters:
  // - `filePath` — input value
  // - `verbose` — input value
  // - `json` — input value
  // - `extraVerbose` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = handleRun(filePath, verbose, json, extraVerbose);
  const abs = absPath(filePath);
  const showVerbose = verbose || extraVerbose;

  // continue when isCliAvailable() && json.
  if (isCliAvailable() && json) {
    const args = ["run", "--json", abs];
    if (showVerbose) args.push("--verbose");
    if (flags.get("secure")) args.push("--secure");
    if (flags.get("inject-security-faults")) args.push("--inject-security-faults");
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
  // RunSimulation.
  //
  // Parameters:
  // - `absPath` — input value
  // - `displayPath` — input value
  // - `verbose` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
  // HandleFmt.
  //
  // Parameters:
  // - `filePath` — input value
  // - `json` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
  // HandleLint.
  //
  // Parameters:
  // - `filePath` — input value
  // - `json` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
  // HandleDoc.
  //
  // Parameters:
  // - `filePath` — input value
  // - `json` — input value
  // - `outPath` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
  // HandleCodegen.
  //
  // Parameters:
  // - `filePath` — input value
  // - `target` — input value
  // - `outPath` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
  // Resolve the on-disk OTA deploy state file path.
  return process.env.SPANDA_DEPLOY_STATE ?? defaultStatePath();
}

function readDeployStateFromDisk(): DeployState {
  // Load persisted OTA deploy state, or return an empty record when missing.
  const path = deployStateFilePath();
  if (!existsSync(path)) {
    return emptyDeployState();
  }
  return loadDeployState(readFileSync(path, "utf-8"));
}

function writeDeployStateToDisk(state: DeployState): void {
  // Persist OTA deploy state, creating parent directories when needed.
  const path = resolve(deployStateFilePath());
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, serializeDeployState(state));
}

function compileProgramOrExit(filePath: string) {
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
  return process.env.SPANDA_DEPLOY_AGENTS ?? defaultAgentsPath();
}

async function handleDeployOta(
  subcommand: string,
  args: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): Promise<void> {
  // Handle OTA deploy subcommands in the TypeScript CLI.
  //
  // Parameters:
  // - `subcommand` — plan, rollout, rollback, or status
  // - `args` — remaining positional arguments after the subcommand
  // - `flags` — parsed CLI flags
  // - `json` — emit JSON output when true
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:
  // handleDeployOta("plan", ["examples/robotics/ota_deployment.sd"], flags, false);

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

  if (subcommand === "plan") {
    if (json) {
      console.log(JSON.stringify(plan, null, 2));
      return;
    }
    console.log(`Deploy plan for ${abs} (version ${version})`);
    for (const assignment of plan.assignments) {
      console.log(`  ${assignment.robotName} -> ${assignment.hardware}`);
    }
    if (plan.certifications.length > 0) {
      console.log(`  certifications: ${plan.certifications.join(", ")}`);
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
    const canaryPercent = Number.parseInt(flagStr(flags, "canary-percent") ?? "10", 10);
    const options = {
      ...defaultRolloutOptions(),
      strategy,
      canaryPercent: Number.isFinite(canaryPercent) ? canaryPercent : 10,
      version,
      dryRun,
    };
    const registry = readAgentRegistryFromDisk(agentsRegistryPath());
    const result = remote
      ? await executeRemoteRollout(plan, options, registry)
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
  if (subcommand === "start") {
    let bind = "127.0.0.1:8765";
    let target = "";
    let token: string | undefined;
    for (let i = 0; i < args.length; i++) {
      if (args[i] === "--bind" && args[i + 1]) {
        bind = args[++i]!;
      } else if (args[i] === "--target" && args[i + 1]) {
        target = args[++i]!;
      } else if (args[i] === "--token" && args[i + 1]) {
        token = args[++i];
      }
    }
    if (!target) {
      console.error("Missing --target Robot@Hardware");
      process.exit(1);
    }
    startDeployAgentServer({ bind, target, token });
    return;
  }

  if (subcommand === "register") {
    const positional = args.filter((arg) => !arg.startsWith("-") && arg !== args[args.indexOf("--token") + 1]);
    const target = positional[0];
    const url = positional[1];
    const tokenIdx = args.indexOf("--token");
    const token = tokenIdx >= 0 ? args[tokenIdx + 1] : undefined;
    if (!target || !url) {
      console.error("Usage: spanda deploy agent register <Robot@Hardware> <http://host:port> [--token <t>]");
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

  console.error("Usage: spanda deploy agent start|register|list");
  process.exit(1);
}

async function handleDeploy(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): Promise<void> {
  // Route deploy commands to OTA handlers or legacy WASM manifest generation.
  //
  // Parameters:
  // - `positional` — command arguments after `deploy`
  // - `flags` — parsed CLI flags
  // - `json` — emit JSON output when true
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:
  // handleDeploy(["plan", "examples/robotics/ota_deployment.sd"], flags, false);

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

function handleFleetOrchestrate(filePath: string | undefined, json: boolean): void {
  // Coordinate fleet missions declared in a Spanda program.
  const { abs, program } = compileProgramOrExit(filePath ?? "");
  const result = orchestrateFleets(program, abs);
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
    }
  }
}

function handleFleet(
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  // Route fleet subcommands to orchestration or native multi-robot simulation.
  //
  // Parameters:
  // - `positional` — command arguments after `fleet`
  // - `flags` — parsed CLI flags
  // - `json` — emit JSON output when true
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:
  // handleFleet(["orchestrate", "examples/robotics/nav2_bridge.sd"], flags, false);

  const sub = positional[0];
  if (sub === "orchestrate") {
    handleFleetOrchestrate(positional[1], json);
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
  console.error("Usage: spanda fleet run [--json] [--trace-*] <file.sd>");
  console.error("       spanda fleet orchestrate [--json] <file.sd>");
  process.exit(1);
}

function handleIr(filePath: string | undefined, json: boolean): void {
  // HandleIr.
  //
  // Parameters:
  // - `filePath` — input value
  // - `json` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
  }
  process.exit(result.status === 0 ? 0 : 1);
}

function handleNativeCodegen(
  command: "llvm-ir" | "compile-native",
  filePath: string | undefined,
  flags: Map<string, string | boolean>,
): void {
  // HandleNativeCodegen.
  //
  // Parameters:
  // - `command` — input value
  // - `filePath` — input value
  // - `flags` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
  args.push(abs);
  const result = runNativeCli(args);
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  process.exit(result.status === 0 ? 0 : 1);
}

function handleDebug(filePath: string | undefined, flags: Map<string, string | boolean>): void {
  // HandleDebug.
  //
  // Parameters:
  // - `filePath` — input value
  // - `flags` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
    console.log("Debug pauses:");

    // Process each pause.
    for (const p of result.pauses) {
      console.log(`  line ${p.line} — ${p.reason}`);
    }
  }
  process.exit(result.ok ? 0 : 1);
}

function handlePackage(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  // HandlePackage.
  //
  // Parameters:
  // - `command` — input value
  // - `positional` — input value
  // - `flags` — input value
  // - `json` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = handlePackage(command, positional, flags, json);
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
  const git = flagStr(flags, "git");

  // continue when git) args.push("--git", git.
  if (git) args.push("--git", git);
  args.push(...positional);
  const result = runNativeCli(args);
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  process.exit(result.status === 0 ? 0 : 1);
}

function handleRegistry(positional: string[], json: boolean): void {
  // HandleRegistry.
  //
  // Parameters:
  // - `positional` — input value
  // - `json` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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
      process.exit(1);
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

function printError(err: unknown): void {
  // PrintError.
  //
  // Parameters:
  // - `err` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

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

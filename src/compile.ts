/**
 * Spanda compile and run pipeline (TypeScript mirror of the Rust split).
 *
 * - **Compile path** (`compile`, `check`, `compileWithRegistry`): lexer → parser → type checker,
 *   aligned with `spanda-driver` + workspace crates in Rust.
 * - **Run path** (`run`, `runSource`): parsed program → certification gate → interpreter,
 *   aligned with `spanda-interpreter::run_program` with `spanda-core::run(source)` as the facade.
 *
 * @module
 */

import { readFileSync } from "node:fs";
import { tokenize } from "./lexer/index.js";
import { parse } from "./parser/index.js";
import { typeCheck, TypeCheckError, checkWithRegistry } from "./types/index.js";
import type { ModuleRegistry } from "./modules/index.js";
import { Interpreter, type RobotBackend, type RobotState } from "./runtime/index.js";
import { createDefaultSimulator } from "./simulator/index.js";
import type { Program } from "./ast/nodes.js";
import {
  certificationRuntimeEnabledFromEnv,
  enforceCertificationRuntime,
} from "./certify-runtime.js";
import { configureSessionPersist } from "./telemetry-store.js";

export type CompileBackend = "typescript" | "rust-native" | "rust-cli";

export type CompileResult = {
  program: Program;
  source: string;
  backend: CompileBackend;
};

export type Diagnostic = {
  message: string;
  line: number;
  column: number;
};

let preferredBackend: CompileBackend = "typescript";

export function setPreferredBackend(backend: CompileBackend): void {
  // Description:
  //     SetPreferredBackend.
  //
  // Inputs:
  //     backend: CompileBackend
  //         Caller-supplied backend.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = setPreferredBackend(backend);
  // Description:
  //     SetPreferredBackend.
  //
  // Inputs:
  //     backend: CompileBackend
  //         Caller-supplied backend.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = setPreferredBackend(backend);

  // const result = setPreferredBackend(backend);
  preferredBackend = backend;
}

export function getPreferredBackend(): CompileBackend {
  // Description:
  //     GetPreferredBackend.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: CompileBackend
  //         Return value from `getPreferredBackend`.
  //
  // Example:
  //     const result = getPreferredBackend();
  // Description:
  //     GetPreferredBackend.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: CompileBackend
  //         Return value from `getPreferredBackend`.
  //
  // Example:
  //     const result = getPreferredBackend();

  // const result = getPreferredBackend();
  return preferredBackend;
}

async function tryRustCliCheck(source: string): Promise<{
  // Description:
  //     TryRustCliCheck.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: Promise<
  //         Return value from `tryRustCliCheck`.
  //
  // Example:
  //     const result = tryRustCliCheck(source);
  // Description:
  //     TryRustCliCheck.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: Promise<
  //         Return value from `tryRustCliCheck`.
  //
  // Example:
  //     const result = tryRustCliCheck(source);

 // const result = tryRustCliCheck(source);
 ok: boolean; diagnostics: Diagnostic[] } | null> {
  try {
    const { isCliAvailable, checkViaCli } = await import("./rust-bridge.js");
    if (!isCliAvailable()) return null;
    return checkViaCli(source);
  } catch {
    return null;
  }
}

export function compileWithRegistry(
  source: string,
  registry?: ModuleRegistry,
  backend?: CompileBackend,
): CompileResult {
  // Description:
  //     CompileWithRegistry.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     registry?: ModuleRegistry
  //         Caller-supplied registry?.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: CompileResult
  //         Return value from `compileWithRegistry`.
  //
  // Example:
  //     const result = compileWithRegistry(source, registry?, backend?);
  // Description:
  //     CompileWithRegistry.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     registry?: ModuleRegistry
  //         Caller-supplied registry?.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: CompileResult
  //         Return value from `compileWithRegistry`.
  //
  // Example:
  //     const result = compileWithRegistry(source, registry?, backend?);

  // const result = compileWithRegistry(source, registry?, backend?);
  const useBackend = backend ?? preferredBackend;

  // continue when useBackend equals "rust-native" || useBackend === "rust-cli".
  if (useBackend === "rust-native" || useBackend === "rust-cli") {
    throw new Error(
      "Use compileAsync() for Rust backends, or compile(source, 'typescript') for the TS interpreter",
    );
  }
  const tokens = tokenize(source);
  const program = parse(tokens);
  checkWithRegistry(program, registry);
  return { program, source, backend: "typescript" };
}

export function compile(source: string, backend?: CompileBackend): CompileResult {
  // Description:
  //     Compile.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: CompileResult
  //         Return value from `compile`.
  //
  // Example:
  //     const result = compile(source, backend?);
  // Description:
  //     Compile.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: CompileResult
  //         Return value from `compile`.
  //
  // Example:
  //     const result = compile(source, backend?);

  // const result = compile(source, backend?);
  const useBackend = backend ?? preferredBackend;

  // continue when useBackend equals "rust-native" || useBackend === "rust-cli".
  if (useBackend === "rust-native" || useBackend === "rust-cli") {
    throw new Error(
      "Use compileAsync() for Rust backends, or compile(source, 'typescript') for the TS interpreter",
    );
  }
  const tokens = tokenize(source);
  const program = parse(tokens);
  typeCheck(program);
  return { program, source, backend: "typescript" };
}

export async function compileAsync(source: string, backend?: CompileBackend): Promise<CompileResult> {
  // Description:
  //     CompileAsync.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: Promise<CompileResult>
  //         Return value from `compileAsync`.
  //
  // Example:
  //     const result = compileAsync(source, backend?);
  // Description:
  //     CompileAsync.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: Promise<CompileResult>
  //         Return value from `compileAsync`.
  //
  // Example:
  //     const result = compileAsync(source, backend?);

  // const result = compileAsync(source, backend?);
  const useBackend = backend ?? preferredBackend;

  // continue when useBackend equals "rust-cli" || useBackend === "rust-native".
  if (useBackend === "rust-cli" || useBackend === "rust-native") {
    const cliResult = await tryRustCliCheck(source);

    // continue when cliResult && !cliResult.ok.
    if (cliResult && !cliResult.ok) {
      throw new TypeCheckError(cliResult.diagnostics);
    }

    // continue when cliResult?.ok.
    if (cliResult?.ok) {

      // Try the operation and handle failures below.
      try {
        const tokens = tokenize(source);
        const program = parse(tokens);
        return { program, source, backend: "rust-cli" };
      } catch {
        return {
          program: {
            kind: "Program",
            moduleName: null,
            imports: [],
            functions: [],
            tests: [],
            externFunctions: [],
            structs: [],
            enums: [],
            traits: [],
            hardwareProfiles: [],
            deployments: [],
            requiresHardware: null,
            requiresNetwork: null,
            requiresConnectivity: null,
            geofences: [],
            fleets: [],
            swarms: [],
            programSafetyZones: [],
            certifications: [],
            connectivityPolicies: [],
            bleServices: [],
            simulateCompatibility: null,
            messages: [],
            validateRules: [],
            killSwitches: [],
            healthChecks: [],
            healthPolicies: [],
            requiresCapabilities: [],
            knowledgeModels: [],
            stateEstimators: [],
            anomalyDetectors: [],
            anomalyHandlers: [],
            prognostics: [],
            mitigations: [],
            operatingModes: [],
            missionPlans: [],
            resiliencePolicies: [],
            recoveryPolicies: [],
            assuranceCases: [],
            robots: [],
            span: {
              start: { line: 1, column: 1, offset: 0 },
              end: { line: 1, column: 1, offset: 0 },
            },
          },
          source,
          backend: "rust-cli",
        };
      }
    }
  }
  return compile(source, "typescript");
}

export function compileFile(path: string, backend?: CompileBackend): CompileResult {
  // Description:
  //     CompileFile.
  //
  // Inputs:
  //     path: string
  //         Caller-supplied path.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: CompileResult
  //         Return value from `compileFile`.
  //
  // Example:
  //     const result = compileFile(path, backend?);
  // Compute source for the following logic.
  const source = readFileSync(path, "utf-8");
  return compile(source, backend);
}

export async function compileFileAsync(path: string, backend?: CompileBackend): Promise<CompileResult> {
  // Description:
  //     CompileFileAsync.
  //
  // Inputs:
  //     path: string
  //         Caller-supplied path.
  //     backend?: CompileBackend
  //         Caller-supplied backend?.
  //
  // Outputs:
  //     result: Promise<CompileResult>
  //         Return value from `compileFileAsync`.
  //
  // Example:
  //     const result = compileFileAsync(path, backend?);
  // Compute source for the following logic.
  const source = readFileSync(path, "utf-8");
  return compileAsync(source, backend);
}

export type RunOptions = {
  backend: RobotBackend;
  entryBehavior?: string;
  maxLoopIterations?: number;
  onMotionBlocked?: (reason: string) => void;
  onLog?: (message: string) => void;
  /** When set, attempt Rust CLI run before TS interpreter */
  rustCli?: boolean;
  moduleRegistry?: ModuleRegistry;
  recordTrace?: boolean;
  traceSource?: string;
  schedulerClock?: "sim" | "wall";
  secure?: boolean;
  injectSecurityFaults?: boolean;
  enforceCertify?: boolean;
  persistTelemetry?: boolean;
};

export function run(program: Program, options: RunOptions): RobotState {
  // Description:
  //     Run.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //     options: RunOptions
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: RobotState
  //         Return value from `run`.
  //
  // Example:
  //     const result = run(program, options);
  // Description:
  //     Run.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //     options: RunOptions
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: RobotState
  //         Return value from `run`.
  //
  // Example:
  //     const result = run(program, options);

  // const result = run(program, options);
  configureSessionPersist(options.persistTelemetry ?? false);
  if (options.enforceCertify || certificationRuntimeEnabledFromEnv()) {
    enforceCertificationRuntime(program, true);
  }
  const interpreter = new Interpreter({
    backend: options.backend,
    maxLoopIterations: options.maxLoopIterations,
    onMotionBlocked: options.onMotionBlocked,
    onLog: options.onLog,
    moduleRegistry: options.moduleRegistry,
    recordTrace: options.recordTrace,
    traceSource: options.traceSource,
    schedulerClock: options.schedulerClock,
    secure: options.secure,
    injectSecurityFaults: options.injectSecurityFaults,
  });
  return interpreter.run(program, options.entryBehavior);
}

export async function runSource(source: string, options: RunOptions): Promise<RobotState> {
  // Description:
  //     RunSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     options: RunOptions
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: Promise<RobotState>
  //         Return value from `runSource`.
  //
  // Example:
  //     const result = runSource(source, options);
  // Description:
  //     RunSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     options: RunOptions
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: Promise<RobotState>
  //         Return value from `runSource`.
  //
  // Example:
  //     const result = runSource(source, options);

  // const result = runSource(source, options);
  if (options.rustCli) {

    // Try the operation and handle failures below.
    try {
      const { isCliAvailable, runViaCli } = await import("./rust-bridge.js");

      // continue when isCliAvailable().
      if (isCliAvailable()) {
        const result = runViaCli(source);
        return {
          pose: {
            x: result.state.pose.x,
            y: result.state.pose.y,
            theta: result.state.pose.theta,
            z: result.state.pose.z,
          },
          velocity: {
            linear: result.state.velocity.linear,
            angular: result.state.velocity.angular,
          },
          emergencyStop: result.state.emergency_stop,
        };
      }
    } catch {
      /* fall through to TS */
    }
  }
  const { program } = compile(source);
  return run(program, options);
}

export function runFile(path: string, options: RunOptions): RobotState {
  // Description:
  //     RunFile.
  //
  // Inputs:
  //     path: string
  //         Caller-supplied path.
  //     options: RunOptions
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: RobotState
  //         Return value from `runFile`.
  //
  // Example:
  //     const result = runFile(path, options);
  // Description:
  //     RunFile.
  //
  // Inputs:
  //     path: string
  //         Caller-supplied path.
  //     options: RunOptions
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: RobotState
  //         Return value from `runFile`.
  //
  // Example:
  //     const result = runFile(path, options);

  // const result = runFile(path, options);
  const { program } = compileFile(path);
  return run(program, options);
}

export type VerifyHardwareOptions = {
  target?: string;
  allTargets?: boolean;
  simulate?: boolean;
  rustCli?: boolean;
};

export async function verifyHardware(
  source: string,
  options: VerifyHardwareOptions = {
  },
): Promise<import("./rust-bridge.js").VerifyResult> {

  // Description:

  //     VerifyHardware.

  //

  // Inputs:

  //     source: string

  //         Caller-supplied source.

  //     options: VerifyHardwareOptions = { }

  //         Caller-supplied options.

  //

  // Outputs:

  //     result: Promise<import("./rust-bridge.js").VerifyResult>

  //         Return value from `verifyHardware`.

  //

  // Example:

  //     const result = verifyHardware(source, options);

  // Description:
  //     VerifyHardware.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.

  //     options: VerifyHardwareOptions = {
  }
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: Promise<import("./rust-bridge.js").VerifyResult>
  //         Return value from `verifyHardware`.
  //
  // Example:
  //     const result = verifyHardware(source, options);
  // Description:
  //     VerifyHardware.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.

  //     options: VerifyHardwareOptions = {
  }
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: Promise<import("./rust-bridge.js").VerifyResult>
  //         Return value from `verifyHardware`.
  //
  // Example:
  //     const result = verifyHardware(source, options);
  // Description:
  //     VerifyHardware.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.

  //     options: VerifyHardwareOptions = {
  }
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: Promise<import("./rust-bridge.js").VerifyResult>
  //         Return value from `verifyHardware`.
  //
  // Example:
  //     const result = verifyHardware(source, options);

  // const result = verifyHardware(source, options);
  const { verifyViaCli, isCliAvailable } = await import("./rust-bridge.js");

  // continue when isCliAvailable is falsy.
  if (!isCliAvailable()) {
    try {
      const { tokenize } = await import("./lexer/index.js");
      const { parse } = await import("./parser/index.js");
      const { verifyHardwareProgram } = await import("./hardware-verify.js");
      const program = parse(tokenize(source));
      return verifyHardwareProgram(program, {
        target: options.target,
        allTargets: options.allTargets,
        simulate: options.simulate,
      });
    } catch (err) {
      return {
        ok: false,
        items: [
          {
            category: "error",
            message:
              err instanceof Error
                ? err.message
                : "Hardware verification requires native CLI (npm run build:rust)",
            severity: "error",
            line: 1,
            column: 1,
          },
        ],
      };
    }
  }
  const args: string[] = [];

  // continue when options.target) args.push("--target", options.target.
  if (options.target) args.push("--target", options.target);

  // continue when options.allTargets) args.push("--all-targets".
  if (options.allTargets) args.push("--all-targets");

  // continue when options.simulate) args.push("--simulate".
  if (options.simulate) args.push("--simulate");
  return verifyViaCli(source, args);
}

export type TestRunResult = {
  passed: number;
  failed: number;
  logs: string[];
};

export function runTestsWithRegistry(
  source: string,
  registry?: ModuleRegistry,
): TestRunResult {
  // Description:
  //     RunTestsWithRegistry.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     registry?: ModuleRegistry
  //         Caller-supplied registry?.
  //
  // Outputs:
  //     result: TestRunResult
  //         Return value from `runTestsWithRegistry`.
  //
  // Example:
  //     const result = runTestsWithRegistry(source, registry?);
  // Description:
  //     RunTestsWithRegistry.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     registry?: ModuleRegistry
  //         Caller-supplied registry?.
  //
  // Outputs:
  //     result: TestRunResult
  //         Return value from `runTestsWithRegistry`.
  //
  // Example:
  //     const result = runTestsWithRegistry(source, registry?);

  // const result = runTestsWithRegistry(source, registry?);
  const { program } = compileWithRegistry(source, registry);
  const logs: string[] = [];
  const backend = createDefaultSimulator();

  // Try the operation and handle failures below.
  try {
    const interpreter = new Interpreter({
      backend,
      maxLoopIterations: 10,
      moduleRegistry: registry,
      onLog: (msg) => logs.push(msg),
    });
    interpreter.runTests(program);
    return { passed: program.tests.length, failed: 0, logs };
  } catch (e) {
    logs.push(e instanceof Error ? e.message : String(e));
    return { passed: 0, failed: Math.max(program.tests.length, 1), logs };
  }
}

export function runTests(source: string): TestRunResult {
  // Description:
  //     RunTests.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: TestRunResult
  //         Return value from `runTests`.
  //
  // Example:
  //     const result = runTests(source);
  // Description:
  //     RunTests.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: TestRunResult
  //         Return value from `runTests`.
  //
  // Example:
  //     const result = runTests(source);

  // const result = runTests(source);
  return runTestsWithRegistry(source, undefined);
}

export { ModuleRegistry, loadProjectModules } from "./modules/index.js";

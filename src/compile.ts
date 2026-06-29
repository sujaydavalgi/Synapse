/**
 * Spanda compile pipeline (TypeScript mirror of the Rust driver compile path).
 *
 * Lexer → parser → type checker only. Run execution lives in `src/cli/run-program.ts`.
 *
 * @module
 */

import { readFileSync } from "node:fs";
import { tokenize } from "./lexer/index.js";
import { parse } from "./parser/index.js";
import { typeCheck, TypeCheckError, checkWithRegistry, type CheckerHost, builtinCheckerHost } from "./types/index.js";
import type { ModuleRegistry } from "./modules/index.js";
import type { Program } from "./ast/nodes.js";

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
let compileCheckerHost: CheckerHost = builtinCheckerHost;

export function setCompileCheckerHost(host: CheckerHost): void {
  // Set the checker host used by compile() and checkWithRegistry() in this process.
  compileCheckerHost = host;
}

export function getCompileCheckerHost(): CheckerHost {
  return compileCheckerHost;
}

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
  checkWithRegistry(program, registry, compileCheckerHost);
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
  typeCheck(program, compileCheckerHost);
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
            continuityPolicies: [],
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

export type VerifyHardwareOptions = {
  target?: string;
  allTargets?: boolean;
  simulate?: boolean;
  rustCli?: boolean;
  host?: import("./hardware-verify.js").HardwareVerifyHost;
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
        host: options.host,
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

export { ModuleRegistry, loadProjectModules } from "./modules/index.js";

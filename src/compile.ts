import { readFileSync } from "node:fs";
import { tokenize } from "./lexer/index.js";
import { parse } from "./parser/index.js";
import { typeCheck, TypeCheckError } from "./types/index.js";
import { Interpreter, type RobotBackend, type RobotState } from "./runtime/index.js";
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

export function setPreferredBackend(backend: CompileBackend): void {
  preferredBackend = backend;
}

export function getPreferredBackend(): CompileBackend {
  return preferredBackend;
}

async function tryRustCliCheck(source: string): Promise<{ ok: boolean; diagnostics: Diagnostic[] } | null> {
  try {
    const { isCliAvailable, checkViaCli } = await import("./rust-bridge.js");
    if (!isCliAvailable()) return null;
    return checkViaCli(source);
  } catch {
    return null;
  }
}

export function compile(source: string, backend?: CompileBackend): CompileResult {
  const useBackend = backend ?? preferredBackend;

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
  const useBackend = backend ?? preferredBackend;

  if (useBackend === "rust-cli" || useBackend === "rust-native") {
    const cliResult = await tryRustCliCheck(source);
    if (cliResult && !cliResult.ok) {
      throw new TypeCheckError(cliResult.diagnostics);
    }
    if (cliResult?.ok) {
      const tokens = tokenize(source);
      const program = parse(tokens);
      return { program, source, backend: "rust-cli" };
    }
  }

  return compile(source, "typescript");
}

export function compileFile(path: string, backend?: CompileBackend): CompileResult {
  const source = readFileSync(path, "utf-8");
  return compile(source, backend);
}

export async function compileFileAsync(path: string, backend?: CompileBackend): Promise<CompileResult> {
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
};

export function run(program: Program, options: RunOptions): RobotState {
  const interpreter = new Interpreter({
    backend: options.backend,
    maxLoopIterations: options.maxLoopIterations,
    onMotionBlocked: options.onMotionBlocked,
    onLog: options.onLog,
  });
  return interpreter.run(program, options.entryBehavior);
}

export async function runSource(source: string, options: RunOptions): Promise<RobotState> {
  if (options.rustCli) {
    try {
      const { isCliAvailable, runViaCli } = await import("./rust-bridge.js");
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
  const { program } = compileFile(path);
  return run(program, options);
}

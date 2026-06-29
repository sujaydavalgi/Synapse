/**
 * Type checker host boundary — inject runtime registries and validators at compile time.
 * @module
 */

import type {
  HalMemberDecl,
  MessageDecl,
  Span,
  SpandaType,
} from "../ast/nodes.js";
import type { StructDecl } from "../foundations.js";
import type {
  PipelineDecl,
  RecoverDecl,
  ResourceBudgetDecl,
  TaskDecl,
  WatchdogDecl,
} from "../foundations.js";
import {
  MessageRegistry,
  isCommCapability,
  transportFromIdent,
  type TransportKind,
} from "../comm/decls.js";

export type CheckerDiagnostic = {
  message: string;
  line: number;
  column: number;
};

export type ResolvedLibModule = {
  sensors: Record<string, unknown>;
};

export type CheckerHost = {
  resolveFfiImport(path: string): unknown | undefined;
  resolveImport(path: string): ResolvedLibModule | undefined;
  resolveAiImport(path: string): unknown | undefined;
  getSocProfile(name: string): unknown | undefined;
  validateHalAgainstSoc(profile: unknown, members: unknown[]): CheckerDiagnostic[];
  halMemberFromDecl(decl: HalMemberDecl): unknown;
  validateTaskTiming(task: TaskDecl): CheckerDiagnostic[];
  validateTaskPriority(task: TaskDecl): CheckerDiagnostic[];
  validatePipeline(pipeline: PipelineDecl): CheckerDiagnostic[];
  validateWatchdog(watchdog: WatchdogDecl, taskNames: string[]): CheckerDiagnostic[];
  validateResourceBudget(budget: ResourceBudgetDecl, span: Span): CheckerDiagnostic[];
  validateRecover(recover: RecoverDecl): CheckerDiagnostic[];
  isKnownCapability(cap: string): boolean;
  parseTrustLevel(level: string): string | null;
  createMessageRegistry(): MessageRegistry;
  messageRegistryFromProgram(messages: MessageDecl[], structs: StructDecl[]): MessageRegistry;
  isCommCapability(action: string): boolean;
  transportFromIdent(s: string): TransportKind | null;
};

const BUILTIN_CAPABILITIES = new Set([
  "navigate",
  "localize",
  "map",
  "plan",
  "perceive",
  "manipulate",
  "communicate",
  "monitor",
]);

const BUILTIN_TRUST_LEVELS = new Set(["untrusted", "low", "medium", "high", "verified"]);

export const builtinCheckerHost: CheckerHost = {
  resolveFfiImport: () => undefined,
  resolveImport: () => undefined,
  resolveAiImport: () => undefined,
  getSocProfile: () => undefined,
  validateHalAgainstSoc: () => [],
  halMemberFromDecl: (decl) => ({ name: decl.name }),
  validateTaskTiming: () => [],
  validateTaskPriority: () => [],
  validatePipeline: () => [],
  validateWatchdog: () => [],
  validateResourceBudget: () => [],
  validateRecover: () => [],
  isKnownCapability: (cap) => BUILTIN_CAPABILITIES.has(cap.toLowerCase()),
  parseTrustLevel: (level) => (BUILTIN_TRUST_LEVELS.has(level.toLowerCase()) ? level : null),
  createMessageRegistry: () => MessageRegistry.new(),
  messageRegistryFromProgram: (messages, structs) => MessageRegistry.fromProgram(messages, structs),
  isCommCapability,
  transportFromIdent,
};

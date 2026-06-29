/**
 * Full type-checker host wiring real runtime registries and validators.
 * @module
 */

import { resolveFfiImport } from "../ffi/registry.js";
import { resolveImport } from "../lib/registry.js";
import { resolveAiImport } from "../ai/registry.js";
import { getSocProfile, validateHalAgainstSoc } from "../soc/index.js";
import {
  validateTaskTiming,
  validateTaskPriority,
  validatePipeline,
  validateWatchdog,
  validateResourceBudget,
  validateRecover,
} from "../reliability.js";
import { halMemberFromDecl } from "../hal/index.js";
import { isKnownCapability, parseTrustLevel } from "../security/index.js";
import {
  MessageRegistry,
  isCommCapability,
  transportFromIdent,
} from "../comm/decls.js";
import type { CheckerHost } from "../types/checker-host.js";

export function createFullCheckerHost(): CheckerHost {
  // Wire the compile-time checker to production registries and validators.
  return {
    resolveFfiImport: (path) => resolveFfiImport(path),
    resolveImport: (path) => {
      const lib = resolveImport(path);
      return lib ? { sensors: lib.sensors } : undefined;
    },
    resolveAiImport: (path) => resolveAiImport(path),
    getSocProfile: (name) => getSocProfile(name),
    validateHalAgainstSoc: (profile, members) =>
      validateHalAgainstSoc(
        profile as Parameters<typeof validateHalAgainstSoc>[0],
        members as Parameters<typeof validateHalAgainstSoc>[1],
      ).map((err) => ({
        message: err.message,
        line: err.line ?? 1,
        column: err.column ?? 1,
      })),
    halMemberFromDecl,
    validateTaskTiming,
    validateTaskPriority,
    validatePipeline,
    validateWatchdog,
    validateResourceBudget,
    validateRecover,
    isKnownCapability,
    parseTrustLevel: (level) => (parseTrustLevel(level) ? level : null),
    createMessageRegistry: () => MessageRegistry.new(),
    messageRegistryFromProgram: (messages, structs) => MessageRegistry.fromProgram(messages, structs),
    isCommCapability,
    transportFromIdent,
  };
}

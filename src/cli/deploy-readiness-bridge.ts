/**
 * Deploy and fleet agent readiness evaluation bridge.
 * @module
 */

import {
  evaluateReadinessSource,
  type ReadinessOptions,
  type ReadinessReport,
} from "../readiness.js";
import type { ReadinessEvaluator } from "../deploy-agent.js";

export type { ReadinessEvaluator };

export const deployReadinessEvaluator: ReadinessEvaluator = evaluateReadinessSource;

export function createDeployReadinessEvaluator(): ReadinessEvaluator {
  return evaluateReadinessSource;
}

export type { ReadinessOptions, ReadinessReport };

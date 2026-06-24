/**
 * Browser-local readiness evaluation for the web operations panel.
 * @module
 */

import { tokenize } from "@spanda/core/lexer/index.js";
import { parse } from "@spanda/core/parser/index.js";
import {
  evaluateReadinessTs,
  type ReadinessReport,
} from "@spanda/core/readiness.js";

export type { ReadinessReport };

export function evaluateReadinessSource(
  source: string,
  options: {
    target?: string;
    includeRuntime?: boolean;
    injectHealthFaults?: boolean;
  } = {},
): ReadinessReport {

  // Description:

  //     EvaluateReadinessSource.

  //

  // Inputs:

  //     source: string

  //         Caller-supplied source.

  //     options: { target?: string; includeRuntime?: boolean; injectHealthFaults?: boolean; } = {}

  //         Caller-supplied options.

  //

  // Outputs:

  //     result: ReadinessReport

  //         Return value from `evaluateReadinessSource`.

  //

  // Example:

  //     const result = evaluateReadinessSource(source, options);

  // Description:
  //     EvaluateReadinessSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.

    // options: {
    target?: string;
    includeRuntime?: boolean;
    injectHealthFaults?: boolean;
  } = {}
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: ReadinessReport
  //         Return value from `evaluateReadinessSource`.
  //
  // Example:
  //     const result = evaluateReadinessSource(source, options);
  // Description:
  //     EvaluateReadinessSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.

    // options: {
    target?: string;
    includeRuntime?: boolean;
    injectHealthFaults?: boolean;
  } = {}
  //         Caller-supplied options.
  //
  // Outputs:
  //     result: ReadinessReport
  //         Return value from `evaluateReadinessSource`.
  //
  // Example:

  //     const result = evaluateReadinessSource(source, options);

  const program = parse(tokenize(source));
  return evaluateReadinessTs(program, options);
}

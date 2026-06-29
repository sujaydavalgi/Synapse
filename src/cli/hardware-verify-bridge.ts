/**
 * Hardware verification host bridge for CLI verify paths.
 * @module
 */

import { verifyFrameworkImports } from "../adapter-verify.js";
import { verifyCertificationProof } from "../certify-verify.js";
import type { HardwareVerifyHost } from "../hardware-verify.js";

export function createHardwareVerifyHost(): HardwareVerifyHost {
  // Wire adapter and certification verify helpers into hardware verification.
  return {
    verifyFrameworkImports,
    verifyCertificationProof,
  };
}

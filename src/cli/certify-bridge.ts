/**
 * Deploy certification proof bridge for OTA planning and remote deploy.
 * @module
 */

import type { Program } from "../ast/nodes.js";
import {
  buildCertificationProofSummary,
  type CertificationProofSummary,
} from "../certify-prover.js";

export type { CertificationProofSummary };

export type CertificationProver = {
  buildProofSummary(program: Program, programPath: string): CertificationProofSummary;
};

export const defaultCertificationProver: CertificationProver = {
  buildProofSummary: buildCertificationProofSummary,
};

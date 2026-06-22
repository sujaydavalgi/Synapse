/**
 * Structured certification proof artifacts for CI and audit workflows.
 * @module
 */

import type { Program } from "./ast/nodes.js";
import type { CompatItem } from "./rust-bridge.js";
import { verifyCertificationProof } from "./certify-verify.js";

export type CertificationEntry = {
  standard: string;
  level?: string;
};

export type DeployTargetEntry = {
  robotName: string;
  hardware: string;
};

export type CertificationProofReport = {
  program: string;
  programHash?: string;
  strict: boolean;
  passed: boolean;
  certifications: CertificationEntry[];
  deployTargets: DeployTargetEntry[];
  checklist: CompatItem[];
  summary: string;
};

export type CertificationProofSummary = {
  passed: boolean;
  passedStrict: boolean;
  summary: string;
  errorCount: number;
  warningCount: number;
};

export function buildCertificationProof(
  program: Program,
  programPath: string,
  strict: boolean,
): CertificationProofReport {
  // Aggregate certify metadata and checklist items into an audit artifact.
  const checklist = verifyCertificationProof(program, strict);
  const passed = !checklist.some((item) => item.severity === "error");
  const certifications = (program.certifications ?? []).map((cert) => ({
    standard: cert.standard,
    level: cert.level ?? undefined,
  }));
  const deployTargets = program.deployments.flatMap((deploy) =>
    deploy.targets.map((hardware) => ({
      robotName: deploy.robotName,
      hardware,
    })),
  );
  const errorCount = checklist.filter((item) => item.severity === "error").length;
  const warningCount = checklist.filter((item) => item.severity === "warning").length;
  const summary = passed
    ? `Certification proof passed (${deployTargets.length} deploy targets, ${certifications.length} certify blocks)`
    : `Certification proof failed with ${errorCount} error(s) and ${warningCount} warning(s)`;
  return {
    program: programPath,
    strict,
    passed,
    certifications,
    deployTargets,
    checklist,
    summary,
  };
}

export function buildCertificationProofSummary(
  program: Program,
  programPath: string,
): CertificationProofSummary {
  // Derive non-strict and strict proof outcomes for deploy plan reporting.
  const proof = buildCertificationProof(program, programPath, false);
  const strict = buildCertificationProof(program, programPath, true);
  const errorCount = strict.checklist.filter((item) => item.severity === "error").length;
  const warningCount = strict.checklist.filter((item) => item.severity === "warning").length;
  const summary = strict.passed ? proof.summary : strict.summary;
  return {
    passed: proof.passed,
    passedStrict: strict.passed,
    summary,
    errorCount,
    warningCount,
  };
}

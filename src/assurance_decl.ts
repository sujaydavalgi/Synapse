/**
 * Mission assurance declaration AST types (mirrors spanda-ast assurance_decl).
 * @module
 */

import type { Span } from "./ast/nodes.js";

export type KnowledgeComponent = { name: string; span: Span };

export type KnowledgeDependency = {
  capability: string;
  requires: string[];
  span: Span;
};

export type KnowledgeModelDecl = {
  kind: "KnowledgeModelDecl";
  name: string;
  components: KnowledgeComponent[];
  dependencies: KnowledgeDependency[];
  span: Span;
};

export type StateEstimatorDecl = {
  kind: "StateEstimatorDecl";
  name: string;
  inputs: string[];
  outputType: string;
  span: Span;
};

export type ExpectedBehavior = {
  metric: string;
  operator: string;
  threshold: string;
  span: Span;
};

export type AnomalyDetectorDecl = {
  kind: "AnomalyDetectorDecl";
  name: string;
  expected: ExpectedBehavior[];
  span: Span;
};

export type AnomalyHandlerDecl = {
  kind: "AnomalyHandlerDecl";
  detector: string;
  severity: string;
  actions: string[];
  span: Span;
};

export type PrognosticRule = {
  kind: string;
  target: string;
  threshold: string | null;
  span: Span;
};

export type PrognosticsDecl = {
  kind: "PrognosticsDecl";
  name: string;
  rules: PrognosticRule[];
  span: Span;
};

export type MitigationBranch = {
  condition: string;
  actions: string[];
  span: Span;
};

export type MitigationDecl = {
  kind: "MitigationDecl";
  name: string;
  branches: MitigationBranch[];
  span: Span;
};

export type OperatingModeDecl = {
  kind: "OperatingModeDecl";
  name: string;
  modeKind: string;
  span: Span;
};

export type MissionStepDecl = { name: string; span: Span };

export type MissionConstraintDecl = { constraint: string; span: Span };

export type MissionPlanDecl = {
  kind: "MissionPlanDecl";
  name: string;
  steps: MissionStepDecl[];
  constraints: MissionConstraintDecl[];
  span: Span;
};

export type ResiliencePolicyDecl = {
  kind: "ResiliencePolicyDecl";
  name: string;
  strategies: string[];
  span: Span;
};

export type AssuranceCaseDecl = {
  kind: "AssuranceCaseDecl";
  name: string;
  evidence: string[];
  span: Span;
};

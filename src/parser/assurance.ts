/**
 * Mission assurance declaration parsing helpers for the TypeScript parser.
 * @module
 */

import type {
  AnomalyDetectorDecl,
  AnomalyHandlerDecl,
  AssuranceCaseDecl,
  ExpectedBehavior,
  KnowledgeComponent,
  KnowledgeDependency,
  KnowledgeModelDecl,
  MitigationBranch,
  MitigationDecl,
  MissionConstraintDecl,
  MissionPlanDecl,
  MissionStepDecl,
  OperatingModeDecl,
  PrognosticRule,
  PrognosticsDecl,
  ResiliencePolicyDecl,
  StateEstimatorDecl,
} from "../assurance_decl.js";
import type { Span } from "../ast/nodes.js";
import type { Token } from "../lexer/index.js";

export class ParseError extends Error {
  constructor(
    message: string,
    readonly line: number,
    readonly column: number,
  ) {
    super(message);
    this.name = "ParseError";
  }
}

export type AssuranceParseCtx = {
  advance(): Token;
  peek(): Token;
  previous(): Token;
  check(type: string): boolean;
  match(...types: string[]): boolean;
  expect(type: string, message: string): Token;
  parseLabel(message: string): string;
  parseDottedName(message: string): string;
  spanFrom(start: Token, end: Token): Span;
};

function parseComparisonOp(ctx: AssuranceParseCtx): string {
  if (ctx.check("GTE")) {
    ctx.advance();
    return ">=";
  }
  if (ctx.check("GT")) {
    ctx.advance();
    return ">";
  }
  if (ctx.check("LTE")) {
    ctx.advance();
    return "<=";
  }
  if (ctx.check("LT")) {
    ctx.advance();
    return "<";
  }
  if (ctx.check("EQ")) {
    ctx.advance();
    return "==";
  }
  const t = ctx.peek();
  throw new ParseError("Expected comparison operator", t.line, t.column);
}

function parseThresholdValue(ctx: AssuranceParseCtx): string {
  let threshold: string;
  if (ctx.check("TRUE")) {
    ctx.advance();
    threshold = "true";
  } else if (ctx.check("FALSE")) {
    ctx.advance();
    threshold = "false";
  } else if (ctx.check("NUMBER") || ctx.check("UNIT_LITERAL") || ctx.check("IDENT")) {
    threshold = ctx.advance().lexeme;
  } else {
    const t = ctx.peek();
    throw new ParseError("Expected threshold value", t.line, t.column);
  }
  if (ctx.check("IDENT")) {
    threshold += ` ${ctx.advance().lexeme}`;
    if (ctx.check("SLASH")) {
      threshold += ctx.advance().lexeme;
      if (ctx.check("IDENT")) threshold += ctx.advance().lexeme;
    }
  } else if (ctx.check("PERCENT")) {
    threshold += ctx.advance().lexeme;
  }
  return threshold;
}

function parseBracketNameList(ctx: AssuranceParseCtx): string[] {
  ctx.expect("LBRACKET", "Expected '['");
  const items: string[] = [];
  while (!ctx.check("RBRACKET") && !ctx.check("EOF")) {
    items.push(ctx.parseDottedName("Expected name in list"));
    if (ctx.check("COMMA")) ctx.advance();
    else break;
  }
  ctx.expect("RBRACKET", "Expected ']'");
  return items;
}

function parseActionStatement(ctx: AssuranceParseCtx): string {
  const parts: string[] = [];
  while (!ctx.check("SEMICOLON") && !ctx.check("RBRACE") && !ctx.check("EOF")) {
    parts.push(ctx.advance().lexeme);
    if (ctx.check("DOT")) parts.push(ctx.advance().lexeme);
    if (ctx.check("LPAREN")) {
      parts.push(ctx.advance().lexeme);
      while (!ctx.check("RPAREN") && !ctx.check("EOF")) parts.push(ctx.advance().lexeme);
      if (ctx.check("RPAREN")) parts.push(ctx.advance().lexeme);
    }
  }
  if (ctx.check("SEMICOLON")) ctx.advance();
  return parts.join("");
}

export function parseKnowledgeModel(ctx: AssuranceParseCtx): KnowledgeModelDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected knowledge model name");
  ctx.expect("LBRACE", "Expected '{' after knowledge_model name");
  const components: KnowledgeComponent[] = [];
  const dependencies: KnowledgeDependency[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "component") {
      ctx.advance();
      const comp = ctx.parseLabel("Expected component name");
      ctx.expect("SEMICOLON", "Expected ';' after component");
      components.push({ name: comp, span: ctx.spanFrom(start, ctx.previous()) });
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "dependency") {
      ctx.advance();
      const capability = ctx.parseLabel("Expected dependency capability");
      ctx.expect("REQUIRES", "Expected 'requires' in dependency");
      const requires = parseBracketNameList(ctx);
      ctx.expect("SEMICOLON", "Expected ';' after dependency");
      dependencies.push({ capability, requires, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected component or dependency in knowledge_model", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close knowledge_model");
  return { kind: "KnowledgeModelDecl", name, components, dependencies, span: ctx.spanFrom(start, end) };
}

export function parseStateEstimator(ctx: AssuranceParseCtx): StateEstimatorDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected state_estimator name");
  ctx.expect("LBRACE", "Expected '{' after state_estimator name");
  let inputs: string[] = [];
  let outputType = "";
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "inputs") {
      ctx.advance();
      inputs = parseBracketNameList(ctx);
      ctx.expect("SEMICOLON", "Expected ';' after inputs");
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "output") {
      ctx.advance();
      outputType = ctx.parseLabel("Expected output type");
      ctx.expect("SEMICOLON", "Expected ';' after output");
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected inputs or output in state_estimator", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close state_estimator");
  return { kind: "StateEstimatorDecl", name, inputs, outputType, span: ctx.spanFrom(start, end) };
}

export function parseAnomalyDetector(ctx: AssuranceParseCtx): AnomalyDetectorDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected anomaly_detector name");
  ctx.expect("LBRACE", "Expected '{' after anomaly_detector name");
  const expected: ExpectedBehavior[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "expected") {
      ctx.advance();
      const metric = ctx.parseDottedName("Expected metric path");
      const operator = parseComparisonOp(ctx);
      const threshold = parseThresholdValue(ctx);
      ctx.expect("SEMICOLON", "Expected ';' after expected rule");
      expected.push({ metric, operator, threshold, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected 'expected' rule in anomaly_detector", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close anomaly_detector");
  return { kind: "AnomalyDetectorDecl", name, expected, span: ctx.spanFrom(start, end) };
}

export function parseAnomalyHandler(ctx: AssuranceParseCtx): AnomalyHandlerDecl {
  const start = ctx.advance();
  ctx.expect("IDENT", "Expected 'anomaly' after on");
  const detector = ctx.parseLabel("Expected anomaly detector name");
  ctx.expect("IDENT", "Expected 'severity'");
  const severity = ctx.parseLabel("Expected severity level");
  ctx.expect("LBRACE", "Expected '{' after anomaly handler");
  const actions: string[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    actions.push(parseActionStatement(ctx));
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close anomaly handler");
  return { kind: "AnomalyHandlerDecl", detector, severity, actions, span: ctx.spanFrom(start, end) };
}

export function parsePrognostics(ctx: AssuranceParseCtx): PrognosticsDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected prognostics name");
  ctx.expect("LBRACE", "Expected '{' after prognostics name");
  const rules: PrognosticRule[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    const kind = ctx.parseLabel("Expected prognostic rule kind");
    const target =
      kind === "predict" || kind === "warn_if"
        ? ctx.parseDottedName("Expected prognostic target")
        : ctx.parseLabel("Expected prognostic target");
    let threshold: string | null = null;
    if (kind === "warn_if" && (ctx.check("LT") || ctx.check("LTE"))) {
      parseComparisonOp(ctx);
      threshold = parseThresholdValue(ctx);
    }
    ctx.expect("SEMICOLON", "Expected ';' after prognostic rule");
    rules.push({ kind, target, threshold, span: ctx.spanFrom(start, ctx.previous()) });
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close prognostics");
  return { kind: "PrognosticsDecl", name, rules, span: ctx.spanFrom(start, end) };
}

export function parseMitigation(ctx: AssuranceParseCtx): MitigationDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected mitigation name");
  ctx.expect("LBRACE", "Expected '{' after mitigation name");
  const branches: MitigationBranch[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IF")) {
      ctx.advance();
      const condition = ctx.parseDottedName("Expected mitigation condition");
      ctx.expect("LBRACE", "Expected '{' after if condition");
      const actions: string[] = [];
      while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
        actions.push(parseActionStatement(ctx));
      }
      ctx.expect("RBRACE", "Expected '}' to close if branch");
      branches.push({ condition, actions, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected 'if' branch in mitigation", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close mitigation");
  return { kind: "MitigationDecl", name, branches, span: ctx.spanFrom(start, end) };
}

export function parseAssuranceCase(ctx: AssuranceParseCtx): AssuranceCaseDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected assurance_case name");
  ctx.expect("LBRACE", "Expected '{' after assurance_case name");
  const evidence: string[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "evidence") {
      ctx.advance();
      evidence.push(ctx.parseDottedName("Expected evidence source"));
      ctx.expect("SEMICOLON", "Expected ';' after evidence");
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected evidence in assurance_case", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close assurance_case");
  return { kind: "AssuranceCaseDecl", name, evidence, span: ctx.spanFrom(start, end) };
}

export function parseResiliencePolicy(ctx: AssuranceParseCtx): ResiliencePolicyDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected resilience_policy name");
  ctx.expect("LBRACE", "Expected '{' after resilience_policy name");
  const strategies: string[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "strategy") {
      ctx.advance();
      strategies.push(ctx.parseLabel("Expected strategy name"));
      ctx.expect("SEMICOLON", "Expected ';' after strategy");
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected strategy in resilience_policy", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close resilience_policy");
  return { kind: "ResiliencePolicyDecl", name, strategies, span: ctx.spanFrom(start, end) };
}

export function parseMissionPlan(ctx: AssuranceParseCtx): MissionPlanDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected mission_plan name");
  ctx.expect("LBRACE", "Expected '{' after mission_plan name");
  const steps: MissionStepDecl[] = [];
  const constraints: MissionConstraintDecl[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "step") {
      ctx.advance();
      const stepName = ctx.parseLabel("Expected step name");
      ctx.expect("SEMICOLON", "Expected ';' after step");
      steps.push({ name: stepName, span: ctx.spanFrom(start, ctx.previous()) });
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "constraint") {
      ctx.advance();
      const parts: string[] = [];
      while (!ctx.check("SEMICOLON") && !ctx.check("EOF")) parts.push(ctx.advance().lexeme);
      ctx.expect("SEMICOLON", "Expected ';' after constraint");
      constraints.push({ constraint: parts.join(" "), span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected step or constraint in mission_plan", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close mission_plan");
  return { kind: "MissionPlanDecl", name, steps, constraints, span: ctx.spanFrom(start, end) };
}

export function parseOperatingMode(ctx: AssuranceParseCtx): OperatingModeDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected operating_mode name");
  ctx.expect("LBRACE", "Expected '{' after operating_mode name");
  let modeKind = "normal";
  if (ctx.check("IDENT")) {
    modeKind = ctx.advance().lexeme;
    ctx.expect("SEMICOLON", "Expected ';' after mode kind");
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close operating_mode");
  return { kind: "OperatingModeDecl", name, modeKind, span: ctx.spanFrom(start, end) };
}

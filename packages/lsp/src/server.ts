#!/usr/bin/env node
/**
 * Spanda Language Server Protocol (LSP) server for editor integration.
 * @module
 */

import {
  createConnection,
  TextDocuments,
  Diagnostic,
  DiagnosticSeverity,
  ProposedFeatures,
  InitializeParams,
  TextDocumentSyncKind,
  CompletionItem,
  CompletionItemKind,
  TextDocumentPositionParams,
  DefinitionParams,
  HoverParams,
  Hover,
  MarkupKind,
  Location,
  DocumentFormattingParams,
  DocumentSymbol,
  SymbolKind,
  TextEdit,
  RenameParams,
  WorkspaceEdit,
} from "vscode-languageserver/node.js";
import { TextDocument } from "vscode-languageserver-textdocument";
import { spawnSync } from "node:child_process";
import { existsSync, unlinkSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

function repoRoot(): string {
  if (process.env.SPANDA_EXTENSION_ROOT?.trim()) {
    return process.env.SPANDA_EXTENSION_ROOT;
  }
  return join(dirname(fileURLToPath(import.meta.url)), "../../..");
}

type Span = { start: { line: number; column: number }; end: { line: number; column: number } };

type SpandaSymbol = {
  name: string;
  kind: string;
  span: Span;
  detail?: string;
  container?: string;
};

function cliPath(): string | null {
  // CliPath.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = cliPath();
  const release = join(repoRoot(), "target/release/spanda");
  const debug = join(repoRoot(), "target/debug/spanda");

  // continue when existsSync(release).
  if (existsSync(release)) return release;

  // continue when existsSync(debug).
  if (existsSync(debug)) return debug;

  const configured = process.env.SPANDA_CLI_PATH?.trim();
  if (configured && existsSync(configured)) return configured;

  const which = spawnSync("which", ["spanda"], { encoding: "utf-8" });
  if (which.status === 0 && which.stdout.trim()) return which.stdout.trim();
  return null;
}

type CliDiagnostic = { message: string; line: number; column: number };

type CompatItem = {
  message: string;
  line: number;
  column: number;
  severity: "pass" | "warning" | "error";
  category: string;
};

const COMM_KEYWORDS = [
  "message",
  "subscribe",
  "publish",
  "execute",
  "discover",
  "bus",
  "device",
  "request",
  "response",
  "feedback",
  "result",
  "qos",
  "reliable",
  "best_effort",
  "rate",
  "history",
  "deadline",
  "where",
  "includes",
  "receive",
  "telemetry",
  "faults",
] as const;

const TRANSPORTS = ["local", "ros2", "mqtt", "dds", "websocket", "sim"] as const;

const HARDWARE_PROFILES = [
  "RoverV1",
  "RoverV2",
  "JetsonOrin",
  "RaspberryPi5",
  "ESP32",
] as const;

const symbolScript = join(repoRoot(), "scripts/lsp-symbols.mts");
const symbolCache = new Map<string, SpandaSymbol[]>();

function runSymbols(args: string[]): unknown {
  // RunSymbols.
  //
  // Parameters:
  // - `args` — input value
  //
  // Returns:
  // `unknown`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = runSymbols(args);
  const result = spawnSync(process.execPath, ["--import", "tsx", symbolScript, ...args], {
    encoding: "utf-8",
    cwd: repoRoot(),
  });

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return null;
  }

  // Try the operation and handle failures below.
  try {
    return JSON.parse(result.stdout);
  } catch {
    return null;
  }
}

function refreshSymbolCache(uri: string, source: string): void {
  // RefreshSymbolCache.
  //
  // Parameters:
  // - `uri` — input value
  // - `source` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = refreshSymbolCache(uri, source);
  const tmp = join(repoRoot(), ".spanda-lsp-symbols.sd");
  writeFileSync(tmp, source);
  const parsed = runSymbols(["index", tmp]) as { symbols?: SpandaSymbol[] } | null;

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  symbolCache.set(uri, parsed?.symbols ?? []);
}

function lookupDefinition(uri: string, source: string, line: number, column: number): SpandaSymbol | null {
  // LookupDefinition.
  //
  // Parameters:
  // - `uri` — input value
  // - `source` — input value
  // - `line` — input value
  // - `column` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = lookupDefinition(uri, source, line, column);
  const tmp = join(repoRoot(), ".spanda-lsp-define.sd");
  writeFileSync(tmp, source);
  const parsed = runSymbols(["define", tmp, String(line), String(column)]) as {
    symbol?: SpandaSymbol | null;
  } | null;

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  return parsed?.symbol ?? null;
}

function lookupHover(source: string, line: number, column: number): string | null {
  // LookupHover.
  //
  // Parameters:
  // - `source` — input value
  // - `line` — input value
  // - `column` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = lookupHover(source, line, column);
  const tmp = join(repoRoot(), ".spanda-lsp-hover.sd");
  writeFileSync(tmp, source);
  const parsed = runSymbols(["hover", tmp, String(line), String(column)]) as {
    markdown?: string | null;
  } | null;

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  return parsed?.markdown ?? null;
}

function spanToRange(span: Span) {
  // SpanToRange.
  //
  // Parameters:
  // - `span` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = spanToRange(span);
  return {
    start: { line: Math.max(0, span.start.line - 1), character: Math.max(0, span.start.column - 1) },
    end: { line: Math.max(0, span.end.line - 1), character: Math.max(0, span.end.column) },
  };
}

function runCliJson(args: string[], source: string): unknown {
  // RunCliJson.
  //
  // Parameters:
  // - `args` — input value
  // - `source` — input value
  //
  // Returns:
  // `unknown`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = runCliJson(args, source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return null;
  }
  const tmp = join(repoRoot(), ".spanda-lsp-check.sd");
  writeFileSync(tmp, source);
  const result = spawnSync(bin, [...args, "--json", tmp], { encoding: "utf-8" });

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      diagnostics: [{ message: result.stderr || "CLI failed", line: 1, column: 1 }],
    };
  }
  return JSON.parse(result.stdout);
}

function checkSourceTs(source: string): CliDiagnostic[] {
  // CheckSourceTs.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `CliDiagnostic[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = checkSourceTs(source);
  const tmp = join(repoRoot(), ".spanda-lsp-ts-check.sd");
  writeFileSync(tmp, source);
  const script = join(repoRoot(), "scripts/lsp-ts-check.mts");
  const result = spawnSync(process.execPath, ["--import", "tsx", script, tmp], {
    encoding: "utf-8",
    cwd: repoRoot(),
  });

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return [{ message: result.stderr || "TypeScript check failed", line: 1, column: 1 }];
  }
  const parsed = JSON.parse(result.stdout) as { ok: boolean; diagnostics?: CliDiagnostic[] };
  return parsed.ok ? [] : (parsed.diagnostics ?? []);
}

function verificationSource(source: string): CompatItem[] {
  // VerificationSource.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `CompatItem[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = verificationSource(source);
  const parsed = runCliJson(["check", "--verification-json"], source) as {
    ok: boolean;
    verification?: CompatItem[];
  } | null;

  // continue when parsed?.verification?.length.
  if (!parsed?.verification?.length) {
    return [];
  }
  return parsed.verification
    .filter((item) => item.severity !== "pass")
    .map((item) => ({
      ...item,
      severity: item.severity === "info" ? "warning" : item.severity,
      category: item.category || "verification",
    }));
}

function checkSource(source: string): CliDiagnostic[] {
  // CheckSource.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `CliDiagnostic[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = checkSource(source);
  const parsed = runCliJson(["check"], source) as {
    ok: boolean;
    diagnostics?: CliDiagnostic[];
  } | null;

  // continue when parsed.
  if (parsed) {
    return parsed.ok ? [] : (parsed.diagnostics ?? []);
  }
  return checkSourceTs(source);
}

function formatSource(source: string): string | null {
  // FormatSource.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = formatSource(source);
  const parsed = runCliJson(["fmt"], source) as {
    ok: boolean;
    formatted?: string;
  } | null;
  return parsed?.formatted ?? null;
}

function symbolKindFor(kind: string): SymbolKind {
  // SymbolKindFor.
  //
  // Parameters:
  // - `kind` — input value
  //
  // Returns:
  // `SymbolKind`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = symbolKindFor(kind);
  switch (kind) {
    case "robot":
      return SymbolKind.Class;
    case "behavior":
    case "function":
      return SymbolKind.Function;
    case "sensor":
    case "actuator":
      return SymbolKind.Field;
    case "agent":
      return SymbolKind.Interface;
    default:
      return SymbolKind.Variable;
  }
}

function documentSymbols(uri: string, _source: string): DocumentSymbol[] {
  // DocumentSymbols.
  //
  // Parameters:
  // - `uri` — input value
  // - `_source` — input value
  //
  // Returns:
  // `DocumentSymbol[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = documentSymbols(uri, _source);
  const cached = symbolCache.get(uri) ?? [];
  return cached.map(
    (sym): DocumentSymbol => ({
      name: sym.name,
      kind: symbolKindFor(sym.kind),
      range: spanToRange(sym.span),
      selectionRange: spanToRange(sym.span),
      detail: sym.detail,
    }),
  );
}

function verifySource(source: string): CompatItem[] {
  // VerifySource.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `CompatItem[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = verifySource(source);
  const parsed = runCliJson(["verify"], source) as {
    ok: boolean;
    items?: CompatItem[];
    diagnostics?: CliDiagnostic[];
  } | null;

  // continue when parsed is falsy.
  if (!parsed) {
    return [];
  }

  // continue when parsed.items?.length.
  if (parsed.items?.length) {
    return parsed.items.filter((i) => i.severity !== "pass");
  }

  // continue when diagnostics is falsy.
  if (!parsed.ok && parsed.diagnostics) {
    return parsed.diagnostics.map((d) => ({
      ...d,
      severity: "error" as const,
      category: "error",
    }));
  }
  return [];
}

const connection = createConnection(ProposedFeatures.all);
const documents = new TextDocuments(TextDocument);

connection.onInitialize((_params: InitializeParams) => ({
  capabilities: {
    textDocumentSync: TextDocumentSyncKind.Incremental,
    completionProvider: {
      triggerCharacters: [" ", ".", ":"],
    },
    definitionProvider: true,
    hoverProvider: true,
    documentFormattingProvider: true,
    documentSymbolProvider: true,
    renameProvider: true,
  },
}));

function validate(textDocument: TextDocument): Diagnostic[] {
  // Validate input.
  //
  // Parameters:
  // - `textDocument` — input value
  //
  // Returns:
  // `Diagnostic[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = validate(textDocument);
  const source = textDocument.getText();
  refreshSymbolCache(textDocument.uri, source);
  const typeErrors = checkSource(source);
  const compatItems = verifySource(source);
  const verificationItems = verificationSource(source);
  const typeDiags = typeErrors.map(
    (d): Diagnostic => ({
      severity: DiagnosticSeverity.Error,
      range: {
        start: { line: Math.max(0, d.line - 1), character: Math.max(0, d.column - 1) },
        end: { line: Math.max(0, d.line - 1), character: Math.max(0, d.column) },
      },
      message: d.message,
      source: "spanda",
    }),
  );
  const compatDiags = [...compatItems, ...verificationItems].map((d): Diagnostic => {
    const severity =
      d.severity === "warning" ? DiagnosticSeverity.Warning : DiagnosticSeverity.Error;
    const prefix = d.category ? `[${d.category}] ` : "";
    return {
      severity,
      range: {
        start: { line: Math.max(0, d.line - 1), character: Math.max(0, d.column - 1) },
        end: { line: Math.max(0, d.line - 1), character: Math.max(0, d.column + 20) },
      },
      message: `${prefix}${d.message}`,
      source: d.category?.startsWith("kill") ? "spanda-verify" : "spanda-compat",
    };
  });
  return [...typeDiags, ...compatDiags];
}

function commCompletions(): CompletionItem[] {
  // CommCompletions.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // `CompletionItem[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = commCompletions();
  const kwItems = COMM_KEYWORDS.map(
    (label): CompletionItem => ({
      label,
      kind: CompletionItemKind.Keyword,
      detail: "Spanda communication",
    }),
  );
  const transportItems = TRANSPORTS.map(
    (label): CompletionItem => ({
      label,
      kind: CompletionItemKind.EnumMember,
      detail: "Transport",
    }),
  );
  return [...kwItems, ...transportItems];
}

function hardwareProfileCompletions(doc: TextDocument, position: { line: number; character: number }): CompletionItem[] {
  const linePrefix = doc.getText({
    start: { line: position.line, character: 0 },
    end: { line: position.line, character: position.character },
  });
  if (!/\bdeploy\b[\s\S]*\bto\s+[\w]*$/i.test(linePrefix)) {
    return [];
  }
  return HARDWARE_PROFILES.map(
    (label): CompletionItem => ({
      label,
      kind: CompletionItemKind.EnumMember,
      detail: "Hardware deploy target (spanda verify)",
      insertText: label,
    }),
  );
}

connection.onCompletion((params: TextDocumentPositionParams): CompletionItem[] => {
  const doc = documents.get(params.textDocument.uri);
  if (!doc) return commCompletions();
  const profileItems = hardwareProfileCompletions(doc, params.position);
  if (profileItems.length > 0) {
    return profileItems;
  }
  const cached = symbolCache.get(params.textDocument.uri) ?? [];
  const symbolItems = cached.map(
    (sym): CompletionItem => ({
      label: sym.name,
      kind: CompletionItemKind.Variable,
      detail: `${sym.kind}${sym.detail ? `: ${sym.detail}` : ""}`,
    }),
  );
  return [...symbolItems, ...commCompletions()];
});

connection.onDefinition((params: DefinitionParams): Location | null => {
  const doc = documents.get(params.textDocument.uri);
  if (!doc) return null;

  const sym = lookupDefinition(
    params.textDocument.uri,
    doc.getText(),
    params.position.line + 1,
    params.position.character + 1,
  );
  if (!sym) return null;

  return {
    uri: params.textDocument.uri,
    range: spanToRange(sym.span),
  };
});

connection.onHover((params: HoverParams): Hover | null => {
  const doc = documents.get(params.textDocument.uri);
  if (!doc) return null;

  const markdown = lookupHover(
    doc.getText(),
    params.position.line + 1,
    params.position.character + 1,
  );
  if (!markdown) return null;

  return {
    contents: {
      kind: MarkupKind.Markdown,
      value: markdown,
    },
  };
});

connection.onDocumentFormatting((params: DocumentFormattingParams): TextEdit[] | null => {
  const doc = documents.get(params.textDocument.uri);
  if (!doc) return null;
  const formatted = formatSource(doc.getText());
  if (!formatted) return null;
  const lastLine = doc.lineCount - 1;
  const lastCharacter = doc.getText().length - (doc.getText().lastIndexOf("\n") + 1);
  return [
    {
      range: {
        start: { line: 0, character: 0 },
        end: { line: lastLine, character: Math.max(0, lastCharacter) },
      },
      newText: formatted,
    },
  ];
});

connection.onDocumentSymbol((params) => {
  const doc = documents.get(params.textDocument.uri);
  if (!doc) return [];
  refreshSymbolCache(params.textDocument.uri, doc.getText());
  return documentSymbols(params.textDocument.uri, doc.getText());
});

connection.onRenameRequest((params: RenameParams): WorkspaceEdit | null => {
  const doc = documents.get(params.textDocument.uri);
  if (!doc) return null;
  const sym = lookupDefinition(
    params.textDocument.uri,
    doc.getText(),
    params.position.line + 1,
    params.position.character + 1,
  );
  if (!sym) return null;
  const source = doc.getText();
  const edits: TextEdit[] = [];
  const regex = new RegExp(`\\b${sym.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`, "g");
  let match: RegExpExecArray | null;
  while ((match = regex.exec(source)) !== null) {
    const start = doc.positionAt(match.index);
    const end = doc.positionAt(match.index + sym.name.length);
    edits.push({ range: { start, end }, newText: params.newName });
  }
  return { changes: { [params.textDocument.uri]: edits } };
});

documents.onDidChangeContent((change: { document: TextDocument }) => {
  connection.sendDiagnostics({ uri: change.document.uri, diagnostics: validate(change.document) });
});

documents.onDidClose((event: { document: TextDocument }) => {
  symbolCache.delete(event.document.uri);
  connection.sendDiagnostics({ uri: event.document.uri, diagnostics: [] });
});

documents.listen(connection);
connection.listen();

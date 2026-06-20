#!/usr/bin/env node
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

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "../../..");

type Span = { start: { line: number; column: number }; end: { line: number; column: number } };

type SpandaSymbol = {
  name: string;
  kind: string;
  span: Span;
  detail?: string;
  container?: string;
};

function cliPath(): string | null {
  const release = join(repoRoot, "target/release/spanda");
  const debug = join(repoRoot, "target/debug/spanda");
  if (existsSync(release)) return release;
  if (existsSync(debug)) return debug;
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

const symbolScript = join(repoRoot, "scripts/lsp-symbols.mts");
const symbolCache = new Map<string, SpandaSymbol[]>();

function runSymbols(args: string[]): unknown {
  const result = spawnSync(process.execPath, ["--import", "tsx", symbolScript, ...args], {
    encoding: "utf-8",
    cwd: repoRoot,
  });
  if (!result.stdout?.trim()) {
    return null;
  }
  try {
    return JSON.parse(result.stdout);
  } catch {
    return null;
  }
}

function refreshSymbolCache(uri: string, source: string): void {
  const tmp = join(repoRoot, ".spanda-lsp-symbols.sd");
  writeFileSync(tmp, source);
  const parsed = runSymbols(["index", tmp]) as { symbols?: SpandaSymbol[] } | null;
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  symbolCache.set(uri, parsed?.symbols ?? []);
}

function lookupDefinition(uri: string, source: string, line: number, column: number): SpandaSymbol | null {
  const tmp = join(repoRoot, ".spanda-lsp-define.sd");
  writeFileSync(tmp, source);
  const parsed = runSymbols(["define", tmp, String(line), String(column)]) as {
    symbol?: SpandaSymbol | null;
  } | null;
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  return parsed?.symbol ?? null;
}

function lookupHover(source: string, line: number, column: number): string | null {
  const tmp = join(repoRoot, ".spanda-lsp-hover.sd");
  writeFileSync(tmp, source);
  const parsed = runSymbols(["hover", tmp, String(line), String(column)]) as {
    markdown?: string | null;
  } | null;
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  return parsed?.markdown ?? null;
}

function spanToRange(span: Span) {
  return {
    start: { line: Math.max(0, span.start.line - 1), character: Math.max(0, span.start.column - 1) },
    end: { line: Math.max(0, span.end.line - 1), character: Math.max(0, span.end.column) },
  };
}

function runCliJson(args: string[], source: string): unknown {
  const bin = cliPath();
  if (!bin) {
    return null;
  }

  const tmp = join(repoRoot, ".spanda-lsp-check.sd");
  writeFileSync(tmp, source);
  const result = spawnSync(bin, [...args, "--json", tmp], { encoding: "utf-8" });
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  if (!result.stdout?.trim()) {
    return {
      ok: false,
      diagnostics: [{ message: result.stderr || "CLI failed", line: 1, column: 1 }],
    };
  }

  return JSON.parse(result.stdout);
}

function checkSourceTs(source: string): CliDiagnostic[] {
  const tmp = join(repoRoot, ".spanda-lsp-ts-check.sd");
  writeFileSync(tmp, source);
  const script = join(repoRoot, "scripts/lsp-ts-check.mts");
  const result = spawnSync(process.execPath, ["--import", "tsx", script, tmp], {
    encoding: "utf-8",
    cwd: repoRoot,
  });
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  if (!result.stdout?.trim()) {
    return [{ message: result.stderr || "TypeScript check failed", line: 1, column: 1 }];
  }

  const parsed = JSON.parse(result.stdout) as { ok: boolean; diagnostics?: CliDiagnostic[] };
  return parsed.ok ? [] : (parsed.diagnostics ?? []);
}

function checkSource(source: string): CliDiagnostic[] {
  const parsed = runCliJson(["check"], source) as {
    ok: boolean;
    diagnostics?: CliDiagnostic[];
  } | null;

  if (parsed) {
    return parsed.ok ? [] : (parsed.diagnostics ?? []);
  }

  return checkSourceTs(source);
}

function formatSource(source: string): string | null {
  const parsed = runCliJson(["fmt"], source) as {
    ok: boolean;
    formatted?: string;
  } | null;
  return parsed?.formatted ?? null;
}

function symbolKindFor(kind: string): SymbolKind {
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
  const parsed = runCliJson(["verify"], source) as {
    ok: boolean;
    items?: CompatItem[];
    diagnostics?: CliDiagnostic[];
  } | null;

  if (!parsed) {
    return [];
  }

  if (parsed.items?.length) {
    return parsed.items.filter((i) => i.severity !== "pass");
  }
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
  const source = textDocument.getText();
  refreshSymbolCache(textDocument.uri, source);
  const typeErrors = checkSource(source);
  const compatItems = verifySource(source);

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

  const compatDiags = compatItems.map((d): Diagnostic => {
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
      source: "spanda-compat",
    };
  });

  return [...typeDiags, ...compatDiags];
}

function commCompletions(): CompletionItem[] {
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

connection.onCompletion((params: TextDocumentPositionParams): CompletionItem[] => {
  const doc = documents.get(params.textDocument.uri);
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

#!/usr/bin/env node
import {
  createConnection,
  TextDocuments,
  Diagnostic,
  DiagnosticSeverity,
  ProposedFeatures,
  InitializeParams,
  TextDocumentSyncKind,
} from "vscode-languageserver/node.js";
import { TextDocument } from "vscode-languageserver-textdocument";
import { spawnSync } from "node:child_process";
import { existsSync, unlinkSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "../../..");

function cliPath(): string | null {
  const release = join(repoRoot, "target/release/spanda");
  const debug = join(repoRoot, "target/debug/spanda");
  if (existsSync(release)) return release;
  if (existsSync(debug)) return debug;
  return null;
}

type CliDiagnostic = { message: string; line: number; column: number };

function checkSource(source: string): CliDiagnostic[] {
  const bin = cliPath();
  if (!bin) {
    return [
      {
        message: "Rust CLI not built — run: npm run build:rust",
        line: 1,
        column: 1,
      },
    ];
  }

  const tmp = join(repoRoot, ".spanda-lsp-check.sd");
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["check", "--json", tmp], { encoding: "utf-8" });
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  if (!result.stdout?.trim()) {
    return [{ message: result.stderr || "CLI check failed", line: 1, column: 1 }];
  }

  const parsed = JSON.parse(result.stdout) as {
    ok: boolean;
    diagnostics?: CliDiagnostic[];
  };
  return parsed.ok ? [] : (parsed.diagnostics ?? []);
}

const connection = createConnection(ProposedFeatures.all);
const documents = new TextDocuments(TextDocument);

connection.onInitialize((_params: InitializeParams) => ({
  capabilities: {
    textDocumentSync: TextDocumentSyncKind.Incremental,
  },
}));

function validate(textDocument: TextDocument): Diagnostic[] {
  const diags = checkSource(textDocument.getText());
  return diags.map(
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
}

documents.onDidChangeContent((change) => {
  connection.sendDiagnostics({ uri: change.document.uri, diagnostics: validate(change.document) });
});

documents.onDidClose((event) => {
  connection.sendDiagnostics({ uri: event.document.uri, diagnostics: [] });
});

documents.listen(connection);
connection.listen();

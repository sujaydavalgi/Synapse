#!/usr/bin/env node
/** JSON readiness diagnostics for LSP when Rust CLI is unavailable. */
import { readFileSync } from "node:fs";
import { readinessDiagnostics } from "../src/readiness.js";

const path = process.argv[2];
if (!path) {
  console.log(JSON.stringify({ ok: false, items: [] }));
  process.exit(1);
}

try {
  const source = readFileSync(path, "utf-8");
  const items = readinessDiagnostics(source, { includeRuntime: true });
  console.log(JSON.stringify({ ok: true, items }));
} catch (err) {
  const message = err instanceof Error ? err.message : String(err);
  console.log(
    JSON.stringify({
      ok: false,
      items: [{ message, line: 1, column: 1, severity: "error", category: "readiness" }],
    }),
  );
}

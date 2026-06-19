export default function init() {
  return Promise.resolve();
}

export function wasm_check() {
  return {
    ok: false,
    diagnostics: [{ message: "WASM not built — run npm run build:wasm", line: 1, column: 1 }],
  };
}

export function wasm_run() {
  return {
    ok: false,
    diagnostics: [{ message: "WASM not built — run npm run build:wasm", line: 1, column: 1 }],
  };
}

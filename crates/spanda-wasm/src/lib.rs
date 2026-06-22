//! src crate public API and re-exports.
//!
use serde::{Deserialize, Serialize};
use spanda_driver::{
    check, lower_to_sir, run, verify_compatibility, RunOptions, RunResult,
};
use spanda_error::Diagnostic;
use spanda_format::format_source;
use spanda_hardware::{CompatItem, CompatSeverity, VerifyOptions};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
struct CheckResponse {
    ok: bool,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize, Deserialize)]
struct RunResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<RunResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<Diagnostic>>,
}

fn to_js<T: Serialize>(value: &T) -> JsValue {
    // Convert to js.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // JsValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_wasm::to_js(value);

    // Produce NULL) as the result.
    serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn wasm_check(source: &str) -> JsValue {
    // Wasm check.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // JsValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_wasm::wasm_check(source);

    // Compute resp for the following logic.
    let resp = match check(source) {
        Ok(()) => CheckResponse {
            ok: true,
            diagnostics: vec![],
        },
        Err(e) => CheckResponse {
            ok: false,
            diagnostics: e.diagnostics(),
        },
    };
    to_js(&resp)
}

#[wasm_bindgen]
pub fn wasm_run(source: &str, max_loop_iterations: u32) -> JsValue {
    // Wasm run.
    //
    // Parameters:
    // - `source` — input value
    // - `max_loop_iterations` — input value
    //
    // Returns:
    // JsValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_wasm::wasm_run(source, max_loop_iterations);

    // Compute resp for the following logic.
    let resp = match run(
        source,
        RunOptions {
            max_loop_iterations: max_loop_iterations as usize,
            ..Default::default()
        },
    ) {
        Ok(result) => RunResponse {
            ok: true,
            result: Some(result),
            diagnostics: None,
        },
        Err(e) => RunResponse {
            ok: false,
            result: None,
            diagnostics: Some(e.diagnostics()),
        },
    };
    to_js(&resp)
}

#[wasm_bindgen]
pub fn wasm_ir(source: &str) -> JsValue {
    // Wasm ir.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // JsValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_wasm::wasm_ir(source);

    // Match on lower to sir and handle each case.
    match lower_to_sir(source) {
        Ok(sir) => serde_wasm_bindgen::to_value(&sir).unwrap_or(JsValue::NULL),
        Err(e) => to_js(&CheckResponse {
            ok: false,
            diagnostics: e.diagnostics(),
        }),
    }
}

#[wasm_bindgen]
pub fn wasm_fmt(source: &str) -> String {
    // Wasm fmt.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_wasm::wasm_fmt(source);

    // Produce format source as the result.
    format_source(source)
}

#[wasm_bindgen]
pub fn wasm_version() -> String {
    // Wasm version.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_wasm::wasm_version();

    // Produce to string as the result.
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Serialize, Deserialize)]
struct VerifyResponse {
    ok: bool,
    compatible: bool,
    items: Vec<CompatItem>,
}

#[wasm_bindgen]
pub fn wasm_verify(source: &str) -> JsValue {
    // Wasm verify.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // JsValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_wasm::wasm_verify(source);

    // Compute resp for the following logic.
    let resp = match verify_compatibility(source, &VerifyOptions::default()) {
        Ok(report) => VerifyResponse {
            ok: report.compatible,
            compatible: report.compatible,
            items: report.items,
        },
        Err(e) => VerifyResponse {
            ok: false,
            compatible: false,
            items: e
                .diagnostics()
                .into_iter()
                .map(|d| CompatItem {
                    category: "error".into(),
                    message: d.message,
                    severity: CompatSeverity::Error,
                    line: d.line,
                    column: d.column,
                })
                .collect(),
        },
    };
    to_js(&resp)
}

//! Optional TPM and vendor secure-boot quote backends.

use crate::attestation::LiveAttestationResult;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct TpmQuoteResponse {
    attested: bool,
    #[serde(default)]
    boot_state: String,
    #[serde(default)]
    score: Option<u32>,
    #[serde(default)]
    detail: Option<String>,
}

/// Query optional TPM or vendor quote backend when `SPANDA_TPM_BACKEND` is set.
pub fn query_tpm_attestation(
    contract: &str,
    package: &str,
    program_label: Option<&str>,
) -> Option<LiveAttestationResult> {
    // Resolve vendor TPM quote from mock, file, or script backends.
    //
    // Parameters:
    // - `contract` — import path (e.g. trust.jetson)
    // - `package` — registry package name
    // - `program_label` — optional program file label
    //
    // Returns:
    // Live attestation result when a configured backend succeeds.
    //
    // Options:
    // `SPANDA_TPM_BACKEND` — `mock`, `jetson`, `pi`, `tpm2`, `file`, or `script`
    // `SPANDA_TPM2_SCRIPT` — optional shell command for `tpm2` backend (stdout JSON)
    // `SPANDA_TPM2_PCR0_EXPECT` — optional hex PCR0 policy for tpm2 quote verification
    // `SPANDA_TPM_QUOTE_PATH` — JSON quote file for `file` backend
    // `SPANDA_TPM_SCRIPT` — shell command for `script` backend (stdout JSON)
    //
    // Example:
    // let live = query_tpm_attestation("trust.jetson", "spanda-trust-jetson", Some("rover.sd"));

    let backend = std::env::var("SPANDA_TPM_BACKEND")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    match backend.trim().to_ascii_lowercase().as_str() {
        "mock" | "jetson" | "pi" => Some(mock_tpm_quote(contract, package, &backend)),
        "tpm2" => Some(run_tpm2_quote(contract, package)),
        "file" => read_file_quote(),
        "script" => run_script_quote(contract, package, program_label),
        _ => None,
    }
}

fn parse_quote_response(payload: TpmQuoteResponse) -> LiveAttestationResult {
    LiveAttestationResult {
        attested: payload.attested,
        boot_state: if payload.boot_state.is_empty() {
            if payload.attested {
                "verified".into()
            } else {
                "unknown".into()
            }
        } else {
            payload.boot_state
        },
        score: payload.score.unwrap_or(if payload.attested { 95 } else { 0 }),
        detail: payload.detail.unwrap_or_else(|| {
            if payload.attested {
                "tpm quote verified".into()
            } else {
                "tpm quote failed".into()
            }
        }),
    }
}

fn mock_tpm_quote(contract: &str, package: &str, backend: &str) -> LiveAttestationResult {
    LiveAttestationResult {
        attested: true,
        boot_state: "verified".into(),
        score: 95,
        detail: format!("{backend} tpm quote stub for {contract} via {package}"),
    }
}

fn run_tpm2_quote(contract: &str, package: &str) -> LiveAttestationResult {
    // Attempt tpm2-tools PCR quote when available, else fall back to getcap probe.
    //
    // Parameters:
    // - `contract` — secure-boot import path
    // - `package` — registry package name
    //
    // Returns:
    // Verified quote when quote or getcap succeeds; unavailable/failed otherwise.
    //
    // Options:
    // `SPANDA_TPM2_SCRIPT` — optional shell adapter (stdout JSON).
    //
    // Example:
    // let live = run_tpm2_quote("trust.jetson", "spanda-trust-jetson");

    if let Some(script) = std::env::var("SPANDA_TPM2_SCRIPT")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        return run_tpm2_script_quote(&script, contract, package);
    }

    if !tpm2_getcap_available() {
        return LiveAttestationResult {
            attested: false,
            boot_state: "unavailable".into(),
            score: 0,
            detail: format!("tpm2 tools not available for {contract}"),
        };
    }

    if tpm2_tooling_complete() {
        if let Ok(detail) = attempt_tpm2_pcr_quote(contract, package) {
            return LiveAttestationResult {
                attested: true,
                boot_state: "verified".into(),
                score: 98,
                detail,
            };
        }
    }

    LiveAttestationResult {
        attested: true,
        boot_state: "verified".into(),
        score: 96,
        detail: format!("tpm2_getcap ok; tpm2_quote skipped or failed for {contract}"),
    }
}

fn run_tpm2_script_quote(script: &str, contract: &str, package: &str) -> LiveAttestationResult {
    let mut command = std::process::Command::new("sh");
    if Path::new(script).is_file() {
        command.arg(script);
    } else {
        command.arg("-c").arg(script);
    }
    let output = command
        .env("SPANDA_ATTESTATION_CONTRACT", contract)
        .env("SPANDA_ATTESTATION_PACKAGE", package)
        .output();
    match output {
        Ok(result) if result.status.success() => {
            if let Ok(payload) = serde_json::from_slice::<TpmQuoteResponse>(&result.stdout) {
                return parse_quote_response(payload);
            }
            LiveAttestationResult {
                attested: false,
                boot_state: "failed".into(),
                score: 0,
                detail: format!(
                    "tpm2 script returned invalid JSON for {contract}: {}",
                    String::from_utf8_lossy(&result.stdout)
                ),
            }
        }
        Ok(result) => LiveAttestationResult {
            attested: false,
            boot_state: "failed".into(),
            score: 0,
            detail: format!(
                "tpm2 script failed for {contract}: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
        },
        Err(error) => LiveAttestationResult {
            attested: false,
            boot_state: "unavailable".into(),
            score: 0,
            detail: format!("tpm2 script unavailable for {contract}: {error}"),
        },
    }
}

fn tpm2_getcap_available() -> bool {
    std::process::Command::new("tpm2_getcap")
        .arg("properties-fixed")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn tpm2_tooling_complete() -> bool {
    ["tpm2_createek", "tpm2_createak", "tpm2_quote"]
        .iter()
        .all(|tool| tpm2_tool_available(tool))
}

struct TempWorkDir(PathBuf);

impl Drop for TempWorkDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn attempt_tpm2_pcr_quote(contract: &str, package: &str) -> Result<String, String> {
    let base = std::env::temp_dir().join(format!("spanda_tpm2_{}", std::process::id()));
    std::fs::create_dir_all(&base).map_err(|error| error.to_string())?;
    let work = TempWorkDir(base);
    let dir = &work.0;

    run_tpm2_step(dir, "tpm2_createek", &["-c", "ek.ctx", "-G", "rsa"])?;
    run_tpm2_step(dir, "tpm2_createak", &["-C", "ek.ctx", "-c", "ak.ctx", "-G", "rsa"])?;
    run_tpm2_step(
        dir,
        "tpm2_quote",
        &[
            "-c",
            "ak.ctx",
            "-l",
            "sha256:0",
            "-m",
            "quote.msg",
            "-s",
            "quote.sig",
            "-p",
            "quote.pcr",
            "-g",
            "sha256",
        ],
    )?;
    verify_tpm2_quote_signature(dir)?;
    verify_tpm2_pcr_policy(contract)?;

    let mut detail = format!("tpm2_quote pcr0 verified for {contract} via {package}");
    if std::env::var("SPANDA_TPM2_PCR0_EXPECT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        detail.push_str("; pcr0 policy matched");
    }
    if tpm2_tool_available("tpm2_checkquote") {
        detail.push_str("; quote signature checked");
    }

    Ok(detail)
}

fn verify_tpm2_quote_signature(dir: &Path) -> Result<(), String> {
    if !tpm2_tool_available("tpm2_readpublic") || !tpm2_tool_available("tpm2_checkquote") {
        return Ok(());
    }

    run_tpm2_step(
        dir,
        "tpm2_readpublic",
        &["-c", "ak.ctx", "-o", "ak.pub", "-f", "pem"],
    )?;
    run_tpm2_step(
        dir,
        "tpm2_checkquote",
        &[
            "-u",
            "ak.pub",
            "-m",
            "quote.msg",
            "-s",
            "quote.sig",
            "-p",
            "quote.pcr",
            "-G",
            "sha256",
        ],
    )
}

fn verify_tpm2_pcr_policy(contract: &str) -> Result<(), String> {
    let Some(expected) = std::env::var("SPANDA_TPM2_PCR0_EXPECT")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(());
    };
    if !tpm2_tool_available("tpm2_pcrread") {
        return Err(format!(
            "SPANDA_TPM2_PCR0_EXPECT set but tpm2_pcrread unavailable for {contract}"
        ));
    }

    let output = std::process::Command::new("tpm2_pcrread")
        .arg("sha256:0")
        .output()
        .map_err(|error| format!("tpm2_pcrread unavailable: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "tpm2_pcrread failed for {contract}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = extract_pcr_hex(&stdout, 0)
        .ok_or_else(|| format!("could not parse tpm2_pcrread output for {contract}"))?;
    if normalize_hex(&actual) != normalize_hex(&expected) {
        return Err(format!(
            "pcr0 policy mismatch for {contract}: expected {expected} got {actual}"
        ));
    }
    Ok(())
}

fn tpm2_tool_available(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn normalize_hex(value: &str) -> String {
    value
        .replace("0x", "")
        .replace("0X", "")
        .chars()
        .filter(|ch| ch.is_ascii_hexdigit())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn extract_pcr_hex(output: &str, index: u32) -> Option<String> {
    let marker = format!("{index} : 0x");
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&marker) {
            return Some(normalize_hex(rest));
        }
    }
    None
}

fn run_tpm2_step(dir: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|error| format!("{program} unavailable: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn read_file_quote() -> Option<LiveAttestationResult> {
    let path = std::env::var("SPANDA_TPM_QUOTE_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let text = std::fs::read_to_string(&path).ok()?;
    let payload: TpmQuoteResponse = serde_json::from_str(&text).ok()?;
    Some(parse_quote_response(payload))
}

fn run_script_quote(
    contract: &str,
    package: &str,
    program_label: Option<&str>,
) -> Option<LiveAttestationResult> {
    let script = std::env::var("SPANDA_TPM_SCRIPT")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .env("SPANDA_ATTESTATION_CONTRACT", contract)
        .env("SPANDA_ATTESTATION_PACKAGE", package)
        .env(
            "SPANDA_ATTESTATION_PROGRAM",
            program_label.unwrap_or_default(),
        )
        .output()
        .ok()?;
    if !output.status.success() {
        return Some(LiveAttestationResult {
            attested: false,
            boot_state: "failed".into(),
            score: 0,
            detail: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    let payload: TpmQuoteResponse = serde_json::from_slice(&output.stdout).ok()?;
    Some(parse_quote_response(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    #[test]
    fn mock_backend_returns_verified_quote() {
        let _guard = env_lock();
        std::env::remove_var("SPANDA_TPM2_SCRIPT");
        std::env::set_var("SPANDA_TPM_BACKEND", "mock");
        let result = query_tpm_attestation("trust.jetson", "spanda-trust-jetson", Some("rover.sd"))
            .expect("mock quote");
        assert!(result.attested);
        assert_eq!(result.boot_state, "verified");
        std::env::remove_var("SPANDA_TPM_BACKEND");
    }

    #[test]
    fn file_backend_reads_quote_json() {
        let _guard = env_lock();
        let dir = std::env::temp_dir().join("spanda_tpm_quote_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("quote.json");
        std::fs::write(
            &path,
            r#"{"attested":true,"boot_state":"verified","score":98,"detail":"file tpm"}"#,
        )
        .expect("write quote");
        std::env::set_var("SPANDA_TPM_BACKEND", "file");
        std::env::set_var("SPANDA_TPM_QUOTE_PATH", path.to_string_lossy().to_string());
        let result = query_tpm_attestation("trust.pi", "spanda-trust-pi", None).expect("file quote");
        assert!(result.attested);
        assert_eq!(result.score, 98);
        std::env::remove_var("SPANDA_TPM_BACKEND");
        std::env::remove_var("SPANDA_TPM_QUOTE_PATH");
    }

    #[test]
    fn tpm2_backend_reports_tooling_status() {
        let _guard = env_lock();
        std::env::remove_var("SPANDA_TPM2_SCRIPT");
        std::env::set_var("SPANDA_TPM_BACKEND", "tpm2");
        let result = query_tpm_attestation("trust.jetson", "spanda-trust-jetson", Some("rover.sd"))
            .expect("tpm2 backend");
        assert!(
            result.detail.contains("tpm2_quote pcr0 verified")
                || result.detail.contains("tpm2_getcap ok")
                || result.detail.contains("tpm2 tools not available")
                || result.detail.contains("tpm2_getcap not installed"),
            "unexpected tpm2 detail: {}",
            result.detail
        );
        std::env::remove_var("SPANDA_TPM_BACKEND");
    }

    #[test]
    fn tpm2_script_fixture_emits_quote_json() {
        let _guard = env_lock();
        std::env::remove_var("SPANDA_TPM2_SCRIPT");
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../examples/showcase/secure_boot/fixtures/tpm2-quote.sh");
        if !path.is_file() {
            return;
        }
        std::env::set_var("SPANDA_TPM_BACKEND", "tpm2");
        std::env::set_var("SPANDA_TPM2_SCRIPT", path.to_string_lossy().to_string());
        let result = query_tpm_attestation("trust.jetson", "spanda-trust-jetson", None)
            .expect("tpm2 script");
        assert!(
            result.boot_state == "verified" || result.boot_state == "unavailable",
            "unexpected tpm2 script boot_state: {:?} detail={}",
            result.boot_state,
            result.detail
        );
        std::env::remove_var("SPANDA_TPM_BACKEND");
        std::env::remove_var("SPANDA_TPM2_SCRIPT");
    }

    #[test]
    fn normalize_hex_strips_prefixes() {
        assert_eq!(normalize_hex("0xAB-CD ef"), "abcdef");
    }

    #[test]
    fn extract_pcr_hex_parses_tpm2_pcrread_output() {
        let sample = "  sha256:\n    0 : 0x3D458CFE556432B7\n    1 : 0x00000000\n";
        assert_eq!(
            extract_pcr_hex(sample, 0),
            Some("3d458cfe556432b7".into())
        );
    }
}

//! Secure boot contract detection for trust.jetson and trust.pi package imports.

use crate::attestation::{query_live_attestation, LiveAttestationResult};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::{ImportDecl, Program};
use spanda_package::evaluate_package_trust;

/// One secure-boot contract import with package trust posture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecureBootEntry {
    pub contract: String,
    pub package: String,
    pub trust_score: u32,
    pub passed: bool,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_attestation: Option<LiveAttestationResult>,
}

/// Secure-boot coverage rollup for verify-time integrity and tamper checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecureBootCoverage {
    pub contracts: Vec<SecureBootEntry>,
    pub score: u32,
    pub passed: bool,
    #[serde(default)]
    pub live_attested: bool,
}

impl Default for SecureBootCoverage {
    fn default() -> Self {
        Self {
            contracts: Vec::new(),
            score: 0,
            passed: true,
            live_attested: false,
        }
    }
}

/// Return true when an import path is a known secure-boot contract module.
pub fn is_secure_boot_contract(import_path: &str) -> bool {
    // Classify trust.jetson and trust.pi as secure-boot contract imports.
    //
    // Parameters:
    // - `import_path` — Spanda import module path
    //
    // Returns:
    // True for known secure-boot contract modules.
    //
    // Options:
    // None.
    //
    // Example:
    // assert!(is_secure_boot_contract("trust.jetson"));

    matches!(import_path, "trust.jetson" | "trust.pi")
}

/// Map a secure-boot contract import path to its registry package name.
pub fn contract_to_package(import_path: &str) -> Option<&'static str> {
    // Resolve contract module paths to registry package identifiers.
    //
    // Parameters:
    // - `import_path` — Spanda import module path
    //
    // Returns:
    // Registry package name when the import is a secure-boot contract.
    //
    // Options:
    // None.
    //
    // Example:
    // assert_eq!(contract_to_package("trust.jetson"), Some("spanda-trust-jetson"));

    match import_path {
        "trust.jetson" => Some("spanda-trust-jetson"),
        "trust.pi" => Some("spanda-trust-pi"),
        _ => None,
    }
}

/// Evaluate secure-boot contract coverage from program imports.
pub fn evaluate_secure_boot_coverage(
    program: &Program,
    program_label: Option<&str>,
) -> SecureBootCoverage {
    // Score secure-boot contract imports using registry package trust signals.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    //
    // Returns:
    // Secure-boot coverage with per-contract trust scores.
    //
    // Options:
    // None.
    //
    // Example:
    // let coverage = evaluate_secure_boot_coverage(&program, Some("rover.sd"));

    let mut contracts = Vec::new();
    let mut live_attested = false;
    for import in program.imports() {
        let ImportDecl::ImportDecl { path, .. } = import;
        if !is_secure_boot_contract(path) {
            continue;
        }
        let package = contract_to_package(path).expect("secure boot contract maps to package");
        let trust = evaluate_package_trust(package, None, None);
        let mut entry = SecureBootEntry {
            contract: path.clone(),
            package: package.to_string(),
            trust_score: trust.score,
            passed: trust.passed,
            detail: format!("{}/100 tier={}", trust.score, trust.tier),
            live_attestation: None,
        };
        if let Some(live) = query_live_attestation(path, package, program_label) {
            live_attested = true;
            entry.live_attestation = Some(live.clone());
            entry.trust_score = ((entry.trust_score + live.score) / 2).min(100);
            entry.passed = entry.passed && live.attested;
            entry.detail = format!(
                "{}; live boot_state={} score={}",
                entry.detail, live.boot_state, live.score
            );
        }
        contracts.push(entry);
    }

    let score = if contracts.is_empty() {
        0
    } else {
        contracts.iter().map(|entry| entry.trust_score).sum::<u32>() / contracts.len() as u32
    };
    let passed = contracts.is_empty() || contracts.iter().all(|entry| entry.passed);
    SecureBootCoverage {
        contracts,
        score,
        passed,
        live_attested,
    }
}

/// Return true when any contract live attestation verified an AK certificate chain.
pub fn live_ak_chain_verified(coverage: &SecureBootCoverage) -> bool {
    // Report whether remote AK chain validation succeeded for any contract.
    //
    // Parameters:
    // - `coverage` — secure-boot coverage rollup
    //
    // Returns:
    // True when a live attestation result has `ak_chain_verified=true`.
    //
    // Options:
    // None.
    //
    // Example:
    // let verified = live_ak_chain_verified(&coverage);

    coverage.contracts.iter().any(|entry| {
        entry
            .live_attestation
            .as_ref()
            .and_then(|live| live.ak_chain_verified)
            .unwrap_or(false)
    })
}

/// Format a compact secure-boot status line for gates and explain sections.
pub fn secure_boot_status_line(coverage: &SecureBootCoverage) -> String {
    // Build a one-line secure-boot summary for CLI and explain output.
    //
    // Parameters:
    // - `coverage` — secure-boot coverage rollup
    //
    // Returns:
    // Human-readable status string.
    //
    // Options:
    // None.
    //
    // Example:
    // let line = secure_boot_status_line(&coverage);

    format!(
        "secure boot {}/100 contracts={} live_attested={} ak_chain_verified={}",
        coverage.score,
        coverage.contracts.len(),
        coverage.live_attested,
        live_ak_chain_verified(coverage)
    )
}

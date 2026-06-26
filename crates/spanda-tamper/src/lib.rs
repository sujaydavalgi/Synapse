//! Verify-time tamper and integrity analysis for Spanda programs.
//!
pub mod assurance;
pub mod attestation;
pub mod detect;
pub mod diagnosis;
pub mod fleet;
pub mod integrity;
pub mod policy;
pub mod remote_attestation;
pub mod runtime;
pub mod secure_boot;
pub mod tpm;

pub use assurance::{
    format_security_assurance_report, generate_security_assurance, SecurityAssuranceFormat,
    SecurityAssuranceReport, SecurityAssuranceSection,
};
pub use attestation::{apply_ak_chain_policy, query_live_attestation, LiveAttestationResult};
pub use detect::{
    format_tamper_report, generate_tamper_check, TamperFinding, TamperFormat, TamperReport,
    TamperSeverity, TamperStatus,
};
pub use diagnosis::{
    diagnose_tamper_trace, format_tamper_diagnosis, TamperDiagnosisFormat, TamperDiagnosisReport,
    TamperTimelineEvent,
};
pub use fleet::{
    build_fleet_tamper_report, correlate_fleet_tamper, correlate_fleet_tamper_traces,
    format_fleet_tamper_report, load_fleet_tamper_manifest,
    FleetTamperCorrelation, FleetTamperManifest, FleetTamperMember, FleetTamperReport,
    MemberTamperDiagnosis,
};
pub use integrity::{
    apply_agent_integrity, compare_agent_integrity, format_integrity_report,
    generate_integrity_report, AgentIntegrityActual, AgentIntegrityExpected,
    ArtifactIntegrityStatus, IntegrityArtifact, IntegrityFormat, IntegrityReport,
};
pub use policy::{
    actions_for_tamper_event, extract_tamper_policies, tamper_policy_coverage, TamperPolicySpec,
};
pub use runtime::{generate_runtime_tamper_check, MissionTrace, TraceFrame};
pub use remote_attestation::{
    attestation_trust_store_dir, validate_ak_cert_chain, AkCertChainValidation,
};
pub use secure_boot::{
    contract_to_package, evaluate_secure_boot_coverage, is_secure_boot_contract,
    live_ak_chain_verified, secure_boot_status_line, SecureBootCoverage, SecureBootEntry,
};

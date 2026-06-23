//! Hardware and certification compatibility verification.
//!
#[cfg(feature = "certify")]
use spanda_certify::verify_certification_proof;
use spanda_error::SpandaError;
use spanda_hardware::{
    verify_program_compatibility, CompatSeverity, CompatibilityReport, VerifyOptions,
};

use crate::compile::{compile, compile_with_registry};
use spanda_typecheck::ModuleRegistry;

pub fn verify_compatibility(
    source: &str,
    options: &VerifyOptions,
) -> Result<CompatibilityReport, SpandaError> {
    verify_compatibility_with_registry(source, options, None)
}

pub fn verify_compatibility_with_registry(
    source: &str,
    options: &VerifyOptions,
    registry: Option<&ModuleRegistry>,
) -> Result<CompatibilityReport, SpandaError> {
    // Compile source and verify hardware plus certification compatibility.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    // - `options` — verify targets, simulation, and strict certify flags
    // - `registry` — optional project module registry for package imports
    //
    // Returns:
    // Compatibility report with merged hardware and certification items.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = verify_compatibility_with_registry(source, &options, Some(&registry))?;

    let program = if let Some(registry) = registry {
        compile_with_registry(source, registry)?.program
    } else {
        compile(source)?.program
    };
    let mut report = verify_program_compatibility(&program, options);
    #[cfg(feature = "certify")]
    report
        .items
        .extend(verify_certification_proof(&program, options.strict_certify));
    report.compatible = !report
        .items
        .iter()
        .any(|item| item.severity == CompatSeverity::Error);
    Ok(report)
}

pub fn verify_compatibility_target(
    source: &str,
    target: Option<&str>,
) -> Result<CompatibilityReport, SpandaError> {
    // Verify compatibility against a single named hardware target.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    // - `target` — optional hardware profile name
    //
    // Returns:
    // Compatibility report for the selected target.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = verify_compatibility_target(source, Some("RoverV1"))?;

    verify_compatibility(
        source,
        &VerifyOptions {
            target: target.map(str::to_string),
            all_targets: false,
            simulate: false,
            strict_certify: false,
        },
    )
}

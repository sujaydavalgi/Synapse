//! Runtime certification gate before executing deploy-target programs.

use crate::ast::Program;
use crate::certify_verify::verify_certification_proof;
use crate::error::SpandaError;
use crate::hardware::CompatSeverity;

/// Fail fast when deploy/certify metadata does not satisfy runtime enforcement.
pub fn enforce_certification_runtime(program: &Program, strict: bool) -> Result<(), SpandaError> {
    // Block run/sim when certification proof checklist reports errors.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `strict` — treat checklist gaps as runtime errors
    //
    // Returns:
    // Ok when enforcement passes, or a SpandaError describing the first gap.
    //
    // Options:
    // None.
    //
    // Example:
    // enforce_certification_runtime(&program, true)?;

    if !strict {
        return Ok(());
    }

    let items = verify_certification_proof(program, true);
    let blocking = items
        .iter()
        .find(|item| item.severity == CompatSeverity::Error);
    if let Some(item) = blocking {
        return Err(SpandaError::Runtime {
            message: format!("certification runtime gate: {}", item.message),
            line: item.line,
        });
    }
    Ok(())
}

/// Return true when runtime certification enforcement is enabled via environment.
pub fn certification_runtime_enabled_from_env() -> bool {
    matches!(
        std::env::var("SPANDA_ENFORCE_CERTIFY").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

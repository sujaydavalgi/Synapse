//! Deploy target inference for readiness evaluation.

use spanda_ast::foundations::DeployDecl;
use spanda_ast::nodes::Program;

/// Return the first `deploy … to <target>` hardware profile name when present.
pub fn default_deploy_target(program: &Program) -> Option<String> {
    // Description:
    //     Default deploy target.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `default_deploy_target`.
    //
    // Example:

    //     let result = spanda_readiness::target::default_deploy_target(progra);

    let Program::Program { deployments, .. } = program;
    deployments.first().and_then(|deploy| {
        let DeployDecl::DeployDecl { targets, .. } = deploy;
        targets.first().cloned()
    })
}

/// Build readiness options from CLI-style flags and program deploy metadata.
pub fn readiness_options_from_flags(
    program: &Program,
    target_flag: Option<String>,
    include_runtime: bool,
    inject_health_faults: bool,
    simulate: bool,
    strict: bool,
) -> crate::types::ReadinessOptions {
    // Description:
    //     Readiness options from flags.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //     arget_flag: Option<String>
    //         Caller-supplied arget flag.
    //     include_runtime: bool
    //         Caller-supplied include runtime.
    //     inject_health_faults: bool
    //         Caller-supplied inject health faults.
    //     simulate: bool
    //         Caller-supplied simulate.
    //     stric: bool
    //         Caller-supplied stric.
    //
    // Outputs:
    //     result: crate::types::ReadinessOptions
    //         Return value from `readiness_options_from_flags`.
    //
    // Example:

    //     let result = spanda_readiness::target::readiness_options_from_flags(progra, arget_flag, include_runtime, inject_health_faults, simulate, stric);

    let target = target_flag.or_else(|| default_deploy_target(program));
    crate::types::ReadinessOptions {
        target,
        policy: None,
        simulate,
        strict,
        include_runtime,
        inject_health_faults,
        system_config: None,
        baseline_config: None,
    }
}

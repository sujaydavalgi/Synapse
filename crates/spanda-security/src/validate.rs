//! Static security validation and audit reporting for Spanda programs.

use crate::{
    is_known_capability, AuthenticationMode, EncryptionMode, IntegrityMode, TrustBoundaryKind,
    TrustBoundaryRegistry,
};
use spanda_ast::foundations::{
    IdentityDecl, SecretDecl, SecureBlockDecl, SecureCommPolicyDecl, TrustBoundaryDecl,
};
use spanda_ast::nodes::{Program, RobotDecl, TopicDecl};
use spanda_comm::{BusDecl, TransportKind};
use spanda_parser::parse;

/// Single security finding from static analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityFinding {
    pub severity: SecuritySeverity,
    pub message: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritySeverity {
    Error,
    Warning,
    Info,
}

/// Aggregated security report for `spanda security check` / `audit`.
#[derive(Debug, Clone, Default)]
pub struct SecurityReport {
    pub findings: Vec<SecurityFinding>,
}

impl SecurityReport {
    pub fn has_errors(&self) -> bool {
        // Description:
        //     Has errors.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: bool
        //         Return value from `has_errors`.
        //
        // Example:

        //     let result = spanda_security::validate::has_errors(&self);

        self.findings
            .iter()
            .any(|f| f.severity == SecuritySeverity::Error)
    }

    pub fn push_error(&mut self, message: impl Into<String>, line: u32, column: u32) {
        // Description:
        //     Push error.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     essage: impl Into<String>
        //         Caller-supplied essage.
        //     line: u32
        //         Caller-supplied line.
        //     column: u32
        //         Caller-supplied column.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_security::validate::push_error(&mut self, essage, line, column);

        self.findings.push(SecurityFinding {
            severity: SecuritySeverity::Error,
            message: message.into(),
            line,
            column,
        });
    }

    pub fn push_warning(&mut self, message: impl Into<String>, line: u32, column: u32) {
        // Description:
        //     Push warning.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     essage: impl Into<String>
        //         Caller-supplied essage.
        //     line: u32
        //         Caller-supplied line.
        //     column: u32
        //         Caller-supplied column.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_security::validate::push_warning(&mut self, essage, line, column);

        self.findings.push(SecurityFinding {
            severity: SecuritySeverity::Warning,
            message: message.into(),
            line,
            column,
        });
    }

    pub fn push_info(&mut self, message: impl Into<String>, line: u32, column: u32) {
        // Description:
        //     Push info.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     essage: impl Into<String>
        //         Caller-supplied essage.
        //     line: u32
        //         Caller-supplied line.
        //     column: u32
        //         Caller-supplied column.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_security::validate::push_info(&mut self, essage, line, column);

        self.findings.push(SecurityFinding {
            severity: SecuritySeverity::Info,
            message: message.into(),
            line,
            column,
        });
    }
}

/// Run static security validation on Spanda source.
pub fn security_check(source: &str) -> Result<SecurityReport, spanda_error::SpandaError> {
    // Description:
    //     Security check.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: Result<SecurityReport, spanda_error::SpandaError>
    //         Return value from `security_check`.
    //
    // Example:

    //     let result = spanda_security::validate::security_check(source);

    let tokens = spanda_lexer::tokenize(source)?;
    let program = parse(tokens)?;
    Ok(analyze_program(&program))
}

/// Produce an audit-oriented security report (includes informational events).
pub fn security_audit(source: &str) -> Result<SecurityReport, spanda_error::SpandaError> {
    // Description:
    //     Security audit.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: Result<SecurityReport, spanda_error::SpandaError>
    //         Return value from `security_audit`.
    //
    // Example:

    //     let result = spanda_security::validate::security_audit(source);

    let tokens = spanda_lexer::tokenize(source)?;
    let program = parse(tokens)?;
    let mut report = analyze_program(&program);
    collect_audit_hints(&program, &mut report);
    Ok(report)
}

/// Produce a security report for an already-parsed program.
pub fn security_analyze_program(program: &Program) -> SecurityReport {
    analyze_program(program)
}

fn analyze_program(program: &Program) -> SecurityReport {
    // Description:
    //     Analyze program.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //
    // Outputs:
    //     result: SecurityReport
    //         Return value from `analyze_program`.
    //
    // Example:

    //     let result = spanda_security::validate::analyze_program(progra);

    let mut report = SecurityReport::default();
    let Program::Program { robots, .. } = program;

    for robot in robots {
        analyze_robot(robot, &mut report);
    }
    report
}

fn analyze_robot(robot: &RobotDecl, report: &mut SecurityReport) {
    // Description:
    //     Analyze robot.
    //
    // Inputs:
    //     robo: &RobotDecl
    //         Caller-supplied robo.
    //     repor: &mut SecurityReport
    //         Caller-supplied repor.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_security::validate::analyze_robot(robo, repor);

    let RobotDecl::RobotDecl {
        identity,
        secrets,
        secure_comm,
        trust_boundaries,
        buses,
        topics,
        services,
        actions,
        permissions,
        ..
    } = robot;

    let mut boundaries = TrustBoundaryRegistry::new();
    for tb in trust_boundaries {
        let TrustBoundaryDecl::TrustBoundaryDecl { name, span } = tb;
        match name.parse::<TrustBoundaryKind>() {
            Ok(kind) => boundaries.declare(kind),
            Err(e) => report.push_error(e, span.start.line, span.start.column),
        }
    }

    let has_identity = identity.is_some();
    let has_key_or_cert = secrets.iter().any(secret_is_crypto_material);

    if let Some(sc) = secure_comm {
        validate_secure_comm(sc, report);
    }

    for bus in buses {
        validate_bus(bus, has_key_or_cert, report);
    }

    for topic in topics {
        validate_secure_endpoint_topic(topic, has_identity, has_key_or_cert, &boundaries, report);
    }

    for service in services {
        let spanda_ast::nodes::ServiceDecl::ServiceDecl { secure, span, .. } = service;
        if let Some(block) = secure {
            validate_secure_block_endpoint(
                block,
                has_identity,
                "service",
                span.start.line,
                span.start.column,
                report,
            );
        }
    }

    for action in actions {
        let spanda_ast::nodes::ActionDecl::ActionDecl {
            secure,
            span,
            action_type,
            ..
        } = action;
        if let Some(block) = secure {
            validate_secure_block_endpoint(
                block,
                has_identity,
                "action",
                span.start.line,
                span.start.column,
                report,
            );
            if action_type.as_deref() == Some("SafeAction")
                && boundaries.contains(TrustBoundaryKind::RobotToRobot)
                && block.encryption.as_deref() != Some("required")
            {
                report.push_error(
                    "SafeAction crossing robot_to_robot trust boundary requires encryption",
                    span.start.line,
                    span.start.column,
                );
            }
        }
    }

    if let Some(perms) = permissions {
        let spanda_ast::foundations::PermissionsDecl::PermissionsDecl { capabilities, .. } = perms;
        for secret in secrets {
            let SecretDecl::SecretDecl { name, span, .. } = secret;
            if !capabilities.iter().any(|c| c == "secret.read") {
                report.push_error(
                    format!("secret '{name}' used without secret.read capability in permissions"),
                    span.start.line,
                    span.start.column,
                );
            }
        }
    } else if !secrets.is_empty() {
        for secret in secrets {
            let SecretDecl::SecretDecl { name, span, .. } = secret;
            report.push_error(
                format!("secret '{name}' declared without secret.read capability"),
                span.start.line,
                span.start.column,
            );
        }
    }
}

fn secret_is_crypto_material(secret: &SecretDecl) -> bool {
    // Description:
    //     Secret is crypto material.
    //
    // Inputs:
    //     secre: &SecretDecl
    //         Caller-supplied secre.
    //
    // Outputs:
    //     result: bool
    //         Return value from `secret_is_crypto_material`.
    //
    // Example:

    //     let result = spanda_security::validate::secret_is_crypto_material(secre);

    let SecretDecl::SecretDecl { name, source, .. } = secret;
    name.contains("key")
        || name.contains("cert")
        || matches!(
            source,
            spanda_ast::foundations::SecretSourceDecl::File { .. }
        )
}

fn validate_secure_comm(sc: &SecureCommPolicyDecl, report: &mut SecurityReport) {
    // Description:
    //     Validate secure comm.
    //
    // Inputs:
    //     sc: &SecureCommPolicyDecl
    //         Caller-supplied sc.
    //     repor: &mut SecurityReport
    //         Caller-supplied repor.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_security::validate::validate_secure_comm(sc, repor);

    let SecureCommPolicyDecl::SecureCommPolicyDecl {
        encryption,
        authentication,
        integrity,
        span,
    } = sc;
    for (field, value) in [
        ("encryption", encryption.as_deref()),
        ("authentication", authentication.as_deref()),
        ("integrity", integrity.as_deref()),
    ] {
        if let Some(v) = value {
            if parse_mode(field, v).is_err() {
                report.push_error(
                    format!("invalid {field} mode '{v}' in secure_comm"),
                    span.start.line,
                    span.start.column,
                );
            }
        }
    }
}

fn validate_bus(bus: &BusDecl, has_key_or_cert: bool, report: &mut SecurityReport) {
    // Description:
    //     Validate bus.
    //
    // Inputs:
    //     bus: &BusDecl
    //         Caller-supplied bus.
    //     has_key_or_cer: bool
    //         Caller-supplied has key or cer.
    //     repor: &mut SecurityReport
    //         Caller-supplied repor.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_security::validate::validate_bus(bus, has_key_or_cer, repor);

    let BusDecl::BusDecl {
        name,
        transport,
        encryption,
        authentication,
        span,
        ..
    } = bus;

    if encryption.as_deref() == Some("required") && !has_key_or_cert {
        report.push_error(
            format!("encrypted bus '{name}' requires key or certificate in secrets"),
            span.start.line,
            span.start.column,
        );
    }

    if encryption.as_deref() == Some("required") && *transport == TransportKind::Local {
        report.push_warning(
            format!("bus '{name}' requires encryption on local transport — OK for dev, not for deployment"),
            span.start.line,
            span.start.column,
        );
    }

    if transport.supports_encryption()
        && encryption.is_none()
        && boundaries_default_robot_to_robot()
    {
        let _ = authentication;
    }
}

fn boundaries_default_robot_to_robot() -> bool {
    // Description:
    //     Boundaries default robot to robot.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `boundaries_default_robot_to_robot`.
    //
    // Example:

    //     let result = spanda_security::validate::boundaries_default_robot_to_robot();

    false
}

fn validate_secure_endpoint_topic(
    topic: &TopicDecl,
    has_identity: bool,
    has_key_or_cert: bool,
    boundaries: &TrustBoundaryRegistry,
    report: &mut SecurityReport,
) {
    // Description:
    //     Validate secure endpoint topic.
    //
    // Inputs:
    //     opic: &TopicDecl
    //         Caller-supplied opic.
    //     has_identity: bool
    //         Caller-supplied has identity.
    //     has_key_or_cer: bool
    //         Caller-supplied has key or cer.
    //     boundaries: &TrustBoundaryRegistry
    //         Caller-supplied boundaries.
    //     repor: &mut SecurityReport
    //         Caller-supplied repor.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_security::validate::validate_secure_endpoint_topic(opic, has_identity, has_key_or_cer, boundaries, repor);

    let TopicDecl::TopicDecl {
        name,
        message_type,
        secure,
        span,
        ..
    } = topic;

    if let Some(block) = secure {
        validate_secure_block_endpoint(
            block,
            has_identity,
            "topic",
            span.start.line,
            span.start.column,
            report,
        );

        if block.encryption.as_deref() == Some("required") && !has_key_or_cert {
            report.push_error(
                format!("encrypted topic '{name}' requires key or certificate config"),
                span.start.line,
                span.start.column,
            );
        }

        if !block.trusted_sources.is_empty() && block.reject_untrusted {
            report.push_info(
                format!("topic '{name}' rejects untrusted actuator sources"),
                span.start.line,
                span.start.column,
            );
        }
    }

    if message_type.contains("SafeAction")
        && boundaries.contains(TrustBoundaryKind::RobotToRobot)
        && secure.as_ref().and_then(|b| b.encryption.as_deref()) != Some("required")
    {
        report.push_error(
            format!("SafeAction topic '{name}' over robot-to-robot must require encryption"),
            span.start.line,
            span.start.column,
        );
    }
}

fn validate_secure_block_endpoint(
    block: &SecureBlockDecl,
    has_identity: bool,
    kind: &str,
    line: u32,
    column: u32,
    report: &mut SecurityReport,
) {
    // Description:
    //     Validate secure block endpoint.
    //
    // Inputs:
    //     block: &SecureBlockDecl
    //         Caller-supplied block.
    //     has_identity: bool
    //         Caller-supplied has identity.
    //     kind: &str
    //         Caller-supplied kind.
    //     line: u32
    //         Caller-supplied line.
    //     column: u32
    //         Caller-supplied column.
    //     repor: &mut SecurityReport
    //         Caller-supplied repor.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_security::validate::validate_secure_block_endpoint(block, has_identity, kind, line, column, repor);

    let needs_identity = block.signed
        || block.encryption.as_deref() == Some("required")
        || block.authentication.as_deref() == Some("mutual");

    if needs_identity && !has_identity {
        report.push_error(
            format!("secure {kind} requires robot identity declaration"),
            line,
            column,
        );
    }

    for (field, value) in [
        ("encryption", block.encryption.as_deref()),
        ("authentication", block.authentication.as_deref()),
        ("integrity", block.integrity.as_deref()),
    ] {
        if let Some(v) = value {
            if parse_mode(field, v).is_err() {
                report.push_error(
                    format!("invalid {field} mode '{v}' in secure block"),
                    line,
                    column,
                );
            }
        }
    }

    for cap in &block.requires {
        if !is_known_capability(cap) {
            report.push_error(
                format!("unknown capability '{cap}' in secure block"),
                block.span.start.line,
                block.span.start.column,
            );
        }
    }
}

fn parse_mode(field: &str, value: &str) -> Result<(), String> {
    // Description:
    //     Parse mode.
    //
    // Inputs:
    //     field: &str
    //         Caller-supplied field.
    //     value: &str
    //         Caller-supplied value.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `parse_mode`.
    //
    // Example:

    //     let result = spanda_security::validate::parse_mode(field, value);

    match field {
        "encryption" => value.parse::<EncryptionMode>().map(|_| ()),
        "authentication" => value.parse::<AuthenticationMode>().map(|_| ()),
        "integrity" => value.parse::<IntegrityMode>().map(|_| ()),
        _ => Ok(()),
    }
    .map_err(|e| e.to_string())
}

fn collect_audit_hints(program: &Program, report: &mut SecurityReport) {
    // Description:
    //     Collect audit hints.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //     repor: &mut SecurityReport
    //         Caller-supplied repor.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_security::validate::collect_audit_hints(progra, repor);

    let Program::Program { robots, .. } = program;
    for robot in robots {
        let RobotDecl::RobotDecl {
            identity,
            secure_comm,
            topics,
            ..
        } = robot;

        if secure_comm.is_some() {
            report.push_info("encryption policy declared via secure_comm", 1, 1);
        }
        if let Some(id) = identity {
            let IdentityDecl::IdentityDecl { fields, .. } = id;
            if fields.iter().any(|(k, _)| k == "cert") {
                report.push_info("certificate-backed identity configured", 1, 1);
            }
        }
        for topic in topics {
            let TopicDecl::TopicDecl { name, secure, .. } = topic;
            if secure.as_ref().and_then(|b| b.encryption.as_deref()) == Some("required") {
                report.push_info(format!("encryption enabled on topic '{name}'"), 1, 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_secure_topic_without_identity() {
        // Description:
        //     Detects secure topic without identity.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_security::validate::detects_secure_topic_without_identity();

        let source = r#"
robot R {
  topic cmd: Velocity publish on "/cmd" secure {
    encryption required;
    signed required;
  };
  behavior run() {}
}
"#;
        let report = security_check(source).unwrap();
        assert!(report.has_errors());
    }
}

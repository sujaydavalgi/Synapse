//! validation support for Spanda.
//!
use crate::error::{PackageError, PackageResult};
use crate::hardware_req::{validate_capability, CapabilityRequirements, HardwareRequirements};
use crate::manifest::PackageManifest;
use crate::registry::find_registry_entry;
use crate::safety::{SafetyLevel, SafetyMetadata};
use spanda_hardware::list_hardware_profiles;
use std::collections::HashSet;

/// Application-level permissions granted to the root package.
#[derive(Debug, Clone, Default)]
pub struct ApplicationPermissions {
    pub capabilities: HashSet<String>,
    pub hardware_targets: Vec<String>,
    pub allowed_safety_levels: HashSet<SafetyLevel>,
    pub allowed_licenses: HashSet<String>,
}

impl ApplicationPermissions {
    pub fn permissive() -> Self {
        // Permissive.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::validation::permissive();

        // Assemble the struct fields and return it.
        Self {
            capabilities: crate::hardware_req::known_capabilities()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            hardware_targets: list_hardware_profiles(),
            allowed_safety_levels: SafetyLevel::all().iter().copied().collect(),
            allowed_licenses: ["Apache-2.0", "MIT", "BSD-3-Clause"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub category: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn ok(&self) -> bool {
        // Ok.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.ok();

        // Produce !self as the result.
        !self
            .issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }

    pub fn push_error(&mut self, category: &str, message: impl Into<String>) {
        // Push error.
        //
        // Parameters:
        // - `self` — method receiver
        // - `category` — input value
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.push_error(category, message);

        // Append into self.
        self.issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            category: category.to_string(),
            message: message.into(),
        });
    }

    pub fn push_warning(&mut self, category: &str, message: impl Into<String>) {
        // Push warning.
        //
        // Parameters:
        // - `self` — method receiver
        // - `category` — input value
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.push_warning(category, message);

        // Compute msg for the following logic.
        let msg = message.into();
        self.warnings.push(msg.clone());
        self.issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            category: category.to_string(),
            message: msg,
        });
    }
}

/// Validate a package manifest before use.
pub fn validate_package(
    manifest: &PackageManifest,
    app_perms: &ApplicationPermissions,
) -> PackageResult<ValidationReport> {
    // Validate package.
    //
    // Parameters:
    // - `manifest` — input value
    // - `app_perms` — input value
    //
    // Returns:
    // PackageResult<ValidationReport>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_package(manifest, app_perms);

    // Create mutable report for accumulating results.
    let mut report = ValidationReport::default();
    validate_version(&manifest.package.version, &mut report);
    validate_capabilities(&manifest.capabilities, &mut report);
    validate_hardware_requirements(&manifest.requires_hardware, &mut report);
    validate_hardware_targets(manifest, app_perms, &mut report);
    validate_safety(&manifest.safety, app_perms, &mut report);
    validate_license(manifest, app_perms, &mut report);
    validate_adapter(manifest, &mut report);
    validate_dependencies(manifest, &mut report);
    check_capability_excess(&manifest.capabilities, app_perms, &mut report);

    // Take this path when report.ok().
    if report.ok() {
        Ok(report)
    } else if report
        .issues
        .iter()
        .any(|i| i.severity == ValidationSeverity::Error)
    {
        let msgs: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .map(|i| i.message.clone())
            .collect();
        Err(PackageError::Validation(msgs.join("; ")))
    } else {
        Ok(report)
    }
}

fn validate_version(version: &str, report: &mut ValidationReport) {
    // Validate version.
    //
    // Parameters:
    // - `version` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_version(version, report);

    // take this path when crate::dependency::parse version(version).is err().
    if crate::dependency::parse_version(version).is_err() {
        report.push_error("version", format!("invalid semver version '{version}'"));
    }
}

fn validate_capabilities(caps: &CapabilityRequirements, report: &mut ValidationReport) {
    // Validate capabilities.
    //
    // Parameters:
    // - `caps` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_capabilities(caps, report);

    // Validate each requested capability.
    for cap in caps.all() {
        // Handle the error returned from validate capability.
        if let Err(e) = validate_capability(cap) {
            report.push_warning("capabilities", e.to_string());
        }
    }
}

fn validate_hardware_requirements(req: &HardwareRequirements, report: &mut ValidationReport) {
    // Validate hardware requirements.
    //
    // Parameters:
    // - `req` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_hardware_requirements(req, report);

    // use mem when memory is present.

    // Emit output when memory provides a mem.
    if let Some(mem) = &req.memory {
        // Take this path when req.memory mb min().is none().
        if req.memory_mb_min().is_none() {
            report.push_error(
                "hardware",
                format!("could not parse memory requirement '{mem}'"),
            );
        }
    }

    // Emit output when gpu provides a gpu.
    if let Some(gpu) = &req.gpu {
        // Check membership before continuing.
        if req.gpu_tops_min().is_none() && !gpu.to_lowercase().contains("required") {
            report.push_warning(
                "hardware",
                format!("could not parse GPU requirement '{gpu}'"),
            );
        }
    }
}

fn validate_hardware_targets(
    manifest: &PackageManifest,
    app_perms: &ApplicationPermissions,
    report: &mut ValidationReport,
) {
    // Validate hardware targets.
    //
    // Parameters:
    // - `manifest` — input value
    // - `app_perms` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_hardware_targets(manifest, app_perms, report);

    // Compute known for the following logic.
    let known = list_hardware_profiles();

    // Process each target.
    for target in &manifest.hardware.targets {
        // Check membership before continuing.
        if !known.contains(target) {
            report.push_warning(
                "target",
                format!("unknown hardware target '{target}' — not in built-in profiles"),
            );
        }

        // Skip further work when hardware targets is empty.
        if !app_perms.hardware_targets.is_empty() && !app_perms.hardware_targets.contains(target) {
            report.push_error(
                "target",
                format!("target '{target}' not allowed by application permissions"),
            );
        }
    }
}

fn validate_safety(
    safety: &SafetyMetadata,
    app_perms: &ApplicationPermissions,
    report: &mut ValidationReport,
) {
    // Validate safety.
    //
    // Parameters:
    // - `safety` — input value
    // - `app_perms` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_safety(safety, app_perms, report);

    // Check membership before continuing.
    if !app_perms.allowed_safety_levels.contains(&safety.level) {
        report.push_error(
            "safety",
            format!(
                "safety level '{}' not permitted for this application",
                safety.level.as_str()
            ),
        );
    }

    // Take this path when safety.requires review.
    if safety.requires_review {
        report.push_warning(
            "safety",
            format!(
                "package requires manual review (level: {})",
                safety.level.as_str()
            ),
        );
    }

    // Take the branch when level equals SimulationOnly.
    if safety.can_control_actuators && safety.level == SafetyLevel::SimulationOnly {
        report.push_error(
            "safety",
            "simulation_only packages cannot control actuators".to_string(),
        );
    }
}

fn validate_license(
    manifest: &PackageManifest,
    app_perms: &ApplicationPermissions,
    report: &mut ValidationReport,
) {
    // Validate license.
    //
    // Parameters:
    // - `manifest` — input value
    // - `app_perms` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_license(manifest, app_perms, report);

    // use license when license is present.

    // Emit output when license provides a license.
    if let Some(license) = &manifest.package.license {
        // Skip further work when allowed licenses is empty.
        if !app_perms.allowed_licenses.is_empty()
            && !app_perms.allowed_licenses.contains(license)
            && manifest.license_compat.is_empty()
        {
            report.push_warning(
                "license",
                format!("license '{license}' may be incompatible with application policy"),
            );
        }
    }

    // Iterate over license compat.
    for compat in &manifest.license_compat {
        // Check membership before continuing.
        if !app_perms.allowed_licenses.contains(compat) {
            report.push_warning(
                "license",
                format!("declared license compatibility '{compat}' not in application allowlist"),
            );
        }
    }
}

fn validate_adapter(manifest: &PackageManifest, report: &mut ValidationReport) {
    // Validate adapter.
    //
    // Parameters:
    // - `manifest` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_adapter(manifest, report);

    // Process each require.
    for req in &manifest.adapter.requires {
        // Handle the error returned from validate capability.
        if let Err(e) = validate_capability(req) {
            report.push_warning("adapter", e.to_string());
        }
    }

    // Skip further work when provides is empty.
    if !manifest.adapter.provides.is_empty() && manifest.adapter.requires.is_empty() {
        report.push_warning(
            "adapter",
            "driver package provides symbols but declares no required capabilities",
        );
    }
}

fn validate_dependencies(manifest: &PackageManifest, report: &mut ValidationReport) {
    // Validate dependencies.
    //
    // Parameters:
    // - `manifest` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::validate_dependencies(manifest, report);

    // Iterate over all dependencies with destructured elements.
    for (name, spec) in manifest.all_dependencies() {
        // Take the branch when source kind equals Registry.
        if spec.source_kind() == crate::dependency::DependencySourceKind::Registry {
            // Take this path when find registry entry(name).is none().
            if find_registry_entry(name).is_none() {
                report.push_warning(
                    "dependencies",
                    format!("registry package '{name}' not in local registry stub"),
                );
            }

            // Handle the error returned from parse version req.
            if let Err(e) = spec.parse_version_req() {
                report.push_error(
                    "dependencies",
                    format!("invalid version constraint for '{name}': {e}"),
                );
            }
        }
    }
}

/// Warn when package capabilities exceed application permissions.
fn check_capability_excess(
    caps: &CapabilityRequirements,
    app_perms: &ApplicationPermissions,
    report: &mut ValidationReport,
) {
    // Check capability excess.
    //
    // Parameters:
    // - `caps` — input value
    // - `app_perms` — input value
    // - `report` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::validation::check_capability_excess(caps, app_perms, report);

    // Validate each requested capability.
    for cap in caps.uses.iter().chain(caps.required.iter()) {
        // Check membership before continuing.
        if !app_perms.capabilities.contains(cap) {
            let severity = if crate::hardware_req::is_high_risk_capability(cap) {
                "requires explicit approval"
            } else {
                "runtime may deny access"
            };
            report.push_warning(
                "capabilities",
                format!(
                    "package requires capability '{cap}' not granted to application — {severity}"
                ),
            );
        }
    }

    // Validate each requested capability.
    for cap in caps.all() {
        // Take this path when crate::hardware req::is high risk capability(cap).
        if crate::hardware_req::is_high_risk_capability(cap) {
            report.push_warning(
                "capabilities",
                format!(
                    "high-risk capability '{cap}' declared — ensure explicit operator approval"
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PackageManifest;
    use crate::safety::SafetyLevel;

    #[test]
    fn validates_safety_level_conflict() {
        // Validates safety level conflict.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::validation::validates_safety_level_conflict();

        let manifest = PackageManifest::parse_str(
            r#"
[package]
name = "test"
version = "0.1.0"

[safety]
level = "simulation_only"
can_control_actuators = true
"#,
        )
        .unwrap();
        let perms = ApplicationPermissions::permissive();
        let report = validate_package(&manifest, &perms).unwrap_err();
        assert!(report.to_string().contains("simulation_only"));
    }

    #[test]
    fn warns_on_capability_excess() {
        // Warns on capability excess.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::validation::warns_on_capability_excess();

        let manifest = PackageManifest::parse_str(
            r#"
[package]
name = "test"
version = "0.1.0"

[capabilities]
uses = ["camera.read"]
"#,
        )
        .unwrap();
        let mut perms = ApplicationPermissions::default();
        perms.capabilities.insert("lidar.read".into());
        perms
            .allowed_safety_levels
            .insert(SafetyLevel::Experimental);
        let report = validate_package(&manifest, &perms).unwrap();
        assert!(report.warnings.iter().any(|w| w.contains("camera.read")));
    }

    #[test]
    fn validates_hardware_metadata() {
        // Validates hardware metadata.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::validation::validates_hardware_metadata();

        let manifest = PackageManifest::parse_str(
            r#"
[package]
name = "test"
version = "0.1.0"

[requires_hardware]
memory = ">=2GB"
gpu = ">=1 TOPS"
sensors = ["Camera", "Lidar"]
"#,
        )
        .unwrap();
        let perms = ApplicationPermissions::permissive();
        let report = validate_package(&manifest, &perms).unwrap();
        assert!(report.ok());
    }
}

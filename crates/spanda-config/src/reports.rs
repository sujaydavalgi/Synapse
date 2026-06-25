//! Human-readable and JSON configuration reports.
//!
use crate::device_identity::traceability_rows;
use crate::drift::{detect_config_drift, format_drift_lines};
use crate::resolved::ResolvedSystemConfig;
use serde::Serialize;

/// Bundle of inspectable configuration reports.
#[derive(Debug, Clone, Serialize)]
pub struct ConfigReportBundle {
    pub resolved: ResolvedSummary,
    pub device_hierarchy: Vec<String>,
    pub logical_physical: LogicalPhysicalSummary,
    pub capabilities: CapabilitySummary,
    pub health: HealthSummary,
    pub trust_security: TrustSecuritySummary,
    pub network: NetworkSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedSummary {
    pub project: String,
    pub fleet_id: Option<String>,
    pub layers_applied: Vec<String>,
    pub fragments_loaded: Vec<String>,
    pub validation_passed: bool,
    pub error_count: usize,
    pub warning_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogicalPhysicalSummary {
    pub robots: usize,
    pub sensors: usize,
    pub actuators: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilitySummary {
    pub entries: Vec<CapabilityEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityEntry {
    pub device_id: String,
    pub robot_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthSummary {
    pub robots_with_policy: Vec<String>,
    pub robots_missing_policy: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrustSecuritySummary {
    pub identities: Vec<IdentityEntry>,
    pub untrusted_devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkSummary {
    pub device_count: usize,
    pub networked_devices: usize,
    pub traceability: Vec<crate::device_identity::TraceabilityRow>,
    pub endpoints: Vec<NetworkEndpointEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkEndpointEntry {
    pub device_id: String,
    pub logical_name: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub protocol: Option<String>,
    pub endpoint_url: Option<String>,
    pub trust_level: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityEntry {
    pub device_id: String,
    pub identity: String,
}

pub fn generate_report_bundle(resolved: &ResolvedSystemConfig) -> ConfigReportBundle {
    // Build the full configuration report bundle.
    //
    // Parameters:
    // - `resolved` — fully resolved system configuration
    //
    // Returns:
    // Structured report sections for CLI and JSON output.
    //
    // Options:
    // None.
    //
    // Example:
    // let bundle = generate_report_bundle(&resolved);

    let mut capability_entries = Vec::new();
    let mut untrusted = Vec::new();
    let mut identities = Vec::new();
    for (robot, _compute, device) in resolved.device_tree.all_devices() {
        capability_entries.push(CapabilityEntry {
            device_id: device.id.clone(),
            robot_id: robot.id.clone(),
            capabilities: device.capabilities.clone(),
        });
        if device.trusted == Some(false) {
            untrusted.push(device.id.clone());
        }
        if let Some(ref id) = device.identity {
            identities.push(IdentityEntry {
                device_id: device.id.clone(),
                identity: id.clone(),
            });
        }
    }

    let robot_ids = resolved.robot_ids();
    let mut with_policy = Vec::new();
    let mut missing_policy = Vec::new();
    for rid in &robot_ids {
        if resolved.health_policy_for(rid).is_some() {
            with_policy.push((*rid).to_string());
        } else {
            missing_policy.push((*rid).to_string());
        }
    }

    let traceability = traceability_rows(&resolved.device_registry);
    let mut endpoints = Vec::new();
    for device in &resolved.device_registry.devices {
        if device.is_networked() {
            endpoints.push(NetworkEndpointEntry {
                device_id: device.id.clone(),
                logical_name: device.logical_name.clone(),
                ip_address: device.ip_address.clone(),
                mac_address: device.mac_address.clone(),
                protocol: device.protocol.clone(),
                endpoint_url: device.endpoint_url.clone(),
                trust_level: device.trust_level.clone(),
            });
        }
    }

    ConfigReportBundle {
        resolved: ResolvedSummary {
            project: resolved.project_name().into(),
            fleet_id: resolved.fleet_id().map(str::to_owned),
            layers_applied: resolved.layers_applied.clone(),
            fragments_loaded: resolved.fragments_loaded.clone(),
            validation_passed: resolved.validation.passed,
            error_count: resolved.validation.error_count(),
            warning_count: resolved.validation.warning_count(),
        },
        device_hierarchy: resolved.device_tree.hierarchy_lines(),
        logical_physical: LogicalPhysicalSummary {
            robots: resolved.logical_map.robots.len(),
            sensors: resolved.logical_map.sensors.len(),
            actuators: resolved.logical_map.actuators.len(),
        },
        capabilities: CapabilitySummary {
            entries: capability_entries,
        },
        health: HealthSummary {
            robots_with_policy: with_policy,
            robots_missing_policy: missing_policy,
        },
        trust_security: TrustSecuritySummary {
            identities,
            untrusted_devices: untrusted,
        },
        network: NetworkSummary {
            device_count: resolved.device_registry.devices.len(),
            networked_devices: resolved.device_registry.network_devices().len(),
            traceability,
            endpoints,
        },
    }
}

pub fn format_report_text(bundle: &ConfigReportBundle) -> String {
    format_report_text_with_options(bundle, false)
}

pub fn format_report_text_with_options(bundle: &ConfigReportBundle, network_only: bool) -> String {
    // Render the report bundle as plain text for terminal output.
    //
    // Parameters:
    // - `bundle` — generated report bundle
    //
    // Returns:
    // Multi-section text report.
    //
    // Options:
    // None.
    //
    // Example:
    // println!("{}", format_report_text(&bundle));

    let mut out = String::new();
    if network_only {
        out.push_str("=== Network / Device Identity ===\n");
        out.push_str(&format!(
            "Devices: {} ({} networked)\n",
            bundle.network.device_count, bundle.network.networked_devices
        ));
        for entry in &bundle.network.endpoints {
            out.push_str(&format!(
                "  {} logical={:?} ip={:?} mac={:?} proto={:?}\n",
                entry.device_id,
                entry.logical_name,
                entry.ip_address,
                entry.mac_address,
                entry.protocol
            ));
            if let Some(ref url) = entry.endpoint_url {
                out.push_str(&format!("    endpoint: {url}\n"));
            }
        }
        out.push_str("\n=== Traceability ===\n");
        for row in &bundle.network.traceability {
            out.push_str(&format!(
                "  {} -> {:?} provider={:?} serial={:?}\n",
                row.device_id, row.logical_name, row.provider, row.serial
            ));
        }
        return out;
    }

    out.push_str("=== Resolved Configuration ===\n");
    out.push_str(&format!("Project: {}\n", bundle.resolved.project));
    if let Some(ref fleet) = bundle.resolved.fleet_id {
        out.push_str(&format!("Fleet: {fleet}\n"));
    }
    out.push_str(&format!(
        "Validation: {} ({} errors, {} warnings)\n",
        if bundle.resolved.validation_passed {
            "PASSED"
        } else {
            "FAILED"
        },
        bundle.resolved.error_count,
        bundle.resolved.warning_count
    ));
    if !bundle.resolved.layers_applied.is_empty() {
        out.push_str("\nLayers:\n");
        for layer in &bundle.resolved.layers_applied {
            out.push_str(&format!("  - {layer}\n"));
        }
    }
    if !bundle.resolved.fragments_loaded.is_empty() {
        out.push_str("\nFragments:\n");
        for frag in &bundle.resolved.fragments_loaded {
            out.push_str(&format!("  - {frag}\n"));
        }
    }
    out.push_str("\n=== Device Hierarchy ===\n");
    for line in &bundle.device_hierarchy {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("\n=== Logical / Physical Mapping ===\n");
    out.push_str(&format!(
        "Robots: {}, Sensors: {}, Actuators: {}\n",
        bundle.logical_physical.robots,
        bundle.logical_physical.sensors,
        bundle.logical_physical.actuators
    ));
    out.push_str("\n=== Capabilities ===\n");
    for entry in &bundle.capabilities.entries {
        out.push_str(&format!(
            "  {} @ {}: [{}]\n",
            entry.device_id,
            entry.robot_id,
            entry.capabilities.join(", ")
        ));
    }
    out.push_str("\n=== Health Policies ===\n");
    if bundle.health.robots_with_policy.is_empty() && bundle.health.robots_missing_policy.is_empty()
    {
        out.push_str("  (no robots configured)\n");
    } else {
        for r in &bundle.health.robots_with_policy {
            out.push_str(&format!("  [ok] {r}\n"));
        }
        for r in &bundle.health.robots_missing_policy {
            out.push_str(&format!("  [missing] {r}\n"));
        }
    }
    out.push_str("\n=== Trust / Security ===\n");
    for id in &bundle.trust_security.identities {
        out.push_str(&format!("  {} -> {}\n", id.device_id, id.identity));
    }
    for d in &bundle.trust_security.untrusted_devices {
        out.push_str(&format!("  [untrusted] {d}\n"));
    }
    out
}

pub fn config_drift_report(
    baseline: &ResolvedSystemConfig,
    current: &ResolvedSystemConfig,
) -> Vec<String> {
    // Compare two resolved configs and list drift.
    //
    // Parameters:
    // - `baseline` — reference configuration
    // - `current` — configuration under inspection
    //
    // Returns:
    // Drift description lines.
    //
    // Options:
    // None.
    //
    // Example:
    // let drift = config_drift_report(&base, &current);

    format_drift_lines(&detect_config_drift(baseline, current))
}

//! Connectivity declaration validation — moved from spanda-hardware to break the upward dep.
//!
//! spanda-connectivity-runtime already depends on spanda-ast and spanda-connectivity,
//! so the validation logic lives here rather than in the hardware layer.

use spanda_ast::foundations::{ConnectivityPolicyDecl, GeofenceDecl, RequiresConnectivityDecl};
use spanda_connectivity::{
    connectivity_key_to_profile_tokens, CompatItem, CompatSeverity, ConnectivityRequirement,
    HardwareProfile,
};
use std::collections::HashSet;

fn pass(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    // Build a passing compat item for the given category and message.
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Pass,
        line,
        column,
    }
}

fn warn(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    // Build a warning compat item for the given category and message.
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Warning,
        line,
        column,
    }
}

fn error(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    // Build an error compat item for the given category and message.
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Error,
        line,
        column,
    }
}

/// Verify `requires_connectivity` against a hardware profile's connectivity list and network metrics.
///
/// Parameters:
/// - `req` — the `requires_connectivity` declaration from the AST
/// - `profile` — hardware profile to verify against
///
/// Returns:
/// Vec of compat items (pass/warn/error) for each checked constraint.
///
/// Options:
/// None.
///
/// Example:
/// let items = verify_requires_connectivity(&req, &profile);
pub fn verify_requires_connectivity(
    req: &RequiresConnectivityDecl,
    profile: &HardwareProfile,
) -> Vec<CompatItem> {
    let RequiresConnectivityDecl::RequiresConnectivityDecl {
        channels,
        latency_ms_max,
        bandwidth_mbps_min,
        packet_loss_pct_max,
        span,
    } = req;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;
    let profile_set: HashSet<String> = profile.connectivity.iter().cloned().collect();

    for (key, level) in channels {
        if *level != ConnectivityRequirement::Required {
            continue;
        }
        let tokens = connectivity_key_to_profile_tokens(key);
        if tokens.is_empty() {
            items.push(warn(
                "connectivity",
                format!("Unknown connectivity key '{key}' in requires_connectivity"),
                line,
                column,
            ));
            continue;
        }
        let satisfied = tokens.iter().any(|t| profile_set.contains(*t));
        if satisfied {
            items.push(pass(
                "connectivity",
                format!("Required connectivity '{key}' present on '{}'", profile.name),
                line,
                column,
            ));
        } else {
            items.push(error(
                "connectivity",
                format!(
                    "Required connectivity '{key}' not on '{}' [{}]",
                    profile.name,
                    profile.connectivity.join(", ")
                ),
                line,
                column,
            ));
        }
    }

    if let Some(min_bw) = bandwidth_mbps_min {
        match profile.network_bandwidth_mbps {
            Some(bw) if bw >= *min_bw => items.push(pass(
                "connectivity",
                format!("Bandwidth {bw} Mbps meets connectivity requirement >= {min_bw} Mbps"),
                line,
                column,
            )),
            Some(bw) => items.push(error(
                "connectivity",
                format!(
                    "Connectivity bandwidth requirement {min_bw} Mbps exceeds target {bw} Mbps"
                ),
                line,
                column,
            )),
            None => items.push(warn(
                "connectivity",
                "Target bandwidth unknown — cannot verify connectivity bandwidth requirement",
                line,
                column,
            )),
        }
    }

    if let Some(max_lat) = latency_ms_max {
        match profile.network_latency_ms {
            Some(lat) if lat <= *max_lat => items.push(pass(
                "connectivity",
                format!("Latency {lat} ms meets connectivity requirement <= {max_lat} ms"),
                line,
                column,
            )),
            Some(lat) => items.push(error(
                "connectivity",
                format!(
                    "Connectivity latency requirement {max_lat} ms exceeded by target {lat} ms"
                ),
                line,
                column,
            )),
            None => items.push(warn(
                "connectivity",
                "Target latency unknown — cannot verify connectivity latency requirement",
                line,
                column,
            )),
        }
    }

    if let Some(max_loss) = packet_loss_pct_max {
        match profile.packet_loss_pct {
            Some(loss) if loss <= *max_loss => items.push(pass(
                "connectivity",
                format!("Packet loss {loss}% meets requirement <= {max_loss}%"),
                line,
                column,
            )),
            Some(loss) => items.push(error(
                "connectivity",
                format!("Packet loss {loss}% exceeds requirement <= {max_loss}%"),
                line,
                column,
            )),
            None => items.push(warn(
                "connectivity",
                "Target packet loss unknown — cannot verify packet_loss requirement",
                line,
                column,
            )),
        }
    }

    items
}

/// Validate geofence declaration geometry.
///
/// Parameters:
/// - `geofence` — the geofence AST declaration
///
/// Returns:
/// Vec of compat items (pass/error) for geometry validity.
///
/// Options:
/// None.
///
/// Example:
/// let items = validate_geofence(&geofence);
pub fn validate_geofence(geofence: &GeofenceDecl) -> Vec<CompatItem> {
    let GeofenceDecl::GeofenceDecl {
        name,
        center_lat,
        center_lon,
        radius_m,
        span,
    } = geofence;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;

    if !(-90.0..=90.0).contains(center_lat) {
        items.push(error(
            "geofence",
            format!("Geofence '{name}' center latitude {center_lat} out of range [-90, 90]"),
            line,
            column,
        ));
    } else if !(-180.0..=180.0).contains(center_lon) {
        items.push(error(
            "geofence",
            format!(
                "Geofence '{name}' center longitude {center_lon} out of range [-180, 180]"
            ),
            line,
            column,
        ));
    } else if *radius_m <= 0.0 {
        items.push(error(
            "geofence",
            format!("Geofence '{name}' radius must be positive"),
            line,
            column,
        ));
    } else {
        items.push(pass(
            "geofence",
            format!("Geofence '{name}' geometry valid"),
            line,
            column,
        ));
    }
    items
}

/// Validate connectivity failover policy link names.
///
/// Parameters:
/// - `policy` — the connectivity policy AST declaration
///
/// Returns:
/// Vec of compat items (pass/warn) for policy structure.
///
/// Options:
/// None.
///
/// Example:
/// let items = validate_connectivity_policy(&policy);
pub fn validate_connectivity_policy(policy: &ConnectivityPolicyDecl) -> Vec<CompatItem> {
    let ConnectivityPolicyDecl::ConnectivityPolicyDecl {
        name,
        preferred,
        fallback,
        emergency,
        span,
        ..
    } = policy;
    let line = span.start.line;
    let column = span.start.column;
    let mut items = vec![pass(
        "connectivity_policy",
        format!(
            "Connectivity policy '{name}' parsed: preferred={preferred}, fallback={fallback}"
        ),
        line,
        column,
    )];
    if preferred == fallback {
        items.push(warn(
            "connectivity_policy",
            format!("Policy '{name}' preferred and fallback are the same link"),
            line,
            column,
        ));
    }
    if let Some(em) = emergency {
        if em == preferred || em == fallback {
            items.push(warn(
                "connectivity_policy",
                format!("Policy '{name}' emergency link duplicates preferred or fallback"),
                line,
                column,
            ));
        }
    }
    items
}

//! Optional live-mode enrichments for HRI vendor package stubs.
//!
//! Real vendor SDKs ship in optional packages; these helpers gate simulated live
//! telemetry and spatial session metadata behind deployment env flags.
//!
use spanda_ast::nodes::UnitKind;
use spanda_runtime::providers::hri::SpatialSessionInfo;
use spanda_runtime::value::RuntimeValue;
use std::collections::HashMap;

fn env_enabled(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Return true when HealthKit-style wearable telemetry should be synthesized.
pub fn healthkit_live_enabled() -> bool {
    env_enabled("SPANDA_LIVE_HEALTHKIT") || env_enabled("SPANDA_LIVE_WEARABLE")
}

/// Return true when HoloLens-style spatial session metadata should be synthesized.
pub fn hololens_live_enabled() -> bool {
    env_enabled("SPANDA_LIVE_HOLOLENS")
        || env_enabled("SPANDA_HOLOLENS_SESSION")
        || env_enabled("SPANDA_SPATIAL_SESSION")
}

/// Return true when Apple Vision Pro spatial metadata should be synthesized.
pub fn vision_pro_live_enabled() -> bool {
    env_enabled("SPANDA_LIVE_VISION_PRO") || hololens_live_enabled()
}

/// Add HealthKit-oriented fields to wearable telemetry for smartwatch packages.
pub fn enrich_healthkit_telemetry(
    package: &str,
    device_id: &str,
    fields: &mut HashMap<String, RuntimeValue>,
) {
    if package != "spanda-smartwatch" || !healthkit_live_enabled() {
        return;
    }
    fields.insert(
        "backend".into(),
        RuntimeValue::String {
            value: "healthkit-stub".into(),
        },
    );
    fields.insert(
        "steps_today".into(),
        RuntimeValue::Number {
            value: 4_200.0,
            unit: UnitKind::None,
        },
    );
    fields.insert(
        "hrv_ms".into(),
        RuntimeValue::Number {
            value: 48.0,
            unit: UnitKind::None,
        },
    );
    fields.insert("workout_active".into(), RuntimeValue::Bool { value: false });
    fields.insert(
        "device_id".into(),
        RuntimeValue::String {
            value: device_id.to_string(),
        },
    );
}

/// Add HoloLens-oriented spatial session metadata.
pub fn enrich_hololens_session(package: &str, device_id: &str, info: &mut SpatialSessionInfo) {
    if package != "spanda-hololens" || !hololens_live_enabled() {
        return;
    }
    info.session_id = format!("hololens-live-{device_id}");
    info.device_id = device_id.to_string();
    info.active = true;
}

/// Add Vision Pro passthrough and hand-tracking flags to overlay poll payloads.
pub fn enrich_vision_pro_overlay(package: &str, fields: &mut HashMap<String, RuntimeValue>) {
    if package != "spanda-vision-pro" || !vision_pro_live_enabled() {
        return;
    }
    fields.insert(
        "backend".into(),
        RuntimeValue::String {
            value: "visionos-stub".into(),
        },
    );
    fields.insert("passthrough".into(), RuntimeValue::Bool { value: true });
    fields.insert("hand_tracking".into(), RuntimeValue::Bool { value: true });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthkit_enrichment_adds_backend_field() {
        std::env::set_var("SPANDA_LIVE_HEALTHKIT", "1");
        let mut fields = HashMap::new();
        enrich_healthkit_telemetry("spanda-smartwatch", "watch-001", &mut fields);
        assert!(fields.contains_key("backend"));
        std::env::remove_var("SPANDA_LIVE_HEALTHKIT");
    }
}

//! Package-scoped provider stubs registered when official packages are installed.
//!
use spanda_ast::nodes::UnitKind;
use spanda_audit::{sha256, Hash, LedgerBackend, MockLedgerBackend};
use spanda_runtime::providers::hri::{
    HriInputProvider, OverlayProvider, SpatialSessionInfo, SpatialSessionProvider,
    WearableTelemetryProvider,
};
use spanda_runtime::providers::traits::{
    CloudProvider, ConnectivityProvider, FleetProvider, LedgerProvider, MaintenanceProvider,
    NavigationProvider, PositioningProvider, SimulationProvider, SlamProvider, VisionProvider,
};
use spanda_runtime::providers::types::{
    ProviderError, ProviderId, ProviderMetadata, ProviderResult, ProviderSafetyLevel,
};
use spanda_runtime::robot_state::RobotState;
use spanda_runtime::value::{runtime_pose, RuntimeValue};
use std::collections::HashMap;

fn package_metadata(package: &str, name: &str, description: &str) -> ProviderMetadata {
    // Description:
    //     Package metadata.
    //
    // Inputs:
    //     package: &str
    //         Caller-supplied package.
    //     name: &str
    //         Caller-supplied name.
    //     description: &str
    //         Caller-supplied description.
    //
    // Outputs:
    //     result: ProviderMetadata
    //         Return value from `package_metadata`.
    //
    // Example:

    //     let result = spanda_providers::package_stubs::package_metadata(package, name, description);

    ProviderMetadata {
        id: ProviderId::new(package, name),
        description: description.into(),
        safety_level: ProviderSafetyLevel::Development,
        capabilities_required: Vec::new(),
        hardware_requirements: Vec::new(),
    }
}

/// Wireless connectivity stub for `spanda-wifi` / `spanda-ble` / `spanda-cellular` bootstrap.
pub struct ConnectivityPackageStub {
    package: &'static str,
    channel: &'static str,
}

impl ConnectivityPackageStub {
    pub fn wifi() -> Self {
        // Description:
        //     Wifi.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `wifi`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::wifi();

        Self {
            package: "spanda-wifi",
            channel: "wifi",
        }
    }

    pub fn ble() -> Self {
        // Description:
        //     Ble.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `ble`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::ble();

        Self {
            package: "spanda-ble",
            channel: "ble",
        }
    }

    pub fn cellular() -> Self {
        // Description:
        //     Cellular.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `cellular`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::cellular();

        Self {
            package: "spanda-cellular",
            channel: "cellular",
        }
    }
}

impl ConnectivityProvider for ConnectivityPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            self.package,
            "project",
            "Package-scoped connectivity stub (radio driver in transport shim)",
        )
    }

    fn connect(&mut self, channel: &str) -> ProviderResult<()> {
        // Description:
        //     Connect.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     channel: &str
        //         Caller-supplied channel.
        //
        // Outputs:
        //     result: ProviderResult<()>
        //         Return value from `connect`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::connect(&mut self, channel);

        let _ = channel;
        Ok(())
    }

    fn disconnect(&mut self, channel: &str) {
        // Description:
        //     Disconnect.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     channel: &str
        //         Caller-supplied channel.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::disconnect(&mut self, channel);

        let _ = channel;
    }

    fn is_connected(&self, channel: &str) -> bool {
        // Description:
        //     Is connected.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     channel: &str
        //         Caller-supplied channel.
        //
        // Outputs:
        //     result: bool
        //         Return value from `is_connected`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::is_connected(&self, channel);

        channel == self.channel
    }

    fn signal_strength_dbm(&self, channel: &str) -> Option<f64> {
        // Description:
        //     Signal strength dbm.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     channel: &str
        //         Caller-supplied channel.
        //
        // Outputs:
        //     result: Option<f64>
        //         Return value from `signal_strength_dbm`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::signal_strength_dbm(&self, channel);

        if channel == self.channel {
            Some(-55.0)
        } else {
            None
        }
    }
}

/// GPS positioning stub for `spanda-gps` package bootstrap.
pub struct GpsPositioningStub;

impl PositioningProvider for GpsPositioningStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            "spanda-gps",
            "project",
            "Package-scoped GPS positioning stub (UART driver in core shim)",
        )
    }

    fn read_fix(&mut self) -> RuntimeValue {
        // Description:
        //     Read fix.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `read_fix`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::read_fix(&mut self);

        RuntimeValue::Object {
            type_name: "GeoPoint".into(),
            fields: [
                (
                    "lat".into(),
                    RuntimeValue::Number {
                        value: 37.0,
                        unit: spanda_ast::nodes::UnitKind::None,
                    },
                ),
                (
                    "lon".into(),
                    RuntimeValue::Number {
                        value: -122.0,
                        unit: spanda_ast::nodes::UnitKind::None,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn accuracy_meters(&self) -> Option<f64> {
        // Description:
        //     Accuracy meters.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Option<f64>
        //         Return value from `accuracy_meters`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::accuracy_meters(&self);

        Some(5.0)
    }
}

/// Navigation stub for `spanda-nav` package bootstrap.
pub struct NavNavigationStub;

impl NavigationProvider for NavNavigationStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            "spanda-nav",
            "project",
            "Package-scoped navigation stub (Nav2 adapter in core shim)",
        )
    }

    fn navigate_to(&mut self, goal: RuntimeValue) -> ProviderResult<RuntimeValue> {
        // Description:
        //     Navigate to.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     goal: RuntimeValue
        //         Caller-supplied goal.
        //
        // Outputs:
        //     result: ProviderResult<RuntimeValue>
        //         Return value from `navigate_to`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::navigate_to(&mut self, goal);

        Ok(RuntimeValue::Object {
            type_name: "NavigationGoal".into(),
            fields: [("goal".into(), goal)].into_iter().collect(),
        })
    }

    fn cancel_navigation(&mut self) {
        // Description:
        //     Cancel navigation.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:
        // let result = spanda_providers::package_stubs::cancel_navigation(&mut self);
    }

    fn navigation_status(&self) -> RuntimeValue {
        // Description:
        //     Navigation status.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `navigation_status`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::navigation_status(&self);

        RuntimeValue::Object {
            type_name: "Trajectory".into(),
            fields: [(
                "status".into(),
                RuntimeValue::String {
                    value: "idle".into(),
                },
            )]
            .into_iter()
            .collect(),
        }
    }
}

/// SLAM stub for `spanda-slam` package bootstrap.
pub struct SlamPackageStub;

impl SlamProvider for SlamPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            "spanda-slam",
            "project",
            "Package-scoped SLAM stub (subprocess adapter in core shim)",
        )
    }

    fn localize(&mut self) -> ProviderResult<RuntimeValue> {
        // Description:
        //     Localize.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: ProviderResult<RuntimeValue>
        //         Return value from `localize`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::localize(&mut self);

        Ok(RuntimeValue::Object {
            type_name: "LocalizationEstimate".into(),
            fields: [
                ("pose".into(), runtime_pose(0.0, 0.0, 0.0, 0.0)),
                (
                    "confidence".into(),
                    RuntimeValue::Number {
                        value: 0.9,
                        unit: spanda_ast::nodes::UnitKind::None,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        })
    }

    fn update_map(&mut self, _sensor_frame: RuntimeValue) -> ProviderResult<RuntimeValue> {
        // Description:
        //     Update map.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     _sensor_frame: RuntimeValue
        //         Caller-supplied sensor frame.
        //
        // Outputs:
        //     result: ProviderResult<RuntimeValue>
        //         Return value from `update_map`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::update_map(&mut self, _sensor_frame);

        Ok(RuntimeValue::Object {
            type_name: "OccupancyGrid".into(),
            fields: [(
                "resolution".into(),
                RuntimeValue::Number {
                    value: 0.05,
                    unit: spanda_ast::nodes::UnitKind::None,
                },
            )]
            .into_iter()
            .collect(),
        })
    }

    fn export_map(&self) -> ProviderResult<RuntimeValue> {
        // Description:
        //     Export map.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderResult<RuntimeValue>
        //         Return value from `export_map`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::export_map(&self);

        Ok(RuntimeValue::Object {
            type_name: "OccupancyGrid".into(),
            fields: [(
                "resolution".into(),
                RuntimeValue::Number {
                    value: 0.05,
                    unit: spanda_ast::nodes::UnitKind::None,
                },
            )]
            .into_iter()
            .collect(),
        })
    }
}

/// Fleet orchestration stub for `spanda-fleet` package bootstrap.
pub struct FleetPackageStub;

impl FleetProvider for FleetPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            "spanda-fleet",
            "project",
            "Package-scoped fleet stub (orchestrator in spanda-fleet crate)",
        )
    }

    fn register_member(&mut self, member_id: &str, _metadata: RuntimeValue) -> ProviderResult<()> {
        // Description:
        //     Register member.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     ember_id: &str
        //         Caller-supplied ember id.
        //     _metadata: RuntimeValue
        //         Caller-supplied metadata.
        //
        // Outputs:
        //     result: ProviderResult<()>
        //         Return value from `register_member`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::register_member(&mut self, ember_id, _metadata);

        let _ = member_id;
        Ok(())
    }

    fn dispatch_task(
        &mut self,
        member_id: &str,
        task: RuntimeValue,
    ) -> ProviderResult<RuntimeValue> {
        // Description:
        //     Dispatch task.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     ember_id: &str
        //         Caller-supplied ember id.
        //     ask: RuntimeValue
        //         Caller-supplied ask.
        //
        // Outputs:
        //     result: ProviderResult<RuntimeValue>
        //         Return value from `dispatch_task`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::dispatch_task(&mut self, ember_id, ask);

        Ok(RuntimeValue::Object {
            type_name: "FleetTask".into(),
            fields: [
                (
                    "member".into(),
                    RuntimeValue::String {
                        value: member_id.into(),
                    },
                ),
                ("task".into(), task),
            ]
            .into_iter()
            .collect(),
        })
    }

    fn member_status(&self, member_id: &str) -> Option<RuntimeValue> {
        // Description:
        //     Member status.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     ember_id: &str
        //         Caller-supplied ember id.
        //
        // Outputs:
        //     result: Option<RuntimeValue>
        //         Return value from `member_status`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::member_status(&self, ember_id);

        Some(RuntimeValue::String {
            value: format!("{member_id}:idle"),
        })
    }
}

/// Ledger provider backed by the in-process mock chain (`spanda-ledger` package).
pub struct LedgerPackageStub {
    backend: MockLedgerBackend,
}

impl Default for LedgerPackageStub {
    fn default() -> Self {
        // Description:
        //     Provide the default value for this type.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `default`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::default();

        Self {
            backend: MockLedgerBackend::new(),
        }
    }
}

impl LedgerProvider for LedgerPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            "spanda-ledger",
            "project",
            "Mock ledger provider (anchors audit digests via spanda-audit)",
        )
    }

    fn append(&mut self, record: RuntimeValue) -> ProviderResult<String> {
        // Description:
        //     Append.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     record: RuntimeValue
        //         Caller-supplied record.
        //
        // Outputs:
        //     result: ProviderResult<String>
        //         Return value from `append`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::append(&mut self, record);

        let payload = runtime_value_summary(&record);
        let digest = sha256(&payload);
        let tx = self
            .backend
            .anchor_hash(&digest)
            .map_err(|err| ledger_err(format!("ledger append failed: {err}")))?;
        Ok(tx.0)
    }

    fn anchor(&mut self, digest: &[u8]) -> ProviderResult<String> {
        // Description:
        //     Anchor.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     diges: &[u8]
        //         Caller-supplied diges.
        //
        // Outputs:
        //     result: ProviderResult<String>
        //         Return value from `anchor`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::anchor(&mut self, diges);

        let hash = Hash(hex_encode(digest));
        let tx = self
            .backend
            .anchor_hash(&hash)
            .map_err(|err| ledger_err(format!("ledger anchor failed: {err}")))?;
        Ok(tx.0)
    }
}

/// Cloud provider with optional HTTP upload when `SPANDA_CLOUD_UPLOAD_URL` is set.
pub struct CloudPackageStub;

fn ledger_err(message: impl Into<String>) -> ProviderError {
    // Description:
    //     Ledger err.
    //
    // Inputs:
    //     essage: impl Into<String>
    //         Caller-supplied essage.
    //
    // Outputs:
    //     result: ProviderError
    //         Return value from `ledger_err`.
    //
    // Example:

    //     let result = spanda_providers::package_stubs::ledger_err(essage);

    ProviderError::new(ProviderId::new("spanda-ledger", "project"), message)
}

fn cloud_err(message: impl Into<String>) -> ProviderError {
    // Description:
    //     Cloud err.
    //
    // Inputs:
    //     essage: impl Into<String>
    //         Caller-supplied essage.
    //
    // Outputs:
    //     result: ProviderError
    //         Return value from `cloud_err`.
    //
    // Example:

    //     let result = spanda_providers::package_stubs::cloud_err(essage);

    ProviderError::new(ProviderId::new("spanda-cloud", "project"), message)
}

fn runtime_value_summary(value: &RuntimeValue) -> String {
    // Description:
    //     Runtime value summary.
    //
    // Inputs:
    //     value: &RuntimeValue
    //         Caller-supplied value.
    //
    // Outputs:
    //     result: String
    //         Return value from `runtime_value_summary`.
    //
    // Example:

    //     let result = spanda_providers::package_stubs::runtime_value_summary(value);

    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Object { type_name, .. } => format!("object:{type_name}"),
        other => format!("{other:?}"),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    // Description:
    //     Hex encode.
    //
    // Inputs:
    //     bytes: &[u8]
    //         Caller-supplied bytes.
    //
    // Outputs:
    //     result: String
    //         Return value from `hex_encode`.
    //
    // Example:

    //     let result = spanda_providers::package_stubs::hex_encode(bytes);

    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn cloud_upload_url() -> Option<String> {
    // Description:
    //     Cloud upload url.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `cloud_upload_url`.
    //
    // Example:

    //     let result = spanda_providers::package_stubs::cloud_upload_url();

    std::env::var("SPANDA_CLOUD_UPLOAD_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn post_json(url: &str, body: &str) -> Result<(), String> {
    // Description:
    //     Post json.
    //
    // Inputs:
    //     url: &str
    //         Caller-supplied url.
    //     body: &str
    //         Caller-supplied body.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `post_json`.
    //
    // Example:

    //     let result = spanda_providers::package_stubs::post_json(rl, body);

    let output = std::process::Command::new("curl")
        .args([
            "-fsS",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "--data-binary",
            body,
            url,
        ])
        .output()
        .map_err(|err| format!("curl failed to start: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "cloud upload failed (status {}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

impl CloudProvider for CloudPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            "spanda-cloud",
            "project",
            "Cloud upload via SPANDA_CLOUD_UPLOAD_URL (curl POST) or in-process stub",
        )
    }

    fn upload(&mut self, path: &str, payload: RuntimeValue) -> ProviderResult<()> {
        // Description:
        //     Upload.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     path: &str
        //         Caller-supplied path.
        //     payload: RuntimeValue
        //         Caller-supplied payload.
        //
        // Outputs:
        //     result: ProviderResult<()>
        //         Return value from `upload`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::upload(&mut self, path, payload);

        let body = serde_json::json!({
            "path": path,
            "payload": runtime_value_summary(&payload),
        })
        .to_string();
        if let Some(url) = cloud_upload_url() {
            post_json(&url, &body).map_err(cloud_err)?;
        }
        Ok(())
    }

    fn invoke(&mut self, endpoint: &str, request: RuntimeValue) -> ProviderResult<RuntimeValue> {
        // Description:
        //     Invoke.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     endpoin: &str
        //         Caller-supplied endpoin.
        //     request: RuntimeValue
        //         Caller-supplied request.
        //
        // Outputs:
        //     result: ProviderResult<RuntimeValue>
        //         Return value from `invoke`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::invoke(&mut self, endpoin, reques);

        Ok(RuntimeValue::Object {
            type_name: "CloudResponse".into(),
            fields: [
                (
                    "endpoint".into(),
                    RuntimeValue::String {
                        value: endpoint.into(),
                    },
                ),
                ("request".into(), request),
            ]
            .into_iter()
            .collect(),
        })
    }
}

/// Maintenance stub for `spanda-maintenance` package bootstrap.
pub struct MaintenancePackageStub;

impl MaintenanceProvider for MaintenancePackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            "spanda-maintenance",
            "project",
            "Package-scoped maintenance health stub",
        )
    }

    fn record_metric(&mut self, _component: &str, _metric: RuntimeValue) {
        // Description:
        //     Record metric.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     _componen: &str
        //         Caller-supplied componen.
        //     _metric: RuntimeValue
        //         Caller-supplied metric.
        //
        // Outputs:
        //     None.
        //
        // Example:
        // let result = spanda_providers::package_stubs::record_metric(&mut self, _componen, _metric);
    }

    fn health_report(&self, component: &str) -> RuntimeValue {
        // Description:
        //     Health report.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     componen: &str
        //         Caller-supplied componen.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `health_report`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::health_report(&self, componen);

        RuntimeValue::Object {
            type_name: "HealthReport".into(),
            fields: [(
                "component".into(),
                RuntimeValue::String {
                    value: component.into(),
                },
            )]
            .into_iter()
            .collect(),
        }
    }
}

/// Vision stub for `spanda-opencv` / `spanda-yolo` package bootstrap.
pub struct VisionPackageStub {
    package: &'static str,
}

impl VisionPackageStub {
    pub fn opencv() -> Self {
        // Description:
        //     Opencv.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `opencv`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::opencv();

        Self {
            package: "spanda-opencv",
        }
    }

    pub fn yolo() -> Self {
        // Description:
        //     Yolo.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `yolo`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::yolo();

        Self {
            package: "spanda-yolo",
        }
    }
}

impl VisionProvider for VisionPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            self.package,
            "project",
            "Package-scoped vision stub (AI runtime in core shim)",
        )
    }

    fn detect(&mut self, request: RuntimeValue) -> RuntimeValue {
        // Description:
        //     Detect.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     request: RuntimeValue
        //         Caller-supplied request.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `detect`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::detect(&mut self, reques);

        RuntimeValue::Object {
            type_name: "Detections".into(),
            fields: [("input".into(), request)].into_iter().collect(),
        }
    }

    fn classify(&mut self, request: RuntimeValue) -> RuntimeValue {
        // Description:
        //     Classify.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     request: RuntimeValue
        //         Caller-supplied request.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `classify`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::classify(&mut self, reques);

        RuntimeValue::Object {
            type_name: "Classification".into(),
            fields: [("input".into(), request)].into_iter().collect(),
        }
    }

    fn embed(&mut self, request: RuntimeValue) -> RuntimeValue {
        // Description:
        //     Embed.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     request: RuntimeValue
        //         Caller-supplied request.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `embed`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::embed(&mut self, reques);

        RuntimeValue::Object {
            type_name: "Embedding".into(),
            fields: [("input".into(), request)].into_iter().collect(),
        }
    }
}

/// Simulation stub for `spanda-gazebo` / `spanda-webots` package bootstrap.
pub struct SimulationPackageStub {
    package: &'static str,
}

impl SimulationPackageStub {
    pub fn gazebo() -> Self {
        // Description:
        //     Gazebo.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `gazebo`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::gazebo();

        Self {
            package: "spanda-gazebo",
        }
    }

    pub fn webots() -> Self {
        // Description:
        //     Webots.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `webots`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::webots();

        Self {
            package: "spanda-webots",
        }
    }
}

impl SimulationProvider for SimulationPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        // Description:
        //     Metadata.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: ProviderMetadata
        //         Return value from `metadata`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::metadata(&self);

        package_metadata(
            self.package,
            "project",
            "Package-scoped simulation backend stub",
        )
    }

    fn reset(&mut self) {
        // Description:
        //     Reset.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:
        // let result = spanda_providers::package_stubs::reset(&mut self);
    }

    fn step(&mut self, _dt_ms: f64) {
        // Description:
        //     Step.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     _dt_ms: f64
        //         Caller-supplied dt ms.
        //
        // Outputs:
        //     None.
        //
        // Example:
        // let result = spanda_providers::package_stubs::step(&mut self, _dt_ms);
    }

    fn observe(&self) -> RobotState {
        // Description:
        //     Observe.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: RobotState
        //         Return value from `observe`.
        //
        // Example:

        //     let result = spanda_providers::package_stubs::observe(&self);

        RobotState {
            pose: spanda_runtime::robot_state::PoseState {
                x: 0.0,
                y: 0.0,
                theta: 0.0,
                z: None,
            },
            velocity: spanda_runtime::robot_state::VelocityState {
                linear: 0.0,
                angular: 0.0,
            },
            emergency_stop: false,
        }
    }
}

/// Package-scoped wearable telemetry stub for H2 registry packages.
pub struct WearablePackageStub {
    package: &'static str,
}

impl WearablePackageStub {
    pub fn new(package: &'static str) -> Self {
        Self { package }
    }
}

impl WearableTelemetryProvider for WearablePackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            self.package,
            "project",
            "Package-scoped wearable telemetry stub",
        )
    }

    fn read_telemetry(&mut self, device_id: &str) -> ProviderResult<RuntimeValue> {
        let live = std::env::var("SPANDA_LIVE_WEARABLE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let mut fields = HashMap::new();
        fields.insert(
            "device_id".into(),
            RuntimeValue::String {
                value: device_id.to_string(),
            },
        );
        fields.insert(
            "heart_rate".into(),
            RuntimeValue::Number {
                value: if live { 72.0 } else { 0.0 },
                unit: UnitKind::None,
            },
        );
        fields.insert(
            "battery_percent".into(),
            RuntimeValue::Number {
                value: if live { 88.0 } else { 100.0 },
                unit: UnitKind::None,
            },
        );
        fields.insert(
            "connected".into(),
            RuntimeValue::Bool {
                value: live || !device_id.is_empty(),
            },
        );
        crate::hri_backends::enrich_healthkit_telemetry(self.package, device_id, &mut fields);
        Ok(RuntimeValue::Object {
            type_name: "WearableTelemetry".into(),
            fields,
        })
    }

    fn connectivity_status(&self, device_id: &str) -> bool {
        crate::hri_backends::healthkit_live_enabled()
            || std::env::var("SPANDA_LIVE_WEARABLE")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(!device_id.is_empty())
    }
}

/// Package-scoped spatial session stub for H2 AR/XR registry packages.
pub struct SpatialSessionPackageStub {
    package: &'static str,
    active_session: Option<SpatialSessionInfo>,
}

impl SpatialSessionPackageStub {
    pub fn new(package: &'static str) -> Self {
        Self {
            package,
            active_session: None,
        }
    }
}

impl SpatialSessionProvider for SpatialSessionPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            self.package,
            "project",
            "Package-scoped spatial session stub",
        )
    }

    fn start_session(&mut self, device_id: &str) -> ProviderResult<SpatialSessionInfo> {
        let live = std::env::var(format!(
            "SPANDA_{}_SESSION",
            self.package
                .replace("spanda-", "")
                .replace('-', "_")
                .to_uppercase()
        ))
        .or_else(|_| std::env::var("SPANDA_SPATIAL_SESSION"))
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
        if !live {
            return Err(ProviderError::new(
                ProviderId::new(self.package, "project"),
                format!("{} session backend not enabled", self.package),
            ));
        }
        let info = SpatialSessionInfo {
            session_id: format!("{}-{}", self.package, device_id),
            device_id: device_id.to_string(),
            active: true,
        };
        let mut info = info;
        crate::hri_backends::enrich_hololens_session(self.package, device_id, &mut info);
        self.active_session = Some(info.clone());
        Ok(info)
    }

    fn stop_session(&mut self, session_id: &str) -> ProviderResult<()> {
        if self
            .active_session
            .as_ref()
            .is_some_and(|s| s.session_id == session_id)
        {
            self.active_session = None;
            Ok(())
        } else {
            Err(ProviderError::new(
                ProviderId::new(self.package, "project"),
                format!("session not found: {session_id}"),
            ))
        }
    }

    fn publish_anchor(&mut self, _session_id: &str, _anchor: RuntimeValue) -> ProviderResult<()> {
        Ok(())
    }

    fn session_status(&self, session_id: &str) -> Option<SpatialSessionInfo> {
        self.active_session
            .as_ref()
            .filter(|s| s.session_id == session_id)
            .cloned()
    }
}

/// Package-scoped HRI input stub for voice, gesture, and eye-tracking packages.
pub struct HriInputPackageStub {
    package: &'static str,
}

impl HriInputPackageStub {
    pub fn new(package: &'static str) -> Self {
        Self { package }
    }
}

impl HriInputProvider for HriInputPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(self.package, "project", "Package-scoped HRI input stub")
    }

    fn poll_events(&mut self) -> ProviderResult<Vec<RuntimeValue>> {
        let live = std::env::var("SPANDA_LIVE_HRI")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let event_kind = match self.package {
            "spanda-voice" => "voice_command",
            "spanda-gesture" => "gesture_recognition",
            "spanda-eye-tracking" => "eye_tracking",
            _ => "hri_event",
        };
        let mut fields = HashMap::new();
        fields.insert(
            "kind".into(),
            RuntimeValue::String {
                value: event_kind.into(),
            },
        );
        fields.insert("active".into(), RuntimeValue::Bool { value: live });
        Ok(vec![RuntimeValue::Object {
            type_name: "HriEvent".into(),
            fields,
        }])
    }
}

/// Package-scoped overlay subscription stub for AR remote-expert layers.
pub struct OverlayPackageStub {
    package: &'static str,
}

impl OverlayPackageStub {
    pub fn new(package: &'static str) -> Self {
        Self { package }
    }
}

impl OverlayProvider for OverlayPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(self.package, "project", "Package-scoped AR overlay stub")
    }

    fn subscribe_overlay(&mut self, layer: &str, device_id: &str) -> ProviderResult<()> {
        if device_id.is_empty() {
            return Err(ProviderError::new(
                ProviderId::new(self.package, "project"),
                "device_id required for overlay subscription",
            ));
        }
        if layer.is_empty() {
            return Err(ProviderError::new(
                ProviderId::new(self.package, "project"),
                "overlay layer required",
            ));
        }
        Ok(())
    }
}

//! Package-scoped provider stubs registered when official packages are installed.
//!
use spanda_audit::{sha256, Hash, LedgerBackend, MockLedgerBackend};
use spanda_runtime::providers::traits::{
    CloudProvider, FleetProvider, LedgerProvider, MaintenanceProvider, NavigationProvider,
    PositioningProvider, SimulationProvider, SlamProvider, VisionProvider,
};
use spanda_runtime::providers::types::{
    ProviderError, ProviderId, ProviderMetadata, ProviderResult, ProviderSafetyLevel,
};
use spanda_runtime::robot_state::RobotState;
use spanda_runtime::value::{runtime_pose, RuntimeValue};

fn package_metadata(package: &str, name: &str, description: &str) -> ProviderMetadata {
    ProviderMetadata {
        id: ProviderId::new(package, name),
        description: description.into(),
        safety_level: ProviderSafetyLevel::Development,
        capabilities_required: Vec::new(),
        hardware_requirements: Vec::new(),
    }
}

/// GPS positioning stub for `spanda-gps` package bootstrap.
pub struct GpsPositioningStub;

impl PositioningProvider for GpsPositioningStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            "spanda-gps",
            "project",
            "Package-scoped GPS positioning stub (UART driver in core shim)",
        )
    }

    fn read_fix(&mut self) -> RuntimeValue {
        RuntimeValue::Object {
            type_name: "GeoPoint".into(),
            fields: [
                ("lat".into(), RuntimeValue::Number { value: 37.0, unit: spanda_ast::nodes::UnitKind::None }),
                ("lon".into(), RuntimeValue::Number { value: -122.0, unit: spanda_ast::nodes::UnitKind::None }),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn accuracy_meters(&self) -> Option<f64> {
        Some(5.0)
    }
}

/// Navigation stub for `spanda-nav` package bootstrap.
pub struct NavNavigationStub;

impl NavigationProvider for NavNavigationStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            "spanda-nav",
            "project",
            "Package-scoped navigation stub (Nav2 adapter in core shim)",
        )
    }

    fn navigate_to(&mut self, goal: RuntimeValue) -> ProviderResult<RuntimeValue> {
        Ok(RuntimeValue::Object {
            type_name: "NavigationGoal".into(),
            fields: [("goal".into(), goal)].into_iter().collect(),
        })
    }

    fn cancel_navigation(&mut self) {}

    fn navigation_status(&self) -> RuntimeValue {
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
        package_metadata(
            "spanda-slam",
            "project",
            "Package-scoped SLAM stub (subprocess adapter in core shim)",
        )
    }

    fn localize(&mut self) -> ProviderResult<RuntimeValue> {
        Ok(RuntimeValue::Object {
            type_name: "LocalizationEstimate".into(),
            fields: [
                (
                    "pose".into(),
                    runtime_pose(0.0, 0.0, 0.0, 0.0),
                ),
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
        package_metadata(
            "spanda-fleet",
            "project",
            "Package-scoped fleet stub (orchestrator in spanda-fleet crate)",
        )
    }

    fn register_member(&mut self, member_id: &str, _metadata: RuntimeValue) -> ProviderResult<()> {
        let _ = member_id;
        Ok(())
    }

    fn dispatch_task(
        &mut self,
        member_id: &str,
        task: RuntimeValue,
    ) -> ProviderResult<RuntimeValue> {
        Ok(RuntimeValue::Object {
            type_name: "FleetTask".into(),
            fields: [
                ("member".into(), RuntimeValue::String { value: member_id.into() }),
                ("task".into(), task),
            ]
            .into_iter()
            .collect(),
        })
    }

    fn member_status(&self, member_id: &str) -> Option<RuntimeValue> {
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
        Self {
            backend: MockLedgerBackend::new(),
        }
    }
}

impl LedgerProvider for LedgerPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            "spanda-ledger",
            "project",
            "Mock ledger provider (anchors audit digests via spanda-audit)",
        )
    }

    fn append(&mut self, record: RuntimeValue) -> ProviderResult<String> {
        let payload = runtime_value_summary(&record);
        let digest = sha256(&payload);
        let tx = self
            .backend
            .anchor_hash(&digest)
            .map_err(|err| ledger_err(format!("ledger append failed: {err}")))?;
        Ok(tx.0)
    }

    fn anchor(&mut self, digest: &[u8]) -> ProviderResult<String> {
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
    ProviderError::new(ProviderId::new("spanda-ledger", "project"), message)
}

fn cloud_err(message: impl Into<String>) -> ProviderError {
    ProviderError::new(ProviderId::new("spanda-cloud", "project"), message)
}

fn runtime_value_summary(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Object { type_name, .. } => format!("object:{type_name}"),
        other => format!("{other:?}"),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn cloud_upload_url() -> Option<String> {
    std::env::var("SPANDA_CLOUD_UPLOAD_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn post_json(url: &str, body: &str) -> Result<(), String> {
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
        package_metadata(
            "spanda-cloud",
            "project",
            "Cloud upload via SPANDA_CLOUD_UPLOAD_URL (curl POST) or in-process stub",
        )
    }

    fn upload(&mut self, path: &str, payload: RuntimeValue) -> ProviderResult<()> {
        let body = serde_json::json!({
            "path": path,
            "payload": runtime_value_summary(&payload),
        })
        .to_string();
        if let Some(url) = cloud_upload_url() {
            post_json(&url, &body).map_err(|err| cloud_err(err))?;
        }
        Ok(())
    }

    fn invoke(&mut self, endpoint: &str, request: RuntimeValue) -> ProviderResult<RuntimeValue> {
        Ok(RuntimeValue::Object {
            type_name: "CloudResponse".into(),
            fields: [
                ("endpoint".into(), RuntimeValue::String { value: endpoint.into() }),
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
        package_metadata(
            "spanda-maintenance",
            "project",
            "Package-scoped maintenance health stub",
        )
    }

    fn record_metric(&mut self, _component: &str, _metric: RuntimeValue) {}

    fn health_report(&self, component: &str) -> RuntimeValue {
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
        Self {
            package: "spanda-opencv",
        }
    }

    pub fn yolo() -> Self {
        Self {
            package: "spanda-yolo",
        }
    }
}

impl VisionProvider for VisionPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            self.package,
            "project",
            "Package-scoped vision stub (AI runtime in core shim)",
        )
    }

    fn detect(&mut self, request: RuntimeValue) -> RuntimeValue {
        RuntimeValue::Object {
            type_name: "Detections".into(),
            fields: [("input".into(), request)].into_iter().collect(),
        }
    }

    fn classify(&mut self, request: RuntimeValue) -> RuntimeValue {
        RuntimeValue::Object {
            type_name: "Classification".into(),
            fields: [("input".into(), request)].into_iter().collect(),
        }
    }

    fn embed(&mut self, request: RuntimeValue) -> RuntimeValue {
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
        Self {
            package: "spanda-gazebo",
        }
    }

    pub fn webots() -> Self {
        Self {
            package: "spanda-webots",
        }
    }
}

impl SimulationProvider for SimulationPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            self.package,
            "project",
            "Package-scoped simulation backend stub",
        )
    }

    fn reset(&mut self) {}

    fn step(&mut self, _dt_ms: f64) {}

    fn observe(&self) -> RobotState {
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

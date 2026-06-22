//! Package-scoped provider stubs registered when official packages are installed.
//!
use spanda_runtime::providers::traits::{
    CloudProvider, FleetProvider, LedgerProvider, MaintenanceProvider, NavigationProvider,
    PositioningProvider, SimulationProvider, SlamProvider, VisionProvider,
};
use spanda_runtime::providers::types::{
    ProviderId, ProviderMetadata, ProviderResult, ProviderSafetyLevel,
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

/// Ledger stub for `spanda-ledger` package bootstrap.
pub struct LedgerPackageStub;

impl LedgerProvider for LedgerPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            "spanda-ledger",
            "project",
            "Package-scoped ledger stub (audit runtime in core shim)",
        )
    }

    fn append(&mut self, _record: RuntimeValue) -> ProviderResult<String> {
        Ok("ledger-entry-1".into())
    }

    fn anchor(&mut self, digest: &[u8]) -> ProviderResult<String> {
        Ok(format!("anchor:{}", digest.len()))
    }
}

/// Cloud stub for `spanda-cloud` package bootstrap.
pub struct CloudPackageStub;

impl CloudProvider for CloudPackageStub {
    fn metadata(&self) -> ProviderMetadata {
        package_metadata(
            "spanda-cloud",
            "project",
            "Package-scoped cloud stub (remote invoke surface)",
        )
    }

    fn upload(&mut self, _path: &str, _payload: RuntimeValue) -> ProviderResult<()> {
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

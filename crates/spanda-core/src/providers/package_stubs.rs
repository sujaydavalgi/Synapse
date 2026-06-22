//! Package-scoped provider stubs registered when official packages are installed.
//!
use crate::providers::traits::{
    CloudProvider, FleetProvider, LedgerProvider, MaintenanceProvider, NavigationProvider,
    PositioningProvider, SimulationProvider, SlamProvider, VisionProvider,
};
use crate::providers::types::{
    ProviderId, ProviderMetadata, ProviderResult, ProviderSafetyLevel,
};
use crate::runtime::{runtime_pose, RuntimeValue};

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

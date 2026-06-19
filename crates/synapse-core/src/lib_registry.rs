use crate::error::PoseState;
use crate::hal::HalBackend;
use crate::runtime::RuntimeValue;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorInterface {
    I2c,
    Spi,
    Uart,
    Usb,
    Ethernet,
    Gpio,
}

impl SensorInterface {
    pub fn as_str(self) -> &'static str {
        match self {
            SensorInterface::I2c => "i2c",
            SensorInterface::Spi => "spi",
            SensorInterface::Uart => "uart",
            SensorInterface::Usb => "usb",
            SensorInterface::Ethernet => "ethernet",
            SensorInterface::Gpio => "gpio",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimState {
    pub pose: PoseState,
}

#[derive(Clone)]
pub struct DriverContext<'a> {
    pub hal: Option<&'a dyn HalBackend>,
    pub hal_binding: Option<&'a str>,
    pub topic: Option<&'a str>,
    pub sim_state: Option<SimState>,
}

pub type SensorReadFn = fn(&DriverContext) -> RuntimeValue;

#[derive(Debug, Clone)]
pub struct SensorDriverDef {
    pub sensor_type: String,
    pub vendor: String,
    pub model: String,
    pub interfaces: Vec<SensorInterface>,
    pub default_bus: Option<SensorInterface>,
    pub methods: Vec<String>,
    pub read: SensorReadFn,
}

#[derive(Debug, Clone)]
pub struct LibModule {
    pub id: String,
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub sensors: HashMap<String, SensorDriverDef>,
}

#[derive(Debug, Clone)]
pub struct LibrarySensorTypeInfo {
    pub robo_type: crate::ast::SynapseType,
    pub library: String,
}

fn scan_reading(ctx: &DriverContext, range: f64) -> RuntimeValue {
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let nearest = (range - x.abs() * 0.3).max(0.05);
    RuntimeValue::Scan { nearest_distance: nearest }
}

fn imu_reading(yaw: f64) -> RuntimeValue {
    use crate::ast::UnitKind;
    RuntimeValue::Object {
        type_name: "IMUReading".into(),
        fields: HashMap::from([
            (
                "roll".into(),
                RuntimeValue::Number {
                    value: 0.0,
                    unit: UnitKind::Rad,
                },
            ),
            (
                "pitch".into(),
                RuntimeValue::Number {
                    value: 0.0,
                    unit: UnitKind::Rad,
                },
            ),
            (
                "yaw".into(),
                RuntimeValue::Number {
                    value: yaw,
                    unit: UnitKind::Rad,
                },
            ),
        ]),
    }
}

fn read_velodyne_vlp16(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 100.0)
}

fn read_velodyne_vlp32(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 200.0)
}

fn read_hokuyo_ust10(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 10.0)
}

fn read_hokuyo_utm30(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 30.0)
}

fn read_bosch_bno055(ctx: &DriverContext) -> RuntimeValue {
    let yaw = ctx
        .sim_state
        .as_ref()
        .map(|s| s.pose.theta)
        .unwrap_or(0.0);
    if let (Some(hal), Some(binding)) = (ctx.hal, ctx.hal_binding) {
        let data = hal.read_i2c(binding, 0x1a, 2);
        let raw = data.first().copied().unwrap_or(0) as u16
            | ((data.get(1).copied().unwrap_or(0) as u16) << 8);
        return imu_reading(f64::from(raw) / 100.0);
    }
    imu_reading(yaw)
}

fn read_bosch_bmp388(ctx: &DriverContext) -> RuntimeValue {
    use crate::ast::UnitKind;
    let alt = ctx
        .sim_state
        .as_ref()
        .and_then(|s| s.pose.z)
        .unwrap_or(0.0);
    RuntimeValue::Number {
        value: alt,
        unit: UnitKind::M,
    }
}

fn read_intel_d435(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 5.0)
}

fn read_intel_d455(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 8.0)
}

fn read_ydlidar_x4(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 6.0)
}

fn read_ydlidar_g4(ctx: &DriverContext) -> RuntimeValue {
    scan_reading(ctx, 16.0)
}

fn read_adafruit_vl53l0x(ctx: &DriverContext) -> RuntimeValue {
    use crate::ast::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let dist = (2.0 - x * 0.1).max(0.02);
    RuntimeValue::Number {
        value: dist,
        unit: UnitKind::M,
    }
}

fn read_sparkfun_lsm9ds1(ctx: &DriverContext) -> RuntimeValue {
    let yaw = ctx
        .sim_state
        .as_ref()
        .map(|s| s.pose.theta)
        .unwrap_or(0.0);
    imu_reading(yaw)
}

fn read_waveshare_uwmf(ctx: &DriverContext) -> RuntimeValue {
    use crate::ast::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let dist = (4.0 - x * 0.2).max(0.02);
    RuntimeValue::Number {
        value: dist,
        unit: UnitKind::M,
    }
}

fn sensor(
    sensor_type: &str,
    vendor: &str,
    model: &str,
    interfaces: &[SensorInterface],
    default_bus: Option<SensorInterface>,
    methods: &[&str],
    read: SensorReadFn,
) -> SensorDriverDef {
    SensorDriverDef {
        sensor_type: sensor_type.to_string(),
        vendor: vendor.to_string(),
        model: model.to_string(),
        interfaces: interfaces.to_vec(),
        default_bus,
        methods: methods.iter().map(|m| (*m).to_string()).collect(),
        read,
    }
}

fn lib(
    id: &str,
    vendor: &str,
    name: &str,
    description: &str,
    sensors: HashMap<String, SensorDriverDef>,
) -> LibModule {
    LibModule {
        id: id.to_string(),
        vendor: vendor.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: description.to_string(),
        sensors,
    }
}

fn build_registry() -> HashMap<String, LibModule> {
    HashMap::from([
        (
            "velodyne.vlp16".to_string(),
            lib(
                "velodyne.vlp16",
                "Velodyne",
                "vlp16",
                "Velodyne VLP-16 3D LiDAR puck",
                HashMap::from([(
                    "VelodyneVLP16".to_string(),
                    sensor(
                        "VelodyneVLP16",
                        "Velodyne",
                        "VLP-16",
                        &[SensorInterface::Ethernet, SensorInterface::Usb],
                        None,
                        &["read", "calibrate"],
                        read_velodyne_vlp16,
                    ),
                )]),
            ),
        ),
        (
            "velodyne.vlp32".to_string(),
            lib(
                "velodyne.vlp32",
                "Velodyne",
                "vlp32",
                "Velodyne VLP-32C ultra puck",
                HashMap::from([(
                    "VelodyneVLP32".to_string(),
                    sensor(
                        "VelodyneVLP32",
                        "Velodyne",
                        "VLP-32C",
                        &[SensorInterface::Ethernet],
                        None,
                        &["read"],
                        read_velodyne_vlp32,
                    ),
                )]),
            ),
        ),
        (
            "hokuyo.ust10".to_string(),
            lib(
                "hokuyo.ust10",
                "Hokuyo",
                "ust10",
                "Hokuyo UST-10LX 2D LiDAR",
                HashMap::from([(
                    "HokuyoUST10".to_string(),
                    sensor(
                        "HokuyoUST10",
                        "Hokuyo",
                        "UST-10LX",
                        &[SensorInterface::Ethernet, SensorInterface::Uart],
                        None,
                        &["read"],
                        read_hokuyo_ust10,
                    ),
                )]),
            ),
        ),
        (
            "hokuyo.utm30".to_string(),
            lib(
                "hokuyo.utm30",
                "Hokuyo",
                "utm30",
                "Hokuyo UTM-30LX-EW outdoor LiDAR",
                HashMap::from([(
                    "HokuyoUTM30".to_string(),
                    sensor(
                        "HokuyoUTM30",
                        "Hokuyo",
                        "UTM-30LX-EW",
                        &[SensorInterface::Ethernet],
                        None,
                        &["read"],
                        read_hokuyo_utm30,
                    ),
                )]),
            ),
        ),
        (
            "bosch.bno055".to_string(),
            lib(
                "bosch.bno055",
                "Bosch",
                "bno055",
                "Bosch BNO055 9-DOF absolute orientation IMU",
                HashMap::from([(
                    "BoschBNO055".to_string(),
                    sensor(
                        "BoschBNO055",
                        "Bosch",
                        "BNO055",
                        &[SensorInterface::I2c, SensorInterface::Uart],
                        Some(SensorInterface::I2c),
                        &["read", "calibrate"],
                        read_bosch_bno055,
                    ),
                )]),
            ),
        ),
        (
            "bosch.bmp388".to_string(),
            lib(
                "bosch.bmp388",
                "Bosch",
                "bmp388",
                "Bosch BMP388 barometric pressure sensor",
                HashMap::from([(
                    "BoschBMP388".to_string(),
                    sensor(
                        "BoschBMP388",
                        "Bosch",
                        "BMP388",
                        &[SensorInterface::I2c, SensorInterface::Spi],
                        Some(SensorInterface::I2c),
                        &["read"],
                        read_bosch_bmp388,
                    ),
                )]),
            ),
        ),
        (
            "intel.realsense".to_string(),
            lib(
                "intel.realsense",
                "Intel",
                "realsense",
                "Intel RealSense depth cameras",
                HashMap::from([
                    (
                        "IntelRealSenseD435".to_string(),
                        sensor(
                            "IntelRealSenseD435",
                            "Intel",
                            "D435",
                            &[SensorInterface::Usb],
                            None,
                            &["read", "read_depth"],
                            read_intel_d435,
                        ),
                    ),
                    (
                        "IntelRealSenseD455".to_string(),
                        sensor(
                            "IntelRealSenseD455",
                            "Intel",
                            "D455",
                            &[SensorInterface::Usb],
                            None,
                            &["read", "read_depth"],
                            read_intel_d455,
                        ),
                    ),
                ]),
            ),
        ),
        (
            "ydlidar.x4".to_string(),
            lib(
                "ydlidar.x4",
                "YDLIDAR",
                "x4",
                "YDLIDAR X4 2D LiDAR",
                HashMap::from([(
                    "YdlidarX4".to_string(),
                    sensor(
                        "YdlidarX4",
                        "YDLIDAR",
                        "X4",
                        &[SensorInterface::Uart, SensorInterface::Usb],
                        Some(SensorInterface::Uart),
                        &["read"],
                        read_ydlidar_x4,
                    ),
                )]),
            ),
        ),
        (
            "ydlidar.g4".to_string(),
            lib(
                "ydlidar.g4",
                "YDLIDAR",
                "g4",
                "YDLIDAR G4 2D LiDAR",
                HashMap::from([(
                    "YdlidarG4".to_string(),
                    sensor(
                        "YdlidarG4",
                        "YDLIDAR",
                        "G4",
                        &[SensorInterface::Uart, SensorInterface::Usb],
                        None,
                        &["read"],
                        read_ydlidar_g4,
                    ),
                )]),
            ),
        ),
        (
            "adafruit.vl53l0x".to_string(),
            lib(
                "adafruit.vl53l0x",
                "Adafruit",
                "vl53l0x",
                "Adafruit VL53L0X time-of-flight distance sensor",
                HashMap::from([(
                    "AdafruitVL53L0X".to_string(),
                    sensor(
                        "AdafruitVL53L0X",
                        "Adafruit",
                        "VL53L0X",
                        &[SensorInterface::I2c],
                        Some(SensorInterface::I2c),
                        &["read"],
                        read_adafruit_vl53l0x,
                    ),
                )]),
            ),
        ),
        (
            "sparkfun.lsm9ds1".to_string(),
            lib(
                "sparkfun.lsm9ds1",
                "SparkFun",
                "lsm9ds1",
                "SparkFun LSM9DS1 9-DOF IMU breakout",
                HashMap::from([(
                    "SparkfunLSM9DS1".to_string(),
                    sensor(
                        "SparkfunLSM9DS1",
                        "SparkFun",
                        "LSM9DS1",
                        &[SensorInterface::I2c, SensorInterface::Spi],
                        Some(SensorInterface::I2c),
                        &["read"],
                        read_sparkfun_lsm9ds1,
                    ),
                )]),
            ),
        ),
        (
            "waveshare.uwmf".to_string(),
            lib(
                "waveshare.uwmf",
                "Waveshare",
                "uwmf",
                "Waveshare ultrasonic distance module",
                HashMap::from([(
                    "WaveshareUWMF".to_string(),
                    sensor(
                        "WaveshareUWMF",
                        "Waveshare",
                        "UWMF",
                        &[SensorInterface::Gpio, SensorInterface::Uart],
                        None,
                        &["read"],
                        read_waveshare_uwmf,
                    ),
                )]),
            ),
        ),
    ])
}

static LIB_REGISTRY: OnceLock<HashMap<String, LibModule>> = OnceLock::new();

fn registry() -> &'static HashMap<String, LibModule> {
    LIB_REGISTRY.get_or_init(build_registry)
}

pub fn resolve_import(path: &str) -> Option<&'static LibModule> {
    registry().get(path)
}

pub fn get_sensor_driver(library_id: &str, sensor_type: &str) -> Option<SensorDriverDef> {
    registry()
        .get(library_id)
        .and_then(|lib| lib.sensors.get(sensor_type))
        .cloned()
}

pub fn get_sensor_type_from_lib(library_id: &str, sensor_type: &str) -> bool {
    get_sensor_driver(library_id, sensor_type).is_some()
}

pub fn all_library_sensor_types() -> HashMap<String, LibrarySensorTypeInfo> {
    let mut result = HashMap::new();
    for (lib_id, module) in registry() {
        for type_name in module.sensors.keys() {
            result.insert(
                type_name.clone(),
                LibrarySensorTypeInfo {
                    robo_type: crate::ast::SynapseType::Named {
                        name: type_name.clone(),
                    },
                    library: lib_id.clone(),
                },
            );
        }
    }
    result
}

pub fn list_libraries() -> Vec<&'static LibModule> {
    registry().values().collect()
}

pub fn list_libraries_by_vendor(vendor: &str) -> Vec<&'static LibModule> {
    let vendor_lower = vendor.to_lowercase();
    registry()
        .values()
        .filter(|l| l.vendor.to_lowercase() == vendor_lower)
        .collect()
}

pub fn read_with_driver(driver: &SensorDriverDef, ctx: &DriverContext) -> RuntimeValue {
    (driver.read)(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_vendor_libraries() {
        assert_eq!(resolve_import("bosch.bno055").unwrap().vendor, "Bosch");
        assert!(resolve_import("velodyne.vlp16")
            .unwrap()
            .sensors
            .contains_key("VelodyneVLP16"));
        assert!(resolve_import("intel.realsense")
            .unwrap()
            .sensors
            .contains_key("IntelRealSenseD435"));
    }

    #[test]
    fn lists_libraries_by_vendor() {
        assert_eq!(list_libraries_by_vendor("Hokuyo").len(), 2);
        assert!(list_libraries().len() >= 10);
    }

    #[test]
    fn scan_reading_uses_sim_pose() {
        let ctx = DriverContext {
            hal: None,
            hal_binding: None,
            topic: None,
            sim_state: Some(SimState {
                pose: PoseState {
                    x: 2.0,
                    y: 0.0,
                    theta: 0.0,
                    z: None,
                },
            }),
        };
        if let RuntimeValue::Scan { nearest_distance } = read_hokuyo_ust10(&ctx) {
            assert!((nearest_distance - 9.4).abs() < 0.01);
        } else {
            panic!("expected scan");
        }
    }

    #[test]
    fn bosch_imu_reads_hal_i2c() {
        use crate::hal::{create_sim_hal, HalMemberConfig};
        let mut hal = create_sim_hal();
        hal.configure(&[HalMemberConfig::I2c {
            name: "imu".to_string(),
            address: 104.0,
        }]);
        hal.seed_imu_registers("imu", 1.5);
        let ctx = DriverContext {
            hal: Some(&hal),
            hal_binding: Some("imu"),
            topic: None,
            sim_state: None,
        };
        if let RuntimeValue::Object { fields, .. } = read_bosch_bno055(&ctx) {
            if let RuntimeValue::Number { value, .. } = fields["yaw"] {
                assert!(value > 0.0);
            } else {
                panic!("expected yaw number");
            }
        } else {
            panic!("expected IMU object");
        }
    }
}

//! lib registry support for Spanda.
//!
use spanda_runtime::robot_state::PoseState;
use spanda_hal::hal::HalBackend;
use spanda_runtime::value::RuntimeValue;
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
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.as_str();

        // Dispatch based on the enum variant or current state.
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
    pub robo_type: spanda_ast::nodes::SpandaType,
    pub library: String,
}

fn scan_reading(ctx: &DriverContext, range: f64) -> RuntimeValue {
    // Scan reading.
    //
    // Parameters:
    // - `ctx` — input value
    // - `range` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::scan_reading(ctx, range);

    // Compute x for the following logic.
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let nearest = (range - x.abs() * 0.3).max(0.05);
    RuntimeValue::Scan {
        nearest_distance: nearest,
    }
}

fn imu_reading(yaw: f64) -> RuntimeValue {
    // Imu reading.
    //
    // Parameters:
    // - `yaw` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::imu_reading(yaw);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
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

fn gps_fix_reading(lat: f64, lon: f64, alt: f64) -> RuntimeValue {
    // Build a GpsFix object from WGS84 coordinates.
    //
    // Parameters:
    // - `lat`, `lon` — degrees
    // - `alt` — altitude in meters
    //
    // Returns:
    // GpsFix runtime object.
    //
    // Options:
    // None.
    //
    // Example:
    // let fix = gps_fix_reading(30.0, -97.0, 150.0);

    use spanda_ast::nodes::UnitKind;
    RuntimeValue::Object {
        type_name: "GpsFix".into(),
        fields: HashMap::from([
            (
                "lat".into(),
                RuntimeValue::Number {
                    value: lat,
                    unit: UnitKind::None,
                },
            ),
            (
                "lon".into(),
                RuntimeValue::Number {
                    value: lon,
                    unit: UnitKind::None,
                },
            ),
            (
                "altitude".into(),
                RuntimeValue::Number {
                    value: alt,
                    unit: UnitKind::M,
                },
            ),
            (
                "fix_quality".into(),
                RuntimeValue::Number {
                    value: 1.0,
                    unit: UnitKind::None,
                },
            ),
        ]),
    }
}

fn read_ublox_neo_m8n(ctx: &DriverContext) -> RuntimeValue {
    // Read u-blox NEO-M8N GNSS fix via UART NMEA stub.
    //
    // Parameters:
    // - `ctx` — driver context with optional HAL and simulation pose
    //
    // Returns:
    // GpsFix runtime object.
    //
    // Options:
    // None.
    //
    // Example:
    // let fix = read_ublox_neo_m8n(&ctx);

    let lat = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let lon = ctx.sim_state.as_ref().map(|s| s.pose.y).unwrap_or(0.0);
    let alt = ctx.sim_state.as_ref().and_then(|s| s.pose.z).unwrap_or(0.0);
    if let (Some(hal), Some(binding)) = (ctx.hal, ctx.hal_binding) {
        let _nmea = hal.read_uart(binding);
        return gps_fix_reading(lat, lon, alt);
    }
    gps_fix_reading(lat, lon, alt)
}

fn read_velodyne_vlp16(ctx: &DriverContext) -> RuntimeValue {
    // Read velodyne vlp16.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_velodyne_vlp16(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 100.0)
}

fn read_velodyne_vlp32(ctx: &DriverContext) -> RuntimeValue {
    // Read velodyne vlp32.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_velodyne_vlp32(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 200.0)
}

fn read_hokuyo_ust10(ctx: &DriverContext) -> RuntimeValue {
    // Read hokuyo ust10.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_hokuyo_ust10(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 10.0)
}

fn read_hokuyo_utm30(ctx: &DriverContext) -> RuntimeValue {
    // Read hokuyo utm30.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_hokuyo_utm30(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 30.0)
}

fn read_bosch_bno055(ctx: &DriverContext) -> RuntimeValue {
    // Read bosch bno055.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_bosch_bno055(ctx);

    // Compute yaw for the following logic.
    let yaw = ctx.sim_state.as_ref().map(|s| s.pose.theta).unwrap_or(0.0);

    // Take this path when let (Some(hal), Some(binding)) = (ctx.hal, ctx.hal binding).
    if let (Some(hal), Some(binding)) = (ctx.hal, ctx.hal_binding) {
        let data = hal.read_i2c(binding, 0x1a, 2);
        let raw = data.first().copied().unwrap_or(0) as u16
            | ((data.get(1).copied().unwrap_or(0) as u16) << 8);
        return imu_reading(f64::from(raw) / 100.0);
    }
    imu_reading(yaw)
}

fn read_bosch_bmp388(ctx: &DriverContext) -> RuntimeValue {
    // Read bosch bmp388.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_bosch_bmp388(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let alt = ctx.sim_state.as_ref().and_then(|s| s.pose.z).unwrap_or(0.0);
    RuntimeValue::Number {
        value: alt,
        unit: UnitKind::M,
    }
}

fn read_bosch_bme280_humidity(ctx: &DriverContext) -> RuntimeValue {
    // Read bosch bme280 humidity.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_bosch_bme280_humidity(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let humidity = (55.0 - x * 2.0).clamp(30.0, 90.0);
    RuntimeValue::Number {
        value: humidity,
        unit: UnitKind::Rh,
    }
}

fn read_adafruit_bh1750(ctx: &DriverContext) -> RuntimeValue {
    // Read adafruit bh1750.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_adafruit_bh1750(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let lux = (400.0 - x * 20.0).clamp(0.0, 100_000.0);
    RuntimeValue::Number {
        value: lux,
        unit: UnitKind::Lux,
    }
}

fn read_adafruit_veml6075(ctx: &DriverContext) -> RuntimeValue {
    // Read adafruit veml6075.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_adafruit_veml6075(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let uvi = (6.0 - x * 0.3).clamp(0.0, 11.0);
    RuntimeValue::Number {
        value: uvi,
        unit: UnitKind::Uvi,
    }
}

fn read_atlas_ph(ctx: &DriverContext) -> RuntimeValue {
    // Read atlas ph.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_atlas_ph(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let ph = (7.0 + x * 0.05).clamp(0.0, 14.0);
    RuntimeValue::Number {
        value: ph,
        unit: UnitKind::Ph,
    }
}

fn read_sparkfun_ec(ctx: &DriverContext) -> RuntimeValue {
    // Read sparkfun ec.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_sparkfun_ec(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let ec = (500.0 + x * 50.0).clamp(0.0, 20_000.0);
    RuntimeValue::Number {
        value: ec,
        unit: UnitKind::MicroSPerCm,
    }
}

fn read_plantower_pms5003(ctx: &DriverContext) -> RuntimeValue {
    // Read plantower pms5003.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_plantower_pms5003(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let pm = (12.0 + x * 3.0).clamp(0.0, 500.0);
    RuntimeValue::Number {
        value: pm,
        unit: UnitKind::UgPerM3,
    }
}

fn read_dfrobot_turbidity(ctx: &DriverContext) -> RuntimeValue {
    // Read dfrobot turbidity.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_dfrobot_turbidity(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let ntu = (2.0 + x * 0.5).clamp(0.0, 1000.0);
    RuntimeValue::Number {
        value: ntu,
        unit: UnitKind::Ntu,
    }
}

fn read_atlas_salinity(ctx: &DriverContext) -> RuntimeValue {
    // Read atlas salinity.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_atlas_salinity(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let ppt = (35.0 - x * 0.1).clamp(0.0, 40.0);
    RuntimeValue::Number {
        value: ppt,
        unit: UnitKind::Ppt,
    }
}

fn read_gq_gmc(ctx: &DriverContext) -> RuntimeValue {
    // Read gq gmc.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_gq_gmc(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let dose = (0.1 + x * 0.02).clamp(0.0, 10.0);
    RuntimeValue::Number {
        value: dose,
        unit: UnitKind::MicroSvPerH,
    }
}

fn read_vegetronix_soil(ctx: &DriverContext) -> RuntimeValue {
    // Read vegetronix soil.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_vegetronix_soil(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let vwc = (40.0 - x * 2.0).clamp(0.0, 100.0);
    RuntimeValue::Number {
        value: vwc,
        unit: UnitKind::PercentVwc,
    }
}

fn read_intel_d435(ctx: &DriverContext) -> RuntimeValue {
    // Read intel d435.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_intel_d435(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 5.0)
}

fn read_intel_d455(ctx: &DriverContext) -> RuntimeValue {
    // Read intel d455.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_intel_d455(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 8.0)
}

fn read_ydlidar_x4(ctx: &DriverContext) -> RuntimeValue {
    // Read ydlidar x4.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_ydlidar_x4(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 6.0)
}

fn read_ydlidar_g4(ctx: &DriverContext) -> RuntimeValue {
    // Read ydlidar g4.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_ydlidar_g4(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 16.0)
}

fn read_ouster_os1(ctx: &DriverContext) -> RuntimeValue {
    // Read ouster os1.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_ouster_os1(ctx);

    // Produce 0) as the result.
    scan_reading(ctx, 120.0)
}

fn read_adafruit_vl53l0x(ctx: &DriverContext) -> RuntimeValue {
    // Read adafruit vl53l0x.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_adafruit_vl53l0x(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
    let x = ctx.sim_state.as_ref().map(|s| s.pose.x).unwrap_or(0.0);
    let dist = (2.0 - x * 0.1).max(0.02);
    RuntimeValue::Number {
        value: dist,
        unit: UnitKind::M,
    }
}

fn read_sparkfun_lsm9ds1(ctx: &DriverContext) -> RuntimeValue {
    // Read sparkfun lsm9ds1.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_sparkfun_lsm9ds1(ctx);

    // Compute yaw for the following logic.
    let yaw = ctx.sim_state.as_ref().map(|s| s.pose.theta).unwrap_or(0.0);
    imu_reading(yaw)
}

fn read_waveshare_uwmf(ctx: &DriverContext) -> RuntimeValue {
    // Read waveshare uwmf.
    //
    // Parameters:
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_waveshare_uwmf(ctx);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::UnitKind;
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
    // Sensor.
    //
    // Parameters:
    // - `sensor_type` — input value
    // - `vendor` — input value
    // - `model` — input value
    // - `interfaces` — input value
    // - `default_bus` — input value
    // - `methods` — input value
    // - `read` — input value
    //
    // Returns:
    // SensorDriverDef.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::sensor(sensor_type, vendor, model, interfaces, default_bus, methods, read);

    // Produce SensorDriverDef as the result.
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
    // Lib.
    //
    // Parameters:
    // - `id` — input value
    // - `vendor` — input value
    // - `name` — input value
    // - `description` — input value
    // - `sensors` — input value
    //
    // Returns:
    // LibModule.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::lib(id, vendor, name, description, sensors);

    // Produce LibModule as the result.
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
    // Build registry.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, LibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::build_registry();

    // Produce from as the result.
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
            "bosch.bme280".to_string(),
            lib(
                "bosch.bme280",
                "Bosch",
                "bme280",
                "Bosch BME280 environmental sensor (humidity, pressure, temperature)",
                HashMap::from([(
                    "BoschBME280".to_string(),
                    sensor(
                        "BoschBME280",
                        "Bosch",
                        "BME280",
                        &[SensorInterface::I2c, SensorInterface::Spi],
                        Some(SensorInterface::I2c),
                        &["read", "calibrate"],
                        read_bosch_bme280_humidity,
                    ),
                )]),
            ),
        ),
        (
            "adafruit.bh1750".to_string(),
            lib(
                "adafruit.bh1750",
                "Adafruit",
                "bh1750",
                "Adafruit BH1750 digital light sensor",
                HashMap::from([(
                    "AdafruitBH1750".to_string(),
                    sensor(
                        "AdafruitBH1750",
                        "Adafruit",
                        "BH1750",
                        &[SensorInterface::I2c],
                        Some(SensorInterface::I2c),
                        &["read"],
                        read_adafruit_bh1750,
                    ),
                )]),
            ),
        ),
        (
            "adafruit.veml6075".to_string(),
            lib(
                "adafruit.veml6075",
                "Adafruit",
                "veml6075",
                "Adafruit VEML6075 UV index sensor",
                HashMap::from([(
                    "AdafruitVEML6075".to_string(),
                    sensor(
                        "AdafruitVEML6075",
                        "Adafruit",
                        "VEML6075",
                        &[SensorInterface::I2c],
                        Some(SensorInterface::I2c),
                        &["read"],
                        read_adafruit_veml6075,
                    ),
                )]),
            ),
        ),
        (
            "atlas.ph".to_string(),
            lib(
                "atlas.ph",
                "Atlas",
                "ph",
                "Atlas Scientific pH sensor",
                HashMap::from([(
                    "AtlasPH".to_string(),
                    sensor(
                        "AtlasPH",
                        "Atlas",
                        "pH",
                        &[SensorInterface::Uart],
                        Some(SensorInterface::Uart),
                        &["read", "calibrate"],
                        read_atlas_ph,
                    ),
                )]),
            ),
        ),
        (
            "sparkfun.ec".to_string(),
            lib(
                "sparkfun.ec",
                "SparkFun",
                "ec",
                "SparkFun conductivity sensor",
                HashMap::from([(
                    "SparkfunEC".to_string(),
                    sensor(
                        "SparkfunEC",
                        "SparkFun",
                        "EC",
                        &[SensorInterface::Uart, SensorInterface::Gpio],
                        Some(SensorInterface::Uart),
                        &["read"],
                        read_sparkfun_ec,
                    ),
                )]),
            ),
        ),
        (
            "plantower.pms5003".to_string(),
            lib(
                "plantower.pms5003",
                "Plantower",
                "pms5003",
                "Plantower PMS5003 particulate matter sensor",
                HashMap::from([(
                    "PlantowerPMS5003".to_string(),
                    sensor(
                        "PlantowerPMS5003",
                        "Plantower",
                        "PMS5003",
                        &[SensorInterface::Uart],
                        Some(SensorInterface::Uart),
                        &["read"],
                        read_plantower_pms5003,
                    ),
                )]),
            ),
        ),
        (
            "dfrobot.turbidity".to_string(),
            lib(
                "dfrobot.turbidity",
                "DFRobot",
                "turbidity",
                "DFRobot turbidity sensor",
                HashMap::from([(
                    "DfrobotTurbidity".to_string(),
                    sensor(
                        "DfrobotTurbidity",
                        "DFRobot",
                        "Turbidity",
                        &[SensorInterface::Uart, SensorInterface::Gpio],
                        None,
                        &["read"],
                        read_dfrobot_turbidity,
                    ),
                )]),
            ),
        ),
        (
            "atlas.salinity".to_string(),
            lib(
                "atlas.salinity",
                "Atlas",
                "salinity",
                "Atlas Scientific salinity sensor",
                HashMap::from([(
                    "AtlasSalinity".to_string(),
                    sensor(
                        "AtlasSalinity",
                        "Atlas",
                        "Salinity",
                        &[SensorInterface::Uart],
                        Some(SensorInterface::Uart),
                        &["read", "calibrate"],
                        read_atlas_salinity,
                    ),
                )]),
            ),
        ),
        (
            "gq.gmc".to_string(),
            lib(
                "gq.gmc",
                "GQ",
                "gmc",
                "GQ GMC geiger counter",
                HashMap::from([(
                    "GqGMC".to_string(),
                    sensor(
                        "GqGMC",
                        "GQ",
                        "GMC",
                        &[SensorInterface::Uart, SensorInterface::Usb],
                        Some(SensorInterface::Uart),
                        &["read"],
                        read_gq_gmc,
                    ),
                )]),
            ),
        ),
        (
            "vegetronix.soil".to_string(),
            lib(
                "vegetronix.soil",
                "Vegetronix",
                "soil",
                "Vegetronix soil moisture sensor",
                HashMap::from([(
                    "VegetronixSoil".to_string(),
                    sensor(
                        "VegetronixSoil",
                        "Vegetronix",
                        "Soil",
                        &[SensorInterface::Gpio, SensorInterface::Uart],
                        None,
                        &["read"],
                        read_vegetronix_soil,
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
        (
            "ouster.os1".to_string(),
            lib(
                "ouster.os1",
                "Ouster",
                "os1",
                "Ouster OS1 digital LiDAR sensor",
                HashMap::from([(
                    "OusterOS1".to_string(),
                    sensor(
                        "OusterOS1",
                        "Ouster",
                        "OS1",
                        &[SensorInterface::Ethernet],
                        Some(SensorInterface::Ethernet),
                        &["read", "calibrate"],
                        read_ouster_os1,
                    ),
                )]),
            ),
        ),
        (
            "ublox.neo_m8n".to_string(),
            lib(
                "ublox.neo_m8n",
                "u-blox",
                "neo_m8n",
                "u-blox NEO-M8N multi-GNSS receiver (UART NMEA)",
                HashMap::from([(
                    "UbloxNEOM8N".to_string(),
                    sensor(
                        "UbloxNEOM8N",
                        "u-blox",
                        "NEO-M8N",
                        &[SensorInterface::Uart],
                        Some(SensorInterface::Uart),
                        &["read"],
                        read_ublox_neo_m8n,
                    ),
                )]),
            ),
        ),
    ])
}

static LIB_REGISTRY: OnceLock<HashMap<String, LibModule>> = OnceLock::new();

fn registry() -> &'static HashMap<String, LibModule> {
    // Registry.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // &'static HashMap<String, LibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::registry();

    // Produce get or init as the result.
    LIB_REGISTRY.get_or_init(build_registry)
}

pub fn resolve_import(path: &str) -> Option<&'static LibModule> {
    // Resolve import.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::resolve_import(path);

    // Produce get as the result.
    registry().get(path)
}

pub fn get_sensor_driver(library_id: &str, sensor_type: &str) -> Option<SensorDriverDef> {
    //
    // Parameters:
    // - `library_id` — input value
    // - `sensor_type` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::get_sensor_driver(library_id, sensor_type);

    // Produce registry as the result.
    registry()
        .get(library_id)
        .and_then(|lib| lib.sensors.get(sensor_type))
        .cloned()
}

pub fn get_sensor_type_from_lib(library_id: &str, sensor_type: &str) -> bool {
    //
    // Parameters:
    // - `library_id` — input value
    // - `sensor_type` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::get_sensor_type_from_lib(library_id, sensor_type);

    // Produce is some as the result.
    get_sensor_driver(library_id, sensor_type).is_some()
}

pub fn all_library_sensor_types() -> HashMap<String, LibrarySensorTypeInfo> {
    // All library sensor types.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, LibrarySensorTypeInfo>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::all_library_sensor_types();

    // Create mutable result for accumulating results.
    let mut result = HashMap::new();

    // Iterate over registry with destructured elements.
    for (lib_id, module) in registry() {
        // Process each key.
        for type_name in module.sensors.keys() {
            result.insert(
                type_name.clone(),
                LibrarySensorTypeInfo {
                    robo_type: spanda_ast::nodes::SpandaType::Named {
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
    // List libraries.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Vec<&'static LibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::list_libraries();

    // Collect filtered entries into a new list.
    registry().values().collect()
}

pub fn list_libraries_by_vendor(vendor: &str) -> Vec<&'static LibModule> {
    // List libraries by vendor.
    //
    // Parameters:
    // - `vendor` — input value
    //
    // Returns:
    // Vec<&'static LibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::list_libraries_by_vendor(vendor);

    // Compute vendor lower for the following logic.
    let vendor_lower = vendor.to_lowercase();
    registry()
        .values()
        .filter(|l| l.vendor.to_lowercase() == vendor_lower)
        .collect()
}

pub fn read_with_driver(driver: &SensorDriverDef, ctx: &DriverContext) -> RuntimeValue {
    // Read with driver.
    //
    // Parameters:
    // - `driver` — input value
    // - `ctx` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_lib_registry::read_with_driver(driver, ctx);

    // Produce read) as the result.
    (driver.read)(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_vendor_libraries() {
        // Resolves vendor libraries.
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
        // let result = spanda_lib_registry::resolves_vendor_libraries();

        assert_eq!(resolve_import("bosch.bno055").unwrap().vendor, "Bosch");
        assert_eq!(resolve_import("ublox.neo_m8n").unwrap().vendor, "u-blox");
        assert!(resolve_import("velodyne.vlp16")
            .unwrap()
            .sensors
            .contains_key("VelodyneVLP16"));
        assert!(resolve_import("intel.realsense")
            .unwrap()
            .sensors
            .contains_key("IntelRealSenseD435"));
        assert!(resolve_import("ouster.os1")
            .unwrap()
            .sensors
            .contains_key("OusterOS1"));
    }

    #[test]
    fn lists_libraries_by_vendor() {
        // Lists libraries by vendor.
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
        // let result = spanda_lib_registry::lists_libraries_by_vendor();

        assert_eq!(list_libraries_by_vendor("Hokuyo").len(), 2);
        assert!(list_libraries().len() >= 10);
    }

    #[test]
    fn scan_reading_uses_sim_pose() {
        // Scan reading uses sim pose.
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
        // let result = spanda_lib_registry::scan_reading_uses_sim_pose();

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
        // Bosch imu reads hal i2c.
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
        // let result = spanda_lib_registry::bosch_imu_reads_hal_i2c();

        use spanda_hal::hal::{create_sim_hal, HalMemberConfig};
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

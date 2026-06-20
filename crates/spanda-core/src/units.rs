//! Physical unit categories, compatibility, and SI conversion.

use crate::ast::UnitKind;

/// Physical dimension for unit algebra (reject e.g. `speed + voltage`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalCategory {
    Scalar,
    Distance,
    Duration,
    Velocity,
    Acceleration,
    Angle,
    AngularVelocity,
    Mass,
    Force,
    Power,
    Voltage,
    Current,
    Temperature,
    Pressure,
    Frequency,
    Humidity,
    Illuminance,
    Luminance,
    Concentration,
    SoundLevel,
    MagneticField,
    RotationalSpeed,
    Torque,
    Energy,
    UvIndex,
    Ph,
    Conductivity,
    ParticulateMatter,
    Turbidity,
    Salinity,
    Radiation,
    SoilMoisture,
}

const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;

pub fn unit_category(unit: UnitKind) -> PhysicalCategory {
    match unit {
        UnitKind::None => PhysicalCategory::Scalar,
        UnitKind::M | UnitKind::Mm | UnitKind::Cm | UnitKind::Km | UnitKind::Ft | UnitKind::In => {
            PhysicalCategory::Distance
        }
        UnitKind::S | UnitKind::Ms | UnitKind::Us | UnitKind::Min | UnitKind::H => {
            PhysicalCategory::Duration
        }
        UnitKind::MPerS | UnitKind::KmPerH | UnitKind::Mph => PhysicalCategory::Velocity,
        UnitKind::MPerS2 | UnitKind::G => PhysicalCategory::Acceleration,
        UnitKind::Rad | UnitKind::Deg => PhysicalCategory::Angle,
        UnitKind::RadPerS | UnitKind::DegPerS => PhysicalCategory::AngularVelocity,
        UnitKind::Kg | UnitKind::Gram | UnitKind::Lb => PhysicalCategory::Mass,
        UnitKind::N | UnitKind::KN => PhysicalCategory::Force,
        UnitKind::W | UnitKind::KW | UnitKind::MW => PhysicalCategory::Power,
        UnitKind::V | UnitKind::MVolt | UnitKind::KVolt => PhysicalCategory::Voltage,
        UnitKind::A | UnitKind::MA => PhysicalCategory::Current,
        UnitKind::Celsius | UnitKind::Fahrenheit | UnitKind::Kelvin => {
            PhysicalCategory::Temperature
        }
        UnitKind::Pa | UnitKind::KPa | UnitKind::Bar | UnitKind::Psi | UnitKind::Mbar => {
            PhysicalCategory::Pressure
        }
        UnitKind::Hz | UnitKind::KHz | UnitKind::MHz => PhysicalCategory::Frequency,
        UnitKind::Rh | UnitKind::PercentRh => PhysicalCategory::Humidity,
        UnitKind::Lux | UnitKind::Lx => PhysicalCategory::Illuminance,
        UnitKind::CdPerM2 | UnitKind::Nit => PhysicalCategory::Luminance,
        UnitKind::Ppm | UnitKind::Ppb => PhysicalCategory::Concentration,
        UnitKind::DB | UnitKind::DBA => PhysicalCategory::SoundLevel,
        UnitKind::MicroTesla | UnitKind::Gauss => PhysicalCategory::MagneticField,
        UnitKind::Rpm => PhysicalCategory::RotationalSpeed,
        UnitKind::NewtonMeter | UnitKind::Nm => PhysicalCategory::Torque,
        UnitKind::Joule | UnitKind::Wh | UnitKind::KWh => PhysicalCategory::Energy,
        UnitKind::Uvi => PhysicalCategory::UvIndex,
        UnitKind::Ph => PhysicalCategory::Ph,
        UnitKind::MicroSPerCm | UnitKind::MilliSPerCm | UnitKind::SPerM => {
            PhysicalCategory::Conductivity
        }
        UnitKind::UgPerM3 => PhysicalCategory::ParticulateMatter,
        UnitKind::Ntu | UnitKind::Fnu => PhysicalCategory::Turbidity,
        UnitKind::Ppt | UnitKind::Psu => PhysicalCategory::Salinity,
        UnitKind::MicroSvPerH | UnitKind::MilliSvPerH => PhysicalCategory::Radiation,
        UnitKind::PercentVwc | UnitKind::Vwc => PhysicalCategory::SoilMoisture,
    }
}

pub fn units_compatible(a: UnitKind, b: UnitKind) -> bool {
    if a == b {
        return true;
    }
    if a == UnitKind::None || b == UnitKind::None {
        return true;
    }
    unit_category(a) == unit_category(b)
}

pub fn unit_matches_named_type(type_name: &str, unit: UnitKind) -> bool {
    match type_name {
        "Distance" => unit_category(unit) == PhysicalCategory::Distance,
        "Duration" => unit_category(unit) == PhysicalCategory::Duration,
        "Velocity" => unit_category(unit) == PhysicalCategory::Velocity,
        "Acceleration" => unit_category(unit) == PhysicalCategory::Acceleration,
        "Angle" => unit_category(unit) == PhysicalCategory::Angle,
        "AngularVelocity" => unit_category(unit) == PhysicalCategory::AngularVelocity,
        "Mass" => unit_category(unit) == PhysicalCategory::Mass,
        "Force" => unit_category(unit) == PhysicalCategory::Force,
        "Power" => unit_category(unit) == PhysicalCategory::Power,
        "Voltage" => unit_category(unit) == PhysicalCategory::Voltage,
        "Current" => unit_category(unit) == PhysicalCategory::Current,
        "Temperature" => unit_category(unit) == PhysicalCategory::Temperature,
        "Pressure" => unit_category(unit) == PhysicalCategory::Pressure,
        "Humidity" => unit_category(unit) == PhysicalCategory::Humidity,
        "Illuminance" => unit_category(unit) == PhysicalCategory::Illuminance,
        "Luminance" => unit_category(unit) == PhysicalCategory::Luminance,
        "Concentration" => unit_category(unit) == PhysicalCategory::Concentration,
        "SoundLevel" => unit_category(unit) == PhysicalCategory::SoundLevel,
        "MagneticField" => unit_category(unit) == PhysicalCategory::MagneticField,
        "RotationalSpeed" => unit_category(unit) == PhysicalCategory::RotationalSpeed,
        "Torque" => unit_category(unit) == PhysicalCategory::Torque,
        "Energy" => unit_category(unit) == PhysicalCategory::Energy,
        "UvIndex" => unit_category(unit) == PhysicalCategory::UvIndex,
        "Ph" => unit_category(unit) == PhysicalCategory::Ph,
        "Conductivity" => unit_category(unit) == PhysicalCategory::Conductivity,
        "ParticulateMatter" => unit_category(unit) == PhysicalCategory::ParticulateMatter,
        "Turbidity" => unit_category(unit) == PhysicalCategory::Turbidity,
        "Salinity" => unit_category(unit) == PhysicalCategory::Salinity,
        "Radiation" => unit_category(unit) == PhysicalCategory::Radiation,
        "SoilMoisture" => unit_category(unit) == PhysicalCategory::SoilMoisture,
        _ => false,
    }
}

/// Convert `value` in `unit` to the canonical unit for its physical category.
pub fn to_canonical(value: f64, unit: UnitKind) -> (f64, UnitKind) {
    let category = unit_category(unit);
    let canonical = canonical_unit(category);
    (to_canonical_linear(value, unit), canonical)
}

pub fn canonical_unit(category: PhysicalCategory) -> UnitKind {
    match category {
        PhysicalCategory::Scalar => UnitKind::None,
        PhysicalCategory::Distance => UnitKind::M,
        PhysicalCategory::Duration => UnitKind::S,
        PhysicalCategory::Velocity => UnitKind::MPerS,
        PhysicalCategory::Acceleration => UnitKind::MPerS2,
        PhysicalCategory::Angle => UnitKind::Rad,
        PhysicalCategory::AngularVelocity => UnitKind::RadPerS,
        PhysicalCategory::Mass => UnitKind::Kg,
        PhysicalCategory::Force => UnitKind::N,
        PhysicalCategory::Power => UnitKind::W,
        PhysicalCategory::Voltage => UnitKind::V,
        PhysicalCategory::Current => UnitKind::A,
        PhysicalCategory::Temperature => UnitKind::Celsius,
        PhysicalCategory::Pressure => UnitKind::Pa,
        PhysicalCategory::Frequency => UnitKind::Hz,
        PhysicalCategory::Humidity => UnitKind::Rh,
        PhysicalCategory::Illuminance => UnitKind::Lux,
        PhysicalCategory::Luminance => UnitKind::CdPerM2,
        PhysicalCategory::Concentration => UnitKind::Ppm,
        PhysicalCategory::SoundLevel => UnitKind::DB,
        PhysicalCategory::MagneticField => UnitKind::MicroTesla,
        PhysicalCategory::RotationalSpeed => UnitKind::Rpm,
        PhysicalCategory::Torque => UnitKind::NewtonMeter,
        PhysicalCategory::Energy => UnitKind::Joule,
        PhysicalCategory::UvIndex => UnitKind::Uvi,
        PhysicalCategory::Ph => UnitKind::Ph,
        PhysicalCategory::Conductivity => UnitKind::MicroSPerCm,
        PhysicalCategory::ParticulateMatter => UnitKind::UgPerM3,
        PhysicalCategory::Turbidity => UnitKind::Ntu,
        PhysicalCategory::Salinity => UnitKind::Ppt,
        PhysicalCategory::Radiation => UnitKind::MicroSvPerH,
        PhysicalCategory::SoilMoisture => UnitKind::PercentVwc,
    }
}

pub fn convert_value(value: f64, from: UnitKind, to: UnitKind) -> Option<f64> {
    if from == to {
        return Some(value);
    }
    if !units_compatible(from, to) {
        return None;
    }
    let cat = unit_category(from);
    let in_canonical = to_canonical(value, from).0;
    Some(from_canonical(in_canonical, cat, to))
}

fn from_canonical(value: f64, category: PhysicalCategory, to: UnitKind) -> f64 {
    match category {
        PhysicalCategory::Distance => match to {
            UnitKind::M => value,
            UnitKind::Mm => value * 1000.0,
            UnitKind::Cm => value * 100.0,
            UnitKind::Km => value / 1000.0,
            UnitKind::Ft => value / 0.3048,
            UnitKind::In => value / 0.0254,
            _ => value,
        },
        PhysicalCategory::Duration => match to {
            UnitKind::S => value,
            UnitKind::Ms => value * 1000.0,
            UnitKind::Us => value * 1_000_000.0,
            UnitKind::Min => value / 60.0,
            UnitKind::H => value / 3600.0,
            _ => value,
        },
        PhysicalCategory::Velocity => match to {
            UnitKind::MPerS => value,
            UnitKind::KmPerH => value * 3.6,
            UnitKind::Mph => value * 2.236_936_292_054_4,
            _ => value,
        },
        PhysicalCategory::Acceleration => match to {
            UnitKind::MPerS2 => value,
            UnitKind::G => value / 9.806_65,
            _ => value,
        },
        PhysicalCategory::Angle => match to {
            UnitKind::Rad => value,
            UnitKind::Deg => value / DEG_TO_RAD,
            _ => value,
        },
        PhysicalCategory::AngularVelocity => match to {
            UnitKind::RadPerS => value,
            UnitKind::DegPerS => value / DEG_TO_RAD,
            _ => value,
        },
        PhysicalCategory::Mass => match to {
            UnitKind::Kg => value,
            UnitKind::Gram => value * 1000.0,
            UnitKind::Lb => value / 0.453_592_37,
            _ => value,
        },
        PhysicalCategory::Force => match to {
            UnitKind::N => value,
            UnitKind::KN => value / 1000.0,
            _ => value,
        },
        PhysicalCategory::Power => match to {
            UnitKind::W => value,
            UnitKind::KW => value / 1000.0,
            UnitKind::MW => value / 1_000_000.0,
            _ => value,
        },
        PhysicalCategory::Voltage => match to {
            UnitKind::V => value,
            UnitKind::MVolt => value * 1000.0,
            UnitKind::KVolt => value / 1000.0,
            _ => value,
        },
        PhysicalCategory::Current => match to {
            UnitKind::A => value,
            UnitKind::MA => value * 1000.0,
            _ => value,
        },
        PhysicalCategory::Temperature => match to {
            UnitKind::Celsius => value,
            UnitKind::Fahrenheit => value * 9.0 / 5.0 + 32.0,
            UnitKind::Kelvin => value + 273.15,
            _ => value,
        },
        PhysicalCategory::Pressure => match to {
            UnitKind::Pa => value,
            UnitKind::KPa => value / 1000.0,
            UnitKind::Bar => value / 100_000.0,
            UnitKind::Mbar => value / 100.0,
            UnitKind::Psi => value / 6_894.757_293_168,
            _ => value,
        },
        PhysicalCategory::Frequency => match to {
            UnitKind::Hz => value,
            UnitKind::KHz => value / 1000.0,
            UnitKind::MHz => value / 1_000_000.0,
            _ => value,
        },
        PhysicalCategory::Humidity => match to {
            UnitKind::Rh | UnitKind::PercentRh => value,
            _ => value,
        },
        PhysicalCategory::Illuminance => match to {
            UnitKind::Lux | UnitKind::Lx => value,
            _ => value,
        },
        PhysicalCategory::Luminance => match to {
            UnitKind::CdPerM2 | UnitKind::Nit => value,
            _ => value,
        },
        PhysicalCategory::Concentration => match to {
            UnitKind::Ppm => value,
            UnitKind::Ppb => value * 1000.0,
            _ => value,
        },
        PhysicalCategory::SoundLevel => match to {
            UnitKind::DB | UnitKind::DBA => value,
            _ => value,
        },
        PhysicalCategory::MagneticField => match to {
            UnitKind::MicroTesla => value,
            UnitKind::Gauss => value / 100.0,
            _ => value,
        },
        PhysicalCategory::RotationalSpeed => match to {
            UnitKind::Rpm => value,
            _ => value,
        },
        PhysicalCategory::Torque => match to {
            UnitKind::NewtonMeter | UnitKind::Nm => value,
            _ => value,
        },
        PhysicalCategory::Energy => match to {
            UnitKind::Joule => value,
            UnitKind::Wh => value / 3600.0,
            UnitKind::KWh => value / 3_600_000.0,
            _ => value,
        },
        PhysicalCategory::UvIndex => match to {
            UnitKind::Uvi => value,
            _ => value,
        },
        PhysicalCategory::Ph => match to {
            UnitKind::Ph => value,
            _ => value,
        },
        PhysicalCategory::Conductivity => match to {
            UnitKind::MicroSPerCm => value,
            UnitKind::MilliSPerCm => value / 1000.0,
            UnitKind::SPerM => value / 10_000.0,
            _ => value,
        },
        PhysicalCategory::ParticulateMatter => match to {
            UnitKind::UgPerM3 => value,
            _ => value,
        },
        PhysicalCategory::Turbidity => match to {
            UnitKind::Ntu | UnitKind::Fnu => value,
            _ => value,
        },
        PhysicalCategory::Salinity => match to {
            UnitKind::Ppt | UnitKind::Psu => value,
            _ => value,
        },
        PhysicalCategory::Radiation => match to {
            UnitKind::MicroSvPerH => value,
            UnitKind::MilliSvPerH => value / 1000.0,
            _ => value,
        },
        PhysicalCategory::SoilMoisture => match to {
            UnitKind::PercentVwc | UnitKind::Vwc => value,
            _ => value,
        },
        PhysicalCategory::Scalar => value,
    }
}

fn to_canonical_linear(value: f64, unit: UnitKind) -> f64 {
    match unit {
        UnitKind::M => value,
        UnitKind::Mm => value / 1000.0,
        UnitKind::Cm => value / 100.0,
        UnitKind::Km => value * 1000.0,
        UnitKind::Ft => value * 0.3048,
        UnitKind::In => value * 0.0254,
        UnitKind::S => value,
        UnitKind::Ms => value / 1000.0,
        UnitKind::Us => value / 1_000_000.0,
        UnitKind::Min => value * 60.0,
        UnitKind::H => value * 3600.0,
        UnitKind::MPerS => value,
        UnitKind::KmPerH => value / 3.6,
        UnitKind::Mph => value / 2.236_936_292_054_4,
        UnitKind::MPerS2 => value,
        UnitKind::G => value * 9.806_65,
        UnitKind::Rad => value,
        UnitKind::Deg => value * DEG_TO_RAD,
        UnitKind::RadPerS => value,
        UnitKind::DegPerS => value * DEG_TO_RAD,
        UnitKind::Kg => value,
        UnitKind::Gram => value / 1000.0,
        UnitKind::Lb => value * 0.453_592_37,
        UnitKind::N => value,
        UnitKind::KN => value * 1000.0,
        UnitKind::W => value,
        UnitKind::KW => value * 1000.0,
        UnitKind::MW => value * 1_000_000.0,
        UnitKind::V => value,
        UnitKind::MVolt => value / 1000.0,
        UnitKind::KVolt => value * 1000.0,
        UnitKind::A => value,
        UnitKind::MA => value / 1000.0,
        UnitKind::Celsius => value,
        UnitKind::Fahrenheit => (value - 32.0) * 5.0 / 9.0,
        UnitKind::Kelvin => value - 273.15,
        UnitKind::Pa => value,
        UnitKind::KPa => value * 1000.0,
        UnitKind::Bar => value * 100_000.0,
        UnitKind::Mbar => value * 100.0,
        UnitKind::Psi => value * 6_894.757_293_168,
        UnitKind::Hz => value,
        UnitKind::KHz => value * 1000.0,
        UnitKind::MHz => value * 1_000_000.0,
        UnitKind::Rh | UnitKind::PercentRh => value,
        UnitKind::Lux | UnitKind::Lx => value,
        UnitKind::CdPerM2 | UnitKind::Nit => value,
        UnitKind::Ppm => value,
        UnitKind::Ppb => value / 1000.0,
        UnitKind::DB | UnitKind::DBA => value,
        UnitKind::MicroTesla => value,
        UnitKind::Gauss => value * 100.0,
        UnitKind::Rpm => value,
        UnitKind::NewtonMeter | UnitKind::Nm => value,
        UnitKind::Joule => value,
        UnitKind::Wh => value * 3600.0,
        UnitKind::KWh => value * 3_600_000.0,
        UnitKind::Uvi => value,
        UnitKind::Ph => value,
        UnitKind::MicroSPerCm => value,
        UnitKind::MilliSPerCm => value * 1000.0,
        UnitKind::SPerM => value * 10_000.0,
        UnitKind::UgPerM3 => value,
        UnitKind::Ntu | UnitKind::Fnu => value,
        UnitKind::Ppt | UnitKind::Psu => value,
        UnitKind::MicroSvPerH => value,
        UnitKind::MilliSvPerH => value * 1000.0,
        UnitKind::PercentVwc | UnitKind::Vwc => value,
        UnitKind::None => value,
    }
}

/// Align two unit values to the left operand's unit for binary ops.
pub fn align_for_binary(
    left: f64,
    left_unit: UnitKind,
    right: f64,
    right_unit: UnitKind,
) -> Option<(f64, f64, UnitKind)> {
    if !units_compatible(left_unit, right_unit) {
        return None;
    }
    if left_unit == right_unit {
        return Some((left, right, left_unit));
    }
    let right_in_left = convert_value(right, right_unit, left_unit)?;
    Some((left, right_in_left, left_unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_distance_units() {
        assert!((convert_value(100.0, UnitKind::Cm, UnitKind::M).unwrap() - 1.0).abs() < 1e-9);
        assert!((convert_value(1.0, UnitKind::Km, UnitKind::M).unwrap() - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn converts_mixed_duration_addition() {
        let (l, r, _) = align_for_binary(500.0, UnitKind::Ms, 0.5, UnitKind::S).unwrap();
        assert!((l - 500.0).abs() < 1e-9);
        assert!((r - 500.0).abs() < 1e-9);
    }

    #[test]
    fn converts_temperature() {
        assert!(
            (convert_value(32.0, UnitKind::Fahrenheit, UnitKind::Celsius).unwrap()).abs() < 1e-9
        );
        assert!(
            (convert_value(0.0, UnitKind::Celsius, UnitKind::Kelvin).unwrap() - 273.15).abs()
                < 1e-9
        );
    }

    #[test]
    fn rejects_incompatible_units() {
        assert!(!units_compatible(UnitKind::M, UnitKind::Kg));
        assert!(convert_value(1.0, UnitKind::M, UnitKind::Kg).is_none());
        assert!(!units_compatible(UnitKind::Rh, UnitKind::Lux));
    }

    #[test]
    fn converts_sensor_units() {
        assert!(
            (convert_value(65.0, UnitKind::PercentRh, UnitKind::Rh).unwrap() - 65.0).abs() < 1e-9
        );
        assert!(
            (convert_value(1000.0, UnitKind::Lux, UnitKind::Lx).unwrap() - 1000.0).abs() < 1e-9
        );
        assert!((convert_value(1000.0, UnitKind::Ppb, UnitKind::Ppm).unwrap() - 1.0).abs() < 1e-9);
        assert!(
            (convert_value(1.0, UnitKind::Gauss, UnitKind::MicroTesla).unwrap() - 100.0).abs()
                < 1e-9
        );
        assert!((convert_value(1.0, UnitKind::Wh, UnitKind::Joule).unwrap() - 3600.0).abs() < 1e-6);
        assert!(
            (convert_value(1.0, UnitKind::MilliSPerCm, UnitKind::MicroSPerCm).unwrap() - 1000.0)
                .abs()
                < 1e-9
        );
        assert!(
            (convert_value(1.0, UnitKind::MilliSvPerH, UnitKind::MicroSvPerH).unwrap() - 1000.0)
                .abs()
                < 1e-9
        );
    }
}

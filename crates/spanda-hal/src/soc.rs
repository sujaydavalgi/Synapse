//! soc support for Spanda.
//!
use spanda_runtime::hal_config::HalMemberConfig;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocCapability {
    Gpio,
    I2c,
    Spi,
    Uart,
    Pwm,
    Adc,
    Wifi,
    Ble,
    Gpu,
    Cuda,
}

#[derive(Debug, Clone)]
pub struct SocProfile {
    pub name: String,
    pub vendor: String,
    pub architecture: String,
    pub clock_mhz: f64,
    pub ram_mb: f64,
    pub gpio_pins: u32,
    pub i2c_buses: u32,
    pub spi_buses: u32,
    pub uart_ports: u32,
    pub adc_channels: u32,
    pub pwm_channels: u32,
    pub capabilities: Vec<SocCapability>,
}

#[derive(Debug, Clone)]
pub struct SocValidationError {
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

fn build_profiles() -> HashMap<&'static str, SocProfile> {
    // Build profiles.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<&'static str, SocProfile>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_hal::soc::build_profiles();

    // Produce from as the result.
    HashMap::from([
        (
            "RaspberryPi4",
            SocProfile {
                name: "RaspberryPi4".into(),
                vendor: "Broadcom".into(),
                architecture: "aarch64".into(),
                clock_mhz: 1500.0,
                ram_mb: 4096.0,
                gpio_pins: 40,
                i2c_buses: 2,
                spi_buses: 2,
                uart_ports: 2,
                adc_channels: 0,
                pwm_channels: 2,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Wifi,
                    SocCapability::Ble,
                ],
            },
        ),
        (
            "RaspberryPi5",
            SocProfile {
                name: "RaspberryPi5".into(),
                vendor: "Broadcom".into(),
                architecture: "aarch64".into(),
                clock_mhz: 2400.0,
                ram_mb: 8192.0,
                gpio_pins: 40,
                i2c_buses: 3,
                spi_buses: 2,
                uart_ports: 2,
                adc_channels: 0,
                pwm_channels: 4,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Wifi,
                    SocCapability::Ble,
                ],
            },
        ),
        (
            "ESP32",
            SocProfile {
                name: "ESP32".into(),
                vendor: "Espressif".into(),
                architecture: "xtensa".into(),
                clock_mhz: 240.0,
                ram_mb: 4.0,
                gpio_pins: 34,
                i2c_buses: 2,
                spi_buses: 3,
                uart_ports: 3,
                adc_channels: 18,
                pwm_channels: 16,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Adc,
                    SocCapability::Wifi,
                    SocCapability::Ble,
                ],
            },
        ),
        (
            "ESP32S3",
            SocProfile {
                name: "ESP32S3".into(),
                vendor: "Espressif".into(),
                architecture: "xtensa".into(),
                clock_mhz: 240.0,
                ram_mb: 8.0,
                gpio_pins: 45,
                i2c_buses: 2,
                spi_buses: 4,
                uart_ports: 3,
                adc_channels: 20,
                pwm_channels: 16,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Adc,
                    SocCapability::Wifi,
                    SocCapability::Ble,
                ],
            },
        ),
        (
            "STM32F4",
            SocProfile {
                name: "STM32F4".into(),
                vendor: "STMicroelectronics".into(),
                architecture: "arm_cortex_m4".into(),
                clock_mhz: 168.0,
                ram_mb: 0.192,
                gpio_pins: 82,
                i2c_buses: 3,
                spi_buses: 3,
                uart_ports: 6,
                adc_channels: 16,
                pwm_channels: 12,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Adc,
                ],
            },
        ),
        (
            "STM32H7",
            SocProfile {
                name: "STM32H7".into(),
                vendor: "STMicroelectronics".into(),
                architecture: "arm_cortex_m7".into(),
                clock_mhz: 480.0,
                ram_mb: 1.0,
                gpio_pins: 114,
                i2c_buses: 4,
                spi_buses: 6,
                uart_ports: 8,
                adc_channels: 36,
                pwm_channels: 20,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Adc,
                ],
            },
        ),
        (
            "JetsonNano",
            SocProfile {
                name: "JetsonNano".into(),
                vendor: "NVIDIA".into(),
                architecture: "aarch64".into(),
                clock_mhz: 1479.0,
                ram_mb: 4096.0,
                gpio_pins: 40,
                i2c_buses: 2,
                spi_buses: 2,
                uart_ports: 2,
                adc_channels: 0,
                pwm_channels: 2,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Gpu,
                    SocCapability::Cuda,
                ],
            },
        ),
        (
            "JetsonOrin",
            SocProfile {
                name: "JetsonOrin".into(),
                vendor: "NVIDIA".into(),
                architecture: "aarch64".into(),
                clock_mhz: 2200.0,
                ram_mb: 32768.0,
                gpio_pins: 40,
                i2c_buses: 3,
                spi_buses: 2,
                uart_ports: 3,
                adc_channels: 0,
                pwm_channels: 4,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Gpu,
                    SocCapability::Cuda,
                    SocCapability::Wifi,
                ],
            },
        ),
        (
            "ArduinoUno",
            SocProfile {
                name: "ArduinoUno".into(),
                vendor: "Arduino".into(),
                architecture: "avr".into(),
                clock_mhz: 16.0,
                ram_mb: 0.002,
                gpio_pins: 20,
                i2c_buses: 1,
                spi_buses: 1,
                uart_ports: 1,
                adc_channels: 6,
                pwm_channels: 6,
                capabilities: vec![
                    SocCapability::Gpio,
                    SocCapability::I2c,
                    SocCapability::Spi,
                    SocCapability::Uart,
                    SocCapability::Pwm,
                    SocCapability::Adc,
                ],
            },
        ),
    ])
}

pub fn get_soc_profile(name: &str) -> Option<SocProfile> {
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_hal::soc::get_soc_profile(name);

    // Produce remove as the result.
    build_profiles().remove(name)
}

pub fn list_soc_profiles() -> Vec<SocProfile> {
    // List soc profiles.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Vec<SocProfile>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_hal::soc::list_soc_profiles();

    // Collect filtered entries into a new list.
    build_profiles().into_values().collect()
}

pub fn validate_hal_against_soc(
    profile: &SocProfile,
    hal_members: &[HalMemberConfig],
) -> Vec<SocValidationError> {
    // Validate hal against soc.
    //
    // Parameters:
    // - `profile` — input value
    // - `hal_members` — input value
    //
    // Returns:
    // Vec<SocValidationError>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_hal::soc::validate_hal_against_soc(profile, hal_members);

    // Create mutable errors for accumulating results.
    let mut errors = Vec::new();
    let mut i2c_count = 0u32;
    let mut spi_count = 0u32;
    let mut uart_count = 0u32;
    let mut adc_count = 0u32;
    let mut pwm_count = 0u32;

    // Process each hal member.
    for m in hal_members {
        // Match on m and handle each case.
        match m {
            HalMemberConfig::I2c { .. } => {
                i2c_count += 1;

                // Take this path when i2c count > profile.i2c buses.
                if i2c_count > profile.i2c_buses {
                    errors.push(SocValidationError {
                        message: format!(
                            "SoC {} supports max {} I2C bus(es)",
                            profile.name, profile.i2c_buses
                        ),
                        line: None,
                        column: None,
                    });
                }

                // Check membership before continuing.
                if !profile.capabilities.contains(&SocCapability::I2c) {
                    errors.push(SocValidationError {
                        message: format!("SoC {} does not support I2C", profile.name),
                        line: None,
                        column: None,
                    });
                }
            }
            HalMemberConfig::Spi { .. } => {
                spi_count += 1;

                // Take this path when spi count > profile.spi buses.
                if spi_count > profile.spi_buses {
                    errors.push(SocValidationError {
                        message: format!(
                            "SoC {} supports max {} SPI bus(es)",
                            profile.name, profile.spi_buses
                        ),
                        line: None,
                        column: None,
                    });
                }
            }
            HalMemberConfig::Uart { .. } => {
                uart_count += 1;

                // Take this path when uart count > profile.uart ports.
                if uart_count > profile.uart_ports {
                    errors.push(SocValidationError {
                        message: format!(
                            "SoC {} supports max {} UART port(s)",
                            profile.name, profile.uart_ports
                        ),
                        line: None,
                        column: None,
                    });
                }
            }
            HalMemberConfig::Adc { .. } => {
                adc_count += 1;

                // Take this path when adc count > profile.adc channels.
                if adc_count > profile.adc_channels {
                    errors.push(SocValidationError {
                        message: format!(
                            "SoC {} supports max {} ADC channel(s)",
                            profile.name, profile.adc_channels
                        ),
                        line: None,
                        column: None,
                    });
                }

                // Check membership before continuing.
                if !profile.capabilities.contains(&SocCapability::Adc) {
                    errors.push(SocValidationError {
                        message: format!("SoC {} does not support ADC", profile.name),
                        line: None,
                        column: None,
                    });
                }
            }
            HalMemberConfig::Pwm { .. } => {
                pwm_count += 1;

                // Take this path when pwm count > profile.pwm channels.
                if pwm_count > profile.pwm_channels {
                    errors.push(SocValidationError {
                        message: format!(
                            "SoC {} supports max {} PWM channel(s)",
                            profile.name, profile.pwm_channels
                        ),
                        line: None,
                        column: None,
                    });
                }
            }
            HalMemberConfig::Gpio { pin, .. } => {
                // Take this path when *pin as u32 >= profile.gpio pins.
                if *pin as u32 >= profile.gpio_pins {
                    errors.push(SocValidationError {
                        message: format!(
                            "GPIO pin {} exceeds {} limit ({} pins)",
                            pin, profile.name, profile.gpio_pins
                        ),
                        line: None,
                        column: None,
                    });
                }
            }
        }
    }
    errors
}

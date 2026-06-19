use crate::ast::{GpioDirection, HalMemberDecl};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalBusKind {
    I2c,
    Spi,
    Uart,
    Usb,
    Ethernet,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HalMemberConfig {
    I2c { name: String, address: f64 },
    Spi {
        name: String,
        bus: f64,
        cs_pin: Option<f64>,
    },
    Gpio {
        name: String,
        pin: f64,
        direction: GpioDirection,
    },
    Pwm {
        name: String,
        pin: f64,
        frequency_hz: f64,
    },
    Uart {
        name: String,
        device: String,
        baud: f64,
    },
    Adc { name: String, channel: f64 },
}

pub trait HalBackend {
    fn configure(&mut self, members: &[HalMemberConfig]);
    fn read_gpio(&self, name: &str) -> bool;
    fn write_gpio(&mut self, name: &str, value: bool);
    fn read_i2c(&self, name: &str, register: u8, length: usize) -> Vec<u8>;
    fn write_i2c(&mut self, name: &str, register: u8, data: &[u8]);
    fn transfer_spi(&self, name: &str, data: &[u8]) -> Vec<u8>;
    fn read_uart(&self, name: &str) -> String;
    fn read_adc(&self, name: &str) -> f64;
    fn set_pwm(&mut self, name: &str, duty_cycle: f64);
    fn get_member(&self, name: &str) -> Option<HalMemberConfig>;
    fn list_members(&self) -> Vec<HalMemberConfig>;
}

pub struct SimHalBackend {
    members: HashMap<String, HalMemberConfig>,
    gpio_state: HashMap<String, bool>,
    i2c_registers: HashMap<String, HashMap<u8, u8>>,
    adc_values: HashMap<String, f64>,
    pwm_duty: HashMap<String, f64>,
    uart_buffers: HashMap<String, String>,
}

impl SimHalBackend {
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
            gpio_state: HashMap::new(),
            i2c_registers: HashMap::new(),
            adc_values: HashMap::new(),
            pwm_duty: HashMap::new(),
            uart_buffers: HashMap::new(),
        }
    }

    pub fn simulate_uart_data(&mut self, name: &str, data: &str) {
        self.uart_buffers.insert(name.to_string(), data.to_string());
    }

    pub fn set_adc_value(&mut self, name: &str, value: f64) {
        self.adc_values.insert(name.to_string(), value);
    }

    pub fn seed_imu_registers(&mut self, bus_name: &str, yaw: f64) {
        let yaw_int = yaw.floor() as i32 * 100;
        self.write_i2c(
            bus_name,
            0x1a,
            &[(yaw_int & 0xff) as u8, ((yaw_int >> 8) & 0xff) as u8],
        );
    }
}

impl Default for SimHalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl HalBackend for SimHalBackend {
    fn configure(&mut self, members: &[HalMemberConfig]) {
        self.members.clear();
        self.gpio_state.clear();
        self.i2c_registers.clear();
        self.adc_values.clear();
        self.pwm_duty.clear();
        self.uart_buffers.clear();

        for m in members {
            let name = match m {
                HalMemberConfig::I2c { name, .. }
                | HalMemberConfig::Spi { name, .. }
                | HalMemberConfig::Gpio { name, .. }
                | HalMemberConfig::Pwm { name, .. }
                | HalMemberConfig::Uart { name, .. }
                | HalMemberConfig::Adc { name, .. } => name.clone(),
            };
            self.members.insert(name.clone(), m.clone());
            match m {
                HalMemberConfig::Gpio { .. } => {
                    self.gpio_state.insert(name, false);
                }
                HalMemberConfig::Adc { .. } => {
                    self.adc_values.insert(name, 0.0);
                }
                HalMemberConfig::Pwm { .. } => {
                    self.pwm_duty.insert(name, 0.0);
                }
                HalMemberConfig::Uart { .. } => {
                    self.uart_buffers.insert(name, String::new());
                }
                HalMemberConfig::I2c { .. } => {
                    self.i2c_registers.insert(name, HashMap::new());
                }
                _ => {}
            }
        }
    }

    fn read_gpio(&self, name: &str) -> bool {
        self.gpio_state.get(name).copied().unwrap_or(false)
    }

    fn write_gpio(&mut self, name: &str, value: bool) {
        self.gpio_state.insert(name.to_string(), value);
    }

    fn read_i2c(&self, name: &str, register: u8, length: usize) -> Vec<u8> {
        let regs = self.i2c_registers.get(name);
        let mut result = Vec::new();
        for i in 0..length {
            let val = regs
                .and_then(|r| r.get(&(register + i as u8)))
                .copied()
                .unwrap_or(0);
            result.push(val);
        }
        result
    }

    fn write_i2c(&mut self, name: &str, register: u8, data: &[u8]) {
        let regs = self
            .i2c_registers
            .entry(name.to_string())
            .or_default();
        for (i, &byte) in data.iter().enumerate() {
            regs.insert(register + i as u8, byte);
        }
    }

    fn transfer_spi(&self, _name: &str, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| (b ^ 0xff) & 0xff).collect()
    }

    fn read_uart(&self, name: &str) -> String {
        self.uart_buffers.get(name).cloned().unwrap_or_default()
    }

    fn read_adc(&self, name: &str) -> f64 {
        self.adc_values.get(name).copied().unwrap_or(0.0)
    }

    fn set_pwm(&mut self, name: &str, duty_cycle: f64) {
        self.pwm_duty
            .insert(name.to_string(), duty_cycle.clamp(0.0, 1.0));
    }

    fn get_member(&self, name: &str) -> Option<HalMemberConfig> {
        self.members.get(name).cloned()
    }

    fn list_members(&self) -> Vec<HalMemberConfig> {
        self.members.values().cloned().collect()
    }
}

pub fn create_sim_hal() -> SimHalBackend {
    SimHalBackend::new()
}

pub fn hal_member_from_decl(decl: &HalMemberDecl) -> HalMemberConfig {
    match decl {
        HalMemberDecl::HalI2cDecl { name, address, .. } => HalMemberConfig::I2c {
            name: name.clone(),
            address: *address,
        },
        HalMemberDecl::HalSpiDecl {
            name,
            bus,
            cs_pin,
            ..
        } => HalMemberConfig::Spi {
            name: name.clone(),
            bus: *bus,
            cs_pin: *cs_pin,
        },
        HalMemberDecl::HalGpioDecl {
            name,
            direction,
            pin,
            ..
        } => HalMemberConfig::Gpio {
            name: name.clone(),
            pin: *pin,
            direction: *direction,
        },
        HalMemberDecl::HalPwmDecl {
            name,
            pin,
            frequency_hz,
            ..
        } => HalMemberConfig::Pwm {
            name: name.clone(),
            pin: *pin,
            frequency_hz: *frequency_hz,
        },
        HalMemberDecl::HalUartDecl {
            name,
            device,
            baud,
            ..
        } => HalMemberConfig::Uart {
            name: name.clone(),
            device: device.clone(),
            baud: *baud,
        },
        HalMemberDecl::HalAdcDecl { name, channel, .. } => HalMemberConfig::Adc {
            name: name.clone(),
            channel: *channel,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulates_i2c() {
        let mut hal = create_sim_hal();
        hal.configure(&[HalMemberConfig::I2c {
            name: "bus".to_string(),
            address: 104.0,
        }]);
        hal.write_i2c("bus", 0x10, &[0xab, 0xcd]);
        assert_eq!(hal.read_i2c("bus", 0x10, 2), vec![0xab, 0xcd]);
    }
}

//! hal support for Spanda.
//!
use crate::ast::HalMemberDecl;
pub use spanda_runtime::hal_config::HalMemberConfig;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalBusKind {
    I2c,
    Spi,
    Uart,
    Usb,
    Ethernet,
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
        // Create a new instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::hal::new();

        // Assemble the struct fields and return it.
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
        // Simulate uart data.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `data` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.simulate_uart_data(name, data);

        // Append into self.
        self.uart_buffers.insert(name.to_string(), data.to_string());
    }

    pub fn set_adc_value(&mut self, name: &str, value: f64) {
        // Set adc value.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_adc_value(name, value);

        // Append into self.
        self.adc_values.insert(name.to_string(), value);
    }

    pub fn seed_imu_registers(&mut self, bus_name: &str, yaw: f64) {
        // Seed imu registers.
        //
        // Parameters:
        // - `self` — method receiver
        // - `bus_name` — input value
        // - `yaw` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.seed_imu_registers(bus_name, yaw);

        // Compute yaw int for the following logic.
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
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::hal::default();

        // Build the result via new.
        Self::new()
    }
}

impl HalBackend for SimHalBackend {
    fn configure(&mut self, members: &[HalMemberConfig]) {
        // Configure.
        //
        // Parameters:
        // - `self` — method receiver
        // - `members` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.configure(members);

        // Call clear on the current instance.
        self.members.clear();
        self.gpio_state.clear();
        self.i2c_registers.clear();
        self.adc_values.clear();
        self.pwm_duty.clear();
        self.uart_buffers.clear();

        // Process each member.
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

            // Match on m and handle each case.
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
        // Read gpio.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.read_gpio(name);

        // Call get on the current instance.
        self.gpio_state.get(name).copied().unwrap_or(false)
    }

    fn write_gpio(&mut self, name: &str, value: bool) {
        // Write gpio.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.write_gpio(name, value);

        // Append into self.
        self.gpio_state.insert(name.to_string(), value);
    }

    fn read_i2c(&self, name: &str, register: u8, length: usize) -> Vec<u8> {
        // Read i2c.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `register` — input value
        // - `length` — input value
        //
        // Returns:
        // Vec<u8>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.read_i2c(name, register, length);

        // Compute regs for the following logic.
        let regs = self.i2c_registers.get(name);
        let mut result = Vec::new();

        // Iterate over length.
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
        // Write i2c.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `register` — input value
        // - `data` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.write_i2c(name, register, data);

        // Compute regs for the following logic.
        let regs = self.i2c_registers.entry(name.to_string()).or_default();

        // Iterate over enumerate with destructured elements.
        for (i, &byte) in data.iter().enumerate() {
            regs.insert(register + i as u8, byte);
        }
    }

    fn transfer_spi(&self, _name: &str, data: &[u8]) -> Vec<u8> {
        // Transfer spi.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_name` — input value
        // - `data` — input value
        //
        // Returns:
        // Vec<u8>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.transfer_spi(_name, data);

        // Collect filtered entries into a new list.
        data.iter().map(|b| b ^ 0xff).collect()
    }

    fn read_uart(&self, name: &str) -> String {
        // Read uart.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.read_uart(name);

        // Call get on the current instance.
        self.uart_buffers.get(name).cloned().unwrap_or_default()
    }

    fn read_adc(&self, name: &str) -> f64 {
        // Read adc.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Numeric result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.read_adc(name);

        // Call get on the current instance.
        self.adc_values.get(name).copied().unwrap_or(0.0)
    }

    fn set_pwm(&mut self, name: &str, duty_cycle: f64) {
        // Set pwm.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `duty_cycle` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_pwm(name, duty_cycle);

        // Call pwm duty on the current instance.
        self.pwm_duty
            .insert(name.to_string(), duty_cycle.clamp(0.0, 1.0));
    }

    fn get_member(&self, name: &str) -> Option<HalMemberConfig> {
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get_member(name);

        // Call get on the current instance.
        self.members.get(name).cloned()
    }

    fn list_members(&self) -> Vec<HalMemberConfig> {
        // List members.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<HalMemberConfig>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.list_members();

        // Collect filtered entries into a new list.
        self.members.values().cloned().collect()
    }
}

pub fn create_sim_hal() -> SimHalBackend {
    // Create sim hal.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SimHalBackend.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hal::create_sim_hal();

    // Produce new as the result.
    SimHalBackend::new()
}

pub fn hal_member_from_decl(decl: &HalMemberDecl) -> HalMemberConfig {
    // Hal member from decl.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // HalMemberConfig.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hal::hal_member_from_decl(decl);

    // Match on decl and handle each case.
    match decl {
        HalMemberDecl::HalI2cDecl { name, address, .. } => HalMemberConfig::I2c {
            name: name.clone(),
            address: *address,
        },
        HalMemberDecl::HalSpiDecl {
            name, bus, cs_pin, ..
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
            name, device, baud, ..
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
        // Simulates i2c.
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
        // let result = spanda_core::hal::simulates_i2c();

        let mut hal = create_sim_hal();
        hal.configure(&[HalMemberConfig::I2c {
            name: "bus".to_string(),
            address: 104.0,
        }]);
        hal.write_i2c("bus", 0x10, &[0xab, 0xcd]);
        assert_eq!(hal.read_i2c("bus", 0x10, 2), vec![0xab, 0xcd]);
    }
}

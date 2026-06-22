//! HAL member configuration shared by provider and board-access shims.
//!
use spanda_ast::nodes::GpioDirection;

/// Declared peripheral binding for I2C, SPI, GPIO, UART, PWM, or ADC members.
#[derive(Debug, Clone, PartialEq)]
pub enum HalMemberConfig {
    I2c {
        name: String,
        address: f64,
    },
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
    Adc {
        name: String,
        channel: f64,
    },
}

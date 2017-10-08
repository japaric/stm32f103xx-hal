//! General Purpose I/O
//!
//! -  PB0
//! -  PB1
//! -  PB3
//! -  PB4
//! -  PB5
//! -  PB6
//! -  PB7
//! -  PB8
//! -  PB9
//! - PB10
//! - PB11
//! - PB12
//! - PB13
//! - PB14
//! - PB15

use stm32f103xx::{GPIOA, GPIOB, GPIOC, GPIOD, GPIOE, RCC};

/// GPIO Ports Enum
pub enum GPIOPort {
   /// Port A
   A,
   /// Port B
   B,
   /// Port C
   C,
   /// Port D
   D,
   /// Port E
   E,
}

/// GPIO Peripheral Trait
pub trait GPIO {
    /// Associated GPIO Port
    const PORT: GPIOPort;
}

impl GPIO for GPIOA {
    const PORT: GPIOPort = GPIOPort::A;
}
impl GPIO for GPIOB {
    const PORT: GPIOPort = GPIOPort::B;
} 
impl GPIO for GPIOC {
    const PORT: GPIOPort = GPIOPort::C;
}
impl GPIO for GPIOD {
    const PORT: GPIOPort = GPIOPort::D;
}
impl GPIO for GPIOE {
    const PORT: GPIOPort = GPIOPort::E;
}

/// Initializes the digital outputs, enabled associated I/O port clock
pub fn init<T: GPIO>(_gpio: &T, rcc: &RCC) {
    match T::PORT {
        GPIOPort::A => rcc.apb2enr.modify(|_, w| w.iopaen().enabled()),
        GPIOPort::B => rcc.apb2enr.modify(|_, w| w.iopben().enabled()),
        GPIOPort::C => rcc.apb2enr.modify(|_, w| w.iopcen().enabled()),
        GPIOPort::D => rcc.apb2enr.modify(|_, w| w.iopden().enabled()),
        GPIOPort::E => rcc.apb2enr.modify(|_, w| w.iopeen().enabled()),
    }
}

/// Possible modes for a GPIO pin
pub enum GPIOMode {
    /// Input mode (reset state)
    Input(InputConfig),
    /// Output mode, max speed 10 MHz.
    Output(OutputConfig),
    /// Output mode, max speed 2 MHz.
    Output2(OutputConfig),
    /// Output mode, max speed 50 MHz
    Output50(OutputConfig),
}

impl GPIOMode {
    /// Output mode, general purpose push-pull
    pub const OUTPUT: GPIOMode = GPIOMode::Output(OutputConfig::GeneralPurposePushPull);
    /// Input mode, pull-up
    pub const INPUT_PULL_UP: GPIOMode = GPIOMode::Input(InputConfig::PullUp);
    /// Input mode, pull-down
    pub const INPUT_PULL_DOWN: GPIOMode = GPIOMode::Input(InputConfig::PullDown);
}

fn mode_value(mode: &GPIOMode) -> u8 {
    match *mode {
        GPIOMode::Input(_) => 0b00,
        GPIOMode::Output(_) => 0b01,
        GPIOMode::Output2(_) => 0b10,
        GPIOMode::Output50(_) => 0b11,
    }
}

/// Possible configurations for a GPIO pin in input mode
pub enum InputConfig {
    /// Analog mode
    Analog,
    /// Floating input (reset state)
    Floating,
    /// Input with pull-up
    PullUp,
    /// Input with pull-down
    PullDown
}

fn input_cnf(config: &InputConfig) -> u8 {
    match *config {
        InputConfig::Analog => 0b00,
        InputConfig::Floating => 0b01,
        InputConfig::PullUp => 0b10,
        InputConfig::PullDown => 0b10,
    }
}

fn input_odr(config: &InputConfig) -> Option<u8> {
    match *config {
        InputConfig::PullDown => Some(0b0),
        InputConfig::PullUp => Some(0b1),
        _ => None,
    }
}

/// Possible configurations for a GPIO pin in output mode
pub enum OutputConfig {
    /// General purpose output Push-pull
    GeneralPurposePushPull,
    /// General purpose output Open-drain
    GeneralPurposeOpenDrain,
    /// Alternate function output Push-pull
    AlternateFunctionPushPull,
    /// Alternate function output Open-drain
    AlternateFunctionOpenDrain,
}

fn output_cnf(config: &OutputConfig) -> u8 {
    match *config {
        OutputConfig::GeneralPurposePushPull => 0b00,
        OutputConfig::GeneralPurposeOpenDrain => 0b01,
        OutputConfig::AlternateFunctionPushPull => 0b10,
        OutputConfig::AlternateFunctionOpenDrain => 0b10,
    }
}

/// Input
pub trait Input {
    /// Returns true if the pin is set "high" (3V3)
    fn is_high(&self) -> bool;

    /// Returns true if the pin is set "low" (0V)
    fn is_low(&self) -> bool;
}

/// Output
pub trait Output {
    /// Sets the pin "high" (3V3)
    fn set_high(&self);

    /// Sets the pin "low" (0V)
    fn set_low(&self);
}

/// GPIO pin
pub trait GPIOPin: Input + Output {
    /// Sets the pin mode (input, output) and configuration
    fn set_mode(&self, mode: GPIOMode);

    /// Sets the input pin mode configuration
    fn configure_input(&self, config: InputConfig);
 
    /// Sets the output pin mode configuration
    fn configure_output(&self, config: OutputConfig);
}

macro_rules! pin {
    ($GPIOY:ident, $PYX:ident, $bsX:ident, $brX:ident, $idrX:ident, $modeX:ident, $cnfX:ident, $odrX:ident, $crZ:ident) => {
        /// Digital output / input
        pub struct $PYX;

        impl GPIOPin for $PYX {
            fn set_mode(&self, mode: GPIOMode) {
                unsafe {
                    (*$GPIOY.get()).$crZ.modify(|_, w| w.$modeX().bits(mode_value(&mode)));
                }
                match mode {
                    GPIOMode::Input(config) => self.configure_input(config),
                    GPIOMode::Output(config) => self.configure_output(config),
                    GPIOMode::Output2(config) => self.configure_output(config),
                    GPIOMode::Output50(config) => self.configure_output(config),
                }
            }

            fn configure_input(&self, config: InputConfig) {
                unsafe {
                    (*$GPIOY.get()).$crZ.modify(|_, w| w.$cnfX().bits(input_cnf(&config)));
                    match input_odr(&config) {
                        Some(odr) => {
                            if odr == 0b1 { // input pull-up
                                (*$GPIOY.get()).bsrr.write(|w| w.$bsX().bit(true));
                            } else { // input pull-down
                                (*$GPIOY.get()).bsrr.write(|w| w.$brX().bit(true));
                            }
                        },
                        None => ()
                    }
                }
            }
            fn configure_output(&self, config: OutputConfig) {
                unsafe {
                    (*$GPIOY.get()).$crZ.modify(|_, w| w.$cnfX().bits(output_cnf(&config)));
                }
           }
       }

        impl Input for $PYX {
            fn is_high(&self) -> bool {
                unsafe {
                    (*$GPIOY.get()).idr.read().$idrX().bit_is_set()
                }
            }

            fn is_low(&self) -> bool {
                unsafe {
                    (*$GPIOY.get()).idr.read().$idrX().bit_is_clear()
                }
            }
        }

        impl Output for $PYX {
            fn set_high(&self) {
                // NOTE(safe) atomic write
                unsafe {
                    (*$GPIOY.get()).bsrr.write(|w| w.$bsX().bit(true));
                }
            }

            fn set_low(&self) {
                // NOTE(safe) atomic write
                unsafe {
                    (*$GPIOY.get()).bsrr.write(|w| w.$brX().bit(true));
                }
            }
        }
    }
}

// Port B GPIO
pin!(GPIOB, PB0, bs0, br0, idr0, mode0, cnf0, odr0, crl);
pin!(GPIOB, PB1, bs1, br1, idr1, mode1, cnf1, odr1, crl);
// PB2 is attached to the Boot1 jumper
pin!(GPIOB, PB3, bs3, br3, idr3, mode3, cnf3, odr3, crl);
pin!(GPIOB, PB4, bs4, br4, idr4, mode4, cnf4, odr4, crl);
pin!(GPIOB, PB5, bs5, br5, idr5, mode5, cnf5, odr5, crl);
pin!(GPIOB, PB6, bs6, br6, idr6, mode6, cnf6, odr6, crl);
pin!(GPIOB, PB7, bs7, br7, idr7, mode7, cnf7, odr7, crl);
pin!(GPIOB, PB8, bs8, br8, idr8, mode8, cnf8, odr8, crh);
pin!(GPIOB, PB9, bs9, br9, idr9, mode9, cnf9, odr9, crh);
pin!(GPIOB, PB10, bs10, br10, idr10, mode10, cnf10, odr10, crh);
pin!(GPIOB, PB11, bs11, br11, idr11, mode11, cnf11, odr11, crh);
pin!(GPIOB, PB12, bs12, br12, idr12, mode12, cnf12, odr12, crh);
pin!(GPIOB, PB13, bs13, br13, idr13, mode13, cnf13, odr13, crh);
pin!(GPIOB, PB14, bs14, br14, idr14, mode14, cnf14, odr14, crh);
pin!(GPIOB, PB15, bs15, br15, idr15, mode15, cnf15, odr15, crh);

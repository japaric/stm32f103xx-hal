//! General Purpose I/O
//!
//! - PB12
//! - PB13
//! - PB14
//! - PB15

use stm32f103xx::{GPIOB, RCC};

/// Initializes the digital outputs
pub fn init(gpiob: &GPIOB, rcc: &RCC) {
    rcc.apb2enr.modify(|_, w| w.iopben().enabled());

    gpiob.crh.modify(|_, w| {
        w.mode12()
            .bits(0b10)
            .cnf12()
            .bits(0b00)
            .mode13()
            .bits(0b10)
            .cnf13()
            .bits(0b00)
            .mode14()
            .bits(0b10)
            .cnf14()
            .bits(0b00)
            .mode15()
            .bits(0b10)
            .cnf15()
            .bits(0b00)
    });
}

macro_rules! pin {
    ($PBX:ident, $bsX:ident, $brX:ident) => {
        /// Digital output
        pub struct $PBX;

        impl $PBX {
            /// Sets the pin "high" (3V3)
            pub fn high(&self) {
                // NOTE(safe) atomic write
                unsafe {
                    (*GPIOB.get()).bsrr.write(|w| w.$bsX().bit(true));
                }
            }

            /// Sets the pin "low" (0V)
            pub fn low(&self) {
                // NOTE(safe) atomic write
                unsafe {
                    (*GPIOB.get()).bsrr.write(|w| w.$brX().bit(true));
                }
            }
        }
    }
}

pin!(PB12, bs12, br12);
pin!(PB13, bs13, br13);
pin!(PB14, bs14, br14);
pin!(PB15, bs15, br15);

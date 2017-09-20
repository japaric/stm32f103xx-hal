//! User LEDs
//!
//! - PC13

use stm32f103xx::{GPIOC, RCC};

/// LED connected to pin PC13
pub struct PC13;

/// Initializes the user LED
pub fn init(gpioc: &GPIOC, rcc: &RCC) {
    // power on GPIOC
    rcc.apb2enr.modify(|_, w| w.iopcen().enabled());

    // configure PC13 as output
    gpioc.bsrr.write(|w| w.bs13().set());
    gpioc.crh.modify(|_, w| w.mode13().output().cnf13().push());
}

impl PC13 {
    /// Turns the LED on
    pub fn on(&self) {
        unsafe {
            (*GPIOC.get()).bsrr.write(|w| w.br13().reset());
        }
    }

    /// Turns the LED off
    pub fn off(&self) {
        unsafe {
            (*GPIOC.get()).bsrr.write(|w| w.bs13().set());
        }
    }
}

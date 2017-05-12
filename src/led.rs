//! PC13 - User LED (Green)

use stm32f103xx::{GPIOC, Gpioc, Rcc};

/// Green LED (PC13)
pub struct Green;

/// Initializes the user LED
pub fn init(gpioc: &Gpioc, rcc: &Rcc) {
    // power on GPIOC
    rcc.apb2enr.modify(|_, w| unsafe { w.iopcen().bits(1) });

    // configure PC13 as output
    gpioc.bsrr.write(|w| unsafe { w.bs13().bits(1) });
    gpioc
        .crh
        .modify(|_, w| unsafe { w.mode13().bits(0b10).cnf13().bits(0b00) });
}

impl Green {
    /// Turns the LED on
    pub fn on(&self) {
        unsafe {
            (*GPIOC.get()).bsrr.write(|w| w.br13().bits(1));
        }
    }

    /// Turns the LED off
    pub fn off(&self) {
        unsafe {
            (*GPIOC.get()).bsrr.write(|w| w.bs13().bits(1));
        }
    }
}

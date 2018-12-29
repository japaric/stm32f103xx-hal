//! Makes an analog reading on channel 0 and prints it to itm

#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m;
extern crate cortex_m_semihosting;
extern crate panic_semihosting;
extern crate stm32f103xx_hal;

use core::fmt::Write;

use cortex_m_semihosting::hio;

use stm32f103xx_hal::prelude::*;

use rt::ExceptionFrame;
use stm32f103xx_hal::adc::{self, AnalogPin};

#[entry]
fn main() -> ! {
    // Aquire the peripherals
    let p = stm32f103xx_hal::stm32f103xx::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();

    // Set up the ADC
    let mut adc = adc::Adc::adc2(p.ADC2, &mut rcc.apb2);

    // Configure gpioa 0 as an analog input
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
    let pb1 = gpiob.pb1.into_analog_input(&mut gpiob.crl);

    loop {
        // Aquire stdout and print the result of an analog reading
        // NOTE: This will probably freeze when running without a debugger connected.
        hio::hstdout().map(|mut hio| {
            writeln!(hio, "reading: {}", pb1.analog_read(&mut adc)).unwrap()
        }).unwrap();
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

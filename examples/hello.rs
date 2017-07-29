//! Prints "Hello, World" in the OpenOCD console
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting as semihosting;

use core::fmt::Write;

use rtfm::app;
use semihosting::hio;

app! {
    device: blue_pill::stm32f103xx,
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    writeln!(hio::hstdout().unwrap(), "Hello, world!").unwrap();

    loop {
        rtfm::wfi();
    }
}

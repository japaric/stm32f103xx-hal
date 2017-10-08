//! Sets PB12 high
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::gpio::{self, PB12, GPIOPin, Output, GPIOMode};
use rtfm::app;

app! {
    device: blue_pill::stm32f103xx,
}

fn init(p: init::Peripherals) {
    gpio::init(p.GPIOB, p.RCC);
    PB12.set_mode(GPIOMode::OUTPUT);
}

fn idle() -> ! {
    PB12.set_high();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

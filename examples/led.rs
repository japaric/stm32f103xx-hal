//! Turns the user LED on

#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::led::{self, Green};
use rtfm::app;

app! {
    device: blue_pill::stm32f103xx,
}

fn init(p: init::Peripherals) {
    led::init(p.GPIOC, p.RCC);
}

fn idle() -> ! {
    Green.on();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

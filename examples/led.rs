//! Turns the user LED on

#![deny(warnings)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::led::{self, Green};

rtfm! {
    device: blue_pill::stm32f103xx,

    init: {
        path: init,
    },

    idle: {
        path: idle,
    },
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

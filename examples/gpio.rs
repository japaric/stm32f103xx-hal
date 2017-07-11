//! Set PB12 high

#![deny(warnings)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::gpio::{self, PB12};

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
    gpio::init(p.GPIOB, p.RCC);
}

fn idle() -> ! {
    PB12.high();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

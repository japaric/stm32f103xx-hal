//! Turns the user LED on

#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::led::{Green, self};
use blue_pill::stm32f103xx;
use rtfm::{P0, T0, TMax};

// RESOURCES
peripherals!(stm32f103xx, {
    GPIOC: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let gpioc = &GPIOC.access(prio, thr);
    let rcc = &RCC.access(prio, thr);

    led::init(gpioc, rcc);
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    Green.on();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

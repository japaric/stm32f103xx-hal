//! Prints "Hello" and then "World" on the OpenOCD console

#![feature(used)]
#![no_std]

// version = "0.2.9"
#[macro_use]
extern crate cortex_m;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate blue_pill;

use blue_pill::stm32f103xx;
use rtfm::{P0, T0, TMax};

// INITIALIZATION PHASE
fn init(_prio: P0, _thr: &TMax) {
    hprintln!("Hello");
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    hprintln!("World");

    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

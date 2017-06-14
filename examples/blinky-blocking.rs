//! Blocking version of blinky

#![allow(unreachable_code)] // for the `block!` macro
#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

extern crate embedded_hal as hal;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

#[macro_use]
extern crate nb;

use blue_pill::led::{self, Green};
use blue_pill::time::Hertz;
use blue_pill::{Timer, stm32f103xx};
use hal::prelude::*;
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1);

// RESOURCES
peripherals!(stm32f103xx, {
    GPIOC: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM3: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let gpioc = &GPIOC.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim3 = TIM3.access(prio, thr);

    let timer = Timer(&*tim3);

    led::init(gpioc, rcc);
    timer.init(FREQUENCY.invert(), rcc);
}

// IDLE LOOP
fn idle(ref prio: P0, ref thr: T0) -> ! {
    let tim3 = TIM3.access(prio, thr);

    let timer = Timer(&*tim3);

    timer.resume();
    let mut state = false;
    loop {
        block!(timer.wait()).unwrap(); // NOTE(unwrap) E = !

        state = !state;

        if state {
            Green.on();
        } else {
            Green.off();
        }
    }
}

// TASKS
tasks!(stm32f103xx, {});

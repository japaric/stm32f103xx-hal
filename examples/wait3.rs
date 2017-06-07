//! Periodic timeouts with TIM3

#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate blue_pill;

use blue_pill::led::{self, Green};
use blue_pill::{Timer, stm32f103xx};
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const FREQUENCY: u32 = 1;

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
    timer.init(FREQUENCY, rcc);
    timer.resume();
}

// IDLE LOOP
fn idle(ref prio: P0, ref thr: T0) -> ! {
    let tim3 = TIM3.access(prio, thr);

    let timer = Timer(&*tim3);

    let mut state = false;
    loop {
        while timer.clear_update_flag().is_err() {}

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

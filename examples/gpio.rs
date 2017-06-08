//! Set PB12 high

#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::gpio::{PB12, self};
use blue_pill::stm32f103xx;
use rtfm::{P0, T0, TMax};

// RESOURCES
peripherals!(stm32f103xx, {
    GPIOB: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let gpiob = &GPIOB.access(prio, thr);
    let rcc = &RCC.access(prio, thr);

    gpio::init(gpiob, rcc);
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    PB12.high();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

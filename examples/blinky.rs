//! Turns the user LED on

#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

extern crate cortex_m_hal as hal;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Timer;
use blue_pill::led::{self, Green};
use blue_pill::stm32f103xx;
use blue_pill::time::Hertz;
use hal::prelude::*;
use rtfm::{Local, P0, P1, T0, T1, TMax};
use stm32f103xx::interrupt::TIM1_UP_TIM10;

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
    TIM1: Peripheral {
        ceiling: C1,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let gpioc = &GPIOC.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim1 = TIM1.access(prio, thr);

    let timer = Timer(&*tim1);

    led::init(gpioc, rcc);
    timer.init(FREQUENCY.invert(), rcc);
    timer.resume();
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {
    blink: Task {
        interrupt: TIM1_UP_TIM10,
        priority: P1,
        enabled: true,
    },
});

fn blink(mut task: TIM1_UP_TIM10, ref prio: P1, ref thr: T1) {
    static STATE: Local<bool, TIM1_UP_TIM10> = Local::new(false);

    let tim1 = TIM1.access(prio, thr);

    let timer = Timer(&*tim1);

    // NOTE(wait) timeout should have already occurred
    timer.wait().unwrap();

    let state = STATE.borrow_mut(&mut task);

    *state = !*state;

    if *state {
        Green.on();
    } else {
        Green.off();
    }
}

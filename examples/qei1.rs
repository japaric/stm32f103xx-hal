//! Quadrature Encoder Interface using TIM1
//!
//! Periodically reports the readings of the QEI

#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

extern crate embedded_hal as hal;

#[macro_use]
extern crate cortex_m;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::time::Hertz;
use blue_pill::{Qei, Timer, stm32f103xx};
use hal::prelude::*;
use rtfm::{Local, P0, P1, T0, T1, TMax};
use stm32f103xx::interrupt::TIM4;

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1);

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOA: Peripheral {
        ceiling: C0,
    },
    ITM: Peripheral {
        ceiling: C1,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM1: Peripheral {
        ceiling: C1,
    },
    TIM4: Peripheral {
        ceiling: C1,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim1 = TIM1.access(prio, thr);
    let tim4 = TIM4.access(prio, thr);

    let qei = Qei(&*tim1);
    let timer = Timer(&*tim4);

    qei.init(afio, gpioa, rcc);

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
    periodic: Task {
        interrupt: TIM4,
        priority: P1,
        enabled: true,
    },
});

fn periodic(ref mut task: TIM4, ref prio: P1, ref thr: T1) {
    static PREVIOUS: Local<Option<u16>, TIM4> = Local::new(None);

    let itm = &ITM.access(prio, thr);
    let previous = PREVIOUS.borrow_mut(task);
    let tim1 = TIM1.access(prio, thr);
    let tim4 = TIM4.access(prio, thr);

    let qei = Qei(&*tim1);
    let timer = Timer(&*tim4);

    // NOTE(unwrap) timeout should have already occurred
    timer.wait().unwrap_or_else(|_| unreachable!());

    let curr = qei.count();
    let dir = qei.direction();

    if let Some(prev) = previous.take() {
        let speed = (curr as i16).wrapping_sub(prev as i16);

        iprintln!(&itm.stim[0], "{} - {} - {:?}", curr, speed, dir);
    }

    *previous = Some(curr);
}

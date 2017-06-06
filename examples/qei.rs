//! Quadrature Encoder Interface
//!
//! Periodically reports the readings of the QEI

#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate blue_pill;

use blue_pill::qei::Qei;
use blue_pill::stm32f103xx;
use blue_pill::timer::Timer;
use rtfm::{Local, P0, P1, T0, T1, TMax};
use stm32f103xx::interrupt::TIM1_UP_TIM10;

// CONFIGURATION
const FREQUENCY: u32 = 1; // Hz

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOB: Peripheral {
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
    let gpiob = &GPIOB.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim1 = &TIM1.access(prio, thr);
    let tim4 = TIM4.access(prio, thr);

    let qei = Qei(&*tim4);
    let timer = Timer(tim1);

    qei.init(afio, gpiob, rcc);

    timer.init(FREQUENCY, rcc);
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
        interrupt: TIM1_UP_TIM10,
        priority: P1,
        enabled: true,
    },
});

fn periodic(ref mut task: TIM1_UP_TIM10, ref prio: P1, ref thr: T1) {
    static PREVIOUS: Local<Option<u16>, TIM1_UP_TIM10> = Local::new(None);

    let itm = &ITM.access(prio, thr);
    let previous = PREVIOUS.borrow_mut(task);
    let tim1 = &TIM1.access(prio, thr);
    let tim4 = TIM4.access(prio, thr);

    let qei = Qei(&*tim4);
    let timer = Timer(tim1);

    if timer.clear_update_flag().is_ok() {
        let curr = qei.count();
        let dir = qei.direction();

        if let Some(prev) = previous.take() {
            let speed = curr as i16 - prev as i16;

            iprintln!(&itm.stim[0], "{} - {} - {:?}", curr, speed, dir);
        }

        *previous = Some(curr);
    } else {
        // NOTE can only be reached via `rtfm::request(periodic)`
        unreachable!()
    }
}

//! Quadrature Encoder Interface using TIM1
//!
//! Periodically reports the readings of the QEI

#![deny(warnings)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;
#[macro_use(iprint, iprintln)]
extern crate cortex_m;
#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::prelude::*;
use blue_pill::time::Hertz;
use blue_pill::{Qei, Timer};
use rtfm::Threshold;

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1);

rtfm! {
    device: blue_pill::stm32f103xx,

    init: {
        path: init,
    },

    idle: {
        path: idle,
    },

    tasks: {
        TIM4: {
            enabled: true,
            priority: 1,
            resources: [ITM, TIM1, TIM4],
        },
    },
}

fn init(p: init::Peripherals) {
    let qei = Qei(p.TIM1);
    let timer = Timer(p.TIM4);

    qei.init(p.AFIO, p.GPIOA, p.RCC);

    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

task!(TIM4, periodic, Locals {
    previous: Option<u16> = None;
});

fn periodic(_t: Threshold, l: &mut Locals, r: TIM4::Resources) {
    let qei = Qei(r.TIM1);
    let timer = Timer(r.TIM4);

    // NOTE(unwrap) timeout should have already occurred
    timer.wait().unwrap_or_else(|_| unreachable!());

    let curr = qei.count();
    let dir = qei.direction();

    if let Some(prev) = l.previous.take() {
        let speed = (curr as i16).wrapping_sub(prev as i16);

        iprintln!(&r.ITM.stim[0], "{} - {} - {:?}", curr, speed, dir);
    }

    l.previous = Some(curr);
}

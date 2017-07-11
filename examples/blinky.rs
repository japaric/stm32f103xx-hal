//! Turns the user LED on

#![deny(warnings)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Timer;
use blue_pill::led::{self, Green};
use blue_pill::prelude::*;
use blue_pill::time::Hertz;
use rtfm::Threshold;

rtfm! {
    device: blue_pill::stm32f103xx,

    resources: {},

    init: {
        path: init,
    },

    idle: {
        path: idle,
    },

    tasks: {
        TIM2: {
            priority: 1,
            enabled: true,
            resources: [TIM2],
        },
    },
}

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1);

// INITIALIZATION PHASE
fn init(p: init::Peripherals) {
    let timer = Timer(p.TIM2);

    led::init(p.GPIOC, p.RCC);
    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume();
}

// IDLE LOOP
fn idle() -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
task!(TIM2, blink, Local {
    state: bool = false;
});

fn blink(_t: Threshold, l: &mut Local, r: TIM2::Resources) {
    let timer = Timer(r.TIM2);

    timer.wait().unwrap();

    l.state = !l.state;

    if l.state {
        Green.on();
    } else {
        Green.off();
    }
}

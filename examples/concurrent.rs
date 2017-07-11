//! Serial loopback

#![deny(warnings)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;

#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::led::{self, Green};
use blue_pill::prelude::*;
use blue_pill::serial::Event;
use blue_pill::time::Hertz;
use blue_pill::{Serial, Timer, stm32f103xx};
use rtfm::Threshold;

rtfm! {
    device: stm32f103xx,

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
        USART1: {
            priority: 1,
            enabled: true,
            resources: [USART1],
        },
    },
}

// CONFIGURATION
pub const BAUD_RATE: Hertz = Hertz(115_200);
pub const FREQUENCY: Hertz = Hertz(1);

// INITIALIZATION PHASE
fn init(p: init::Peripherals) {
    let serial = Serial(p.USART1);
    let timer = Timer(p.TIM2);

    led::init(p.GPIOC, p.RCC);

    serial.init(BAUD_RATE.invert(), p.AFIO, None, p.GPIOA, p.RCC);
    serial.listen(Event::Rxne);

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
task!(TIM2, blinky, Local {
    state: bool = false;
});

fn blinky(_t: Threshold, l: &mut Local, r: TIM2::Resources) {
    let timer = Timer(r.TIM2);

    timer.wait().unwrap();

    l.state = !l.state;

    if l.state {
        Green.on();
    } else {
        Green.off();
    }
}

task!(USART1, loopback);

fn loopback(_t: Threshold, r: USART1::Resources) {
    let serial = Serial(r.USART1);

    let byte = serial.read().unwrap();
    serial.write(byte).unwrap();
}

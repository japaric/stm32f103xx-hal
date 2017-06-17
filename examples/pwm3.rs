//! Output a PWM with a duty cycle of ~6% on all the channels of TIM3

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

use blue_pill::{Channel, Pwm, stm32f103xx};
use blue_pill::time::Hertz;
use hal::prelude::*;
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1_000);

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOA: Peripheral {
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
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim3 = TIM3.access(prio, thr);

    let pwm = Pwm(&*tim3);

    pwm.init(FREQUENCY.invert(), afio, None, gpioa, rcc);
    let duty = pwm.get_max_duty() / 16;

    const CHANNELS: [Channel; 2] = [Channel::_1, Channel::_2];

    for c in &CHANNELS {
        pwm.set_duty(*c, duty);
    }

    for c in &CHANNELS {
        pwm.enable(*c);
        rtfm::bkpt();
    }
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

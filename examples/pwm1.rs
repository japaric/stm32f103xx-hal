//! Output a PWM with a duty cycle of ~6% on all the channels of TIM1
// FIXME doesn't seem to work :-(

#![deny(warnings)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::prelude::*;
use blue_pill::time::Hertz;
use blue_pill::{Channel, Pwm};

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1_000);

rtfm! {
    device: blue_pill::stm32f103xx,

    init: {
        path: init,
    },

    idle: {
        path: idle,
    },
}

fn init(p: init::Peripherals) {
    let pwm = Pwm(p.TIM1);

    pwm.init(FREQUENCY.invert(), p.AFIO, p.GPIOA, p.RCC);
    let duty = pwm.get_max_duty() / 16;

    const CHANNELS: [Channel; 4] =
        [Channel::_1, Channel::_2, Channel::_3, Channel::_4];

    for c in &CHANNELS {
        pwm.set_duty(*c, duty);
    }

    for c in &CHANNELS {
        pwm.enable(*c);
        rtfm::bkpt();
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

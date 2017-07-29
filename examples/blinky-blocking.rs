//! Blocking version of blinky

#![allow(unreachable_code)] // for the `block!` macro
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;

extern crate cortex_m_rtfm as rtfm;

#[macro_use(block)]
extern crate nb;

use blue_pill::Timer;
use blue_pill::led::{self, Green};
use blue_pill::prelude::*;
use blue_pill::time::Hertz;
use rtfm::{app, Threshold};

const FREQUENCY: Hertz = Hertz(1);

app! {
    device: blue_pill::stm32f103xx,

    idle: {
        resources: [TIM3],
    }
}

fn init(p: init::Peripherals) {
    led::init(p.GPIOC, p.RCC);

    let timer = Timer(&*p.TIM3);

    timer.init(FREQUENCY.invert(), p.RCC);
}

fn idle(_t: &mut Threshold, r: idle::Resources) -> ! {
    let timer = Timer(&*r.TIM3);

    timer.resume();
    let mut state = false;
    loop {
        block!(timer.wait()).unwrap(); // NOTE(unwrap) E = !

        state = !state;

        if state {
            Green.on();
        } else {
            Green.off();
        }
    }
}

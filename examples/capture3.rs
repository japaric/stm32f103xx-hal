//! Input capture using TIM3

#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[macro_use(iprint, iprintln)]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate nb;

use blue_pill::prelude::*;
use blue_pill::time::Milliseconds;
use blue_pill::{Capture, Channel};
use rtfm::app;

// CONFIGURATION
const RESOLUTION: Milliseconds = Milliseconds(1);

app! {
    device: blue_pill::stm32f103xx,

    idle: {
        resources: [ITM, TIM3],
    },
}

fn init(p: init::Peripherals) {
    let capture = Capture(p.TIM3);

    capture.init(RESOLUTION, p.AFIO, p.GPIOA, p.RCC);
}

fn idle(r: idle::Resources) -> ! {
    const CHANNELS: [Channel; 2] = [Channel::_1, Channel::_2];

    let capture = Capture(r.TIM3);

    for c in &CHANNELS {
        capture.enable(*c);
    }

    loop {
        for c in &CHANNELS {
            match capture.capture(*c) {
                Ok(snapshot) => {
                    iprintln!(&r.ITM.stim[0], "{:?}: {:?} ms", c, snapshot);
                }
                Err(nb::Error::WouldBlock) => {}
                Err(nb::Error::Other(e)) => panic!("{:?}", e),
            }
        }
    }
}

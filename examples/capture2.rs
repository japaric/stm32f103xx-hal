//! Input capture using TIM2

#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

#[macro_use]
extern crate cortex_m;

extern crate cortex_m_hal as hal;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate nb;

use blue_pill::stm32f103xx;
use blue_pill::time::Milliseconds;
use blue_pill::{Capture, Channel};
use hal::prelude::*;
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const RESOLUTION: Milliseconds = Milliseconds(1);

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOA: Peripheral {
        ceiling: C0,
    },
    ITM: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM2: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim2 = TIM2.access(prio, thr);

    let capture = Capture(&*tim2);

    capture.init(RESOLUTION, afio, gpioa, rcc);
}

// IDLE LOOP
fn idle(ref prio: P0, ref thr: T0) -> ! {
    const CHANNELS: [Channel; 4] =
        [Channel::_1, Channel::_2, Channel::_3, Channel::_4];

    let itm = ITM.access(prio, thr);
    let tim2 = TIM2.access(prio, thr);

    let capture = Capture(&*tim2);

    for c in &CHANNELS {
        capture.enable(*c);
    }

    loop {
        for c in &CHANNELS {
            match capture.capture(*c) {
                Ok(snapshot) => {
                    iprintln!(&itm.stim[0], "{:?}: {:?} ms", c, snapshot);
                }
                Err(nb::Error::WouldBlock) => {}
                Err(nb::Error::Other(e)) => panic!("{:?}", e),
            }
        }
    }
}

// TASKS
tasks!(stm32f103xx, {});

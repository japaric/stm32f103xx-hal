//! Drive a ring of 24 WS2812 LEDs
//!
//! To test this demo connect the data-in pin of the LED ring to pin PA0

#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate embedded_hal as hal;

#[macro_use]
extern crate nb;

use blue_pill::dma::{Buffer, Dma1Channel2};
use blue_pill::time::Hertz;
use blue_pill::{Channel, Pwm, stm32f103xx};
use hal::prelude::*;
use rtfm::{C1, P0, Resource, T0, TMax};

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(200_000);
const _0: u8 = 3;
const _1: u8 = 5;

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    DMA1: Peripheral {
        ceiling: C0,
    },
    GPIOA: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM2: Peripheral {
        ceiling: C0,
    },
});

static BUFFER: Resource<Buffer<[u8; (24 * 24) + 1], Dma1Channel2>, C1> =
    Resource::new(Buffer::new([_0; (24 * 24) + 1]));

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let buffer = BUFFER.access(prio, thr);
    let dma1 = &DMA1.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim2 = TIM2.access(prio, thr);

    let pwm = Pwm(&*tim2);

    pwm.init(FREQUENCY.invert(), afio, Some(dma1), gpioa, rcc);
    pwm.enable(Channel::_1);

    // end of frame
    *buffer.borrow_mut().last_mut().unwrap() = 0;

    // set each RGB value to 0x0A0A0A
    for byte in buffer.borrow_mut()[..(24 * 24)].chunks_mut(8) {
        byte.copy_from_slice(&[_0, _0, _0, _0, _1, _1, _1, _1]);
    }

    pwm.set_duties(dma1, Channel::_1, buffer).unwrap();

    block!(buffer.release(dma1)).unwrap();

    rtfm::bkpt();
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

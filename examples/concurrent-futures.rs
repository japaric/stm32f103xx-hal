//! Two concurrent tasks using futures

#![allow(unreachable_code)] // for the `try_nb!` macro
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

extern crate futures;

#[macro_use]
extern crate nb;

use blue_pill::led::{self, Green};
use blue_pill::{Serial, Timer, stm32f103xx};
use futures::{Async, Future};
use futures::future::{self, Loop};
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const BAUD_RATE: u32 = 115_200;
const FREQUENCY: u32 = 1;

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOA: Peripheral {
        ceiling: C0,
    },
    GPIOC: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM3: Peripheral {
        ceiling: C0,
    },
    USART1: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let gpioc = &GPIOC.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim3 = TIM3.access(prio, thr);
    let usart1 = USART1.access(prio, thr);

    let timer = Timer(&*tim3);
    let serial = Serial(&*usart1);

    led::init(gpioc, rcc);
    serial.init(BAUD_RATE, afio, gpioa, rcc);
    timer.init(FREQUENCY, rcc);
}

// IDLE LOOP
#[inline(never)]
fn idle(ref prio: P0, ref thr: T0) -> ! {
    let tim3 = TIM3.access(prio, thr);
    let usart1 = USART1.access(prio, thr);

    let timer = Timer(&*tim3);
    let serial = Serial(&*usart1);

    // Tasks
    let mut blinky = future::loop_fn::<_, (), _, _>(true, |state| {
        future::poll_fn(move || Ok(Async::Ready(try_nb!(timer.wait()))))
            .map(move |_| {
                if state {
                    Green.on();
                } else {
                    Green.off();
                }

                Loop::Continue(!state)
            })
    });

    let mut loopback = future::loop_fn::<_, (), _, _>((), |_| {
        future::poll_fn(move || Ok(Async::Ready(try_nb!(serial.read()))))
            .and_then(|byte| {
                future::poll_fn(
                    move || Ok(Async::Ready(try_nb!(serial.write(byte)))),
                )
            })
            .map(|_| Loop::Continue(()))
    });

    // Event loop
    timer.resume();
    loop {
        loopback.poll().unwrap(); // NOTE(unwrap) E = !
        blinky.poll().unwrap();
    }
}

// TASKS
tasks!(stm32f103xx, {});

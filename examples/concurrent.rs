//! Serial loopback

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

extern crate nb;

use blue_pill::led::{self, Green};
use blue_pill::serial::Event;
use blue_pill::time::Hertz;
use blue_pill::{Serial, Timer, stm32f103xx};
use hal::prelude::*;
use rtfm::{Local, P0, P1, T0, T1, TMax};
use stm32f103xx::interrupt::{TIM1_UP_TIM10, USART1};

// CONFIGURATION
pub const BAUD_RATE: Hertz = Hertz(115_200);
pub const FREQUENCY: Hertz = Hertz(1);

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
    USART1: Peripheral {
        ceiling: C1,
    },
    TIM1: Peripheral {
        ceiling: C1,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let gpioc = &GPIOC.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let usart1 = USART1.access(prio, thr);
    let tim1 = TIM1.access(prio, thr);

    let serial = Serial(&*usart1);
    let timer = Timer(&*tim1);

    led::init(gpioc, rcc);

    serial.init(BAUD_RATE.invert(), afio, None, gpioa, rcc);
    serial.listen(Event::Rxne);

    timer.init(FREQUENCY.invert(), rcc);
    timer.resume();
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {
    blinky: Task {
        interrupt: TIM1_UP_TIM10,
        priority: P1,
        enabled: true,
    },
    loopback: Task {
        interrupt: USART1,
        priority: P1,
        enabled: true,
    },
});

fn blinky(ref mut task: TIM1_UP_TIM10, ref prio: P1, ref thr: T1) {
    static STATE: Local<bool, TIM1_UP_TIM10> = Local::new(false);

    let state = STATE.borrow_mut(task);
    let tim1 = TIM1.access(prio, thr);

    let timer = Timer(&*tim1);

    // NOTE(unwrap) timeout should have already occurred
    timer.wait().unwrap_or_else(|_| unreachable!());

    *state = !*state;

    if *state {
        Green.on();
    } else {
        Green.off();
    }
}

fn loopback(_task: USART1, ref prio: P1, ref thr: T1) {
    let usart1 = USART1.access(prio, thr);

    let serial = Serial(&*usart1);

    match serial.read().and_then(|byte| serial.write(byte)) {
        Err(nb::Error::Other(e)) => panic!("{:?}", e),
        Err(nb::Error::WouldBlock) => unreachable!(),
        Ok(()) => {}
    }
}

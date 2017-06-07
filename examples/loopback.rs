//! Serial loopback

#![feature(const_fn)]
#![feature(used)]
#![no_std]

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate blue_pill;

use blue_pill::{Serial, stm32f103xx};
use rtfm::{P0, P1, T0, T1, TMax};
use stm32f103xx::interrupt::USART1;

// CONFIGURATION
pub const BAUD_RATE: u32 = 115_200;

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
    USART1: Peripheral {
        ceiling: C1,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let usart1 = USART1.access(prio, thr);

    let serial = Serial(&*usart1);

    serial.init(BAUD_RATE, afio, gpioa, rcc);
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
    loopback: Task {
        interrupt: USART1,
        priority: P1,
        enabled: true,
    },
});

fn loopback(_task: USART1, ref prio: P1, ref thr: T1) {
    let usart1 = USART1.access(prio, thr);

    let serial = Serial(&*usart1);

    if let Ok(byte) = serial.read() {
        if serial.write(byte).is_err() {
            // NOTE(unreachable!) unlikely to overrun the TX buffer because we
            // are sending _one_ byte per byte received
            #[cfg(debug_assertions)]
            unreachable!()
        }
    } else {
        // NOTE(unreachable!) only reachable through `rtfm::request(loopback)`
        #[cfg(debug_assertions)]
        unreachable!()
    }
}

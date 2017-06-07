//! Test the USART1 instance
//!
//! Connect the TX and RX pins to run this test

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
use rtfm::{P0, T0, TMax};

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

    const BYTE: u8 = b'A';

    assert!(serial.write(BYTE).is_ok());

    for _ in 0..1_000 {
        if let Ok(byte) = serial.read() {
            assert_eq!(byte, BYTE);
            return;
        }
    }

    unreachable!()
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    rtfm::bkpt();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

//! Test the USART3 instance
//!
//! Connect the TX and RX pins to run this test

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

use blue_pill::{Serial, stm32f103xx};
use blue_pill::time::Hertz;
use hal::prelude::*;
use nb::Error;
use rtfm::{P0, T0, TMax};

// CONFIGURATION
pub const BAUD_RATE: Hertz = Hertz(115_200);

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOB: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    USART3: Peripheral {
        ceiling: C1,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpiob = &GPIOB.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let usart3 = USART3.access(prio, thr);

    let serial = Serial(&*usart3);

    serial.init(BAUD_RATE.invert(), afio, None, gpiob, rcc);

    const BYTE: u8 = b'A';

    assert!(serial.write(BYTE).is_ok());

    for _ in 0..1_000 {
        match serial.read() {
            Ok(byte) => {
                assert_eq!(byte, BYTE);
                return;
            }
            Err(Error::Other(e)) => panic!("{:?}", e),
            Err(Error::WouldBlock) => continue,
        }
    }

    panic!("Timeout")
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    // OK
    rtfm::bkpt();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

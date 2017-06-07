//! Interfacing the MPU9250

#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate blue_pill;

use blue_pill::{Spi, stm32f103xx};
use rtfm::{P0, T0, TMax};

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
    SPI1: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let spi1 = SPI1.access(prio, thr);

    let spi = Spi(&*spi1);

    spi.init(afio, gpioa, rcc);
}

// IDLE LOOP
fn idle(ref prio: P0, ref thr: T0) -> ! {
    // Register to read
    const WHO_AM_I: u8 = 117;

    // Junk data
    const JUNK: u8 = 0xaa;

    // Expected answer
    const ANS: u8 = 0x73;

    // Read mode
    pub const R: u8 = 1 << 7;

    let itm = &ITM.access(prio, thr);
    let spi1 = SPI1.access(prio, thr);

    let spi = Spi(&*spi1);

    rtfm::bkpt();

    spi.enable();

    // The SPI is buffered. We can send a few bytes
    while spi.send(WHO_AM_I | R).is_err() {}

    let _junk = loop {
        if let Ok(byte) = spi.receive() {
            break byte;
        }
    };

    while spi.send(JUNK).is_err() {}

    let ans = loop {
        if let Ok(byte) = spi.receive() {
            break byte;
        }
    };

    spi.disable();

    iprintln!(&itm.stim[0], "TESTING ...");

    assert_eq!(ans, ANS);

    iprintln!(&itm.stim[0], "OK");

    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

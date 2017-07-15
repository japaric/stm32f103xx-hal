//! Interfacing the MPU9250 using SPI1

#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[macro_use(iprint, iprintln)]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Spi;
use blue_pill::prelude::*;
use rtfm::app;

app! {
    device: blue_pill::stm32f103xx,

    idle: {
        resources: [ITM, SPI1],
    },
}

fn init(p: init::Peripherals) {
    let spi = Spi(p.SPI1);

    spi.init(p.AFIO, p.GPIOA, p.RCC);
}

fn idle(r: idle::Resources) -> ! {
    // Register to read
    const WHO_AM_I: u8 = 117;

    // Junk data
    const JUNK: u8 = 0xaa;

    // Expected answer
    const ANS: u8 = 0x73;

    // Read mode
    pub const R: u8 = 1 << 7;

    let spi = Spi(r.SPI1);

    rtfm::bkpt();

    spi.enable();

    while spi.send(WHO_AM_I | R).is_err() {}

    let _junk = loop {
        if let Ok(byte) = spi.read() {
            break byte;
        }
    };

    while spi.send(JUNK).is_err() {}

    let ans = loop {
        if let Ok(byte) = spi.read() {
            break byte;
        }
    };

    spi.disable();

    iprintln!(&r.ITM.stim[0], "TESTING ...");

    assert_eq!(ans, ANS);

    iprintln!(&r.ITM.stim[0], "OK");

    loop {
        rtfm::wfi();
    }
}

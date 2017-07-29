//! Test the USART1 instance
//!
//! Connect the TX and RX pins to run this test

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
extern crate nb;

use blue_pill::Serial;
use blue_pill::prelude::*;
use blue_pill::time::Hertz;
use rtfm::app;

const BAUD_RATE: Hertz = Hertz(115_200);

app! {
    device: blue_pill::stm32f103xx,
}

fn init(p: init::Peripherals) {
    let serial = Serial(p.USART1);

    serial.init(BAUD_RATE.invert(), p.AFIO, None, p.GPIOA, p.RCC);

    const BYTE: u8 = b'A';

    assert!(serial.write(BYTE).is_ok());

    for _ in 0..1_000 {
        match serial.read() {
            Ok(byte) => {
                assert_eq!(byte, BYTE);
                return;
            }
            Err(nb::Error::Other(e)) => panic!("{:?}", e),
            Err(nb::Error::WouldBlock) => continue,
        }
    }

    panic!("Timeout")
}

fn idle() -> ! {
    // OK
    rtfm::bkpt();

    // Sleep
    loop {
        rtfm::wfi();
    }
}

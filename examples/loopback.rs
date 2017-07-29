//! Serial loopback via USART1
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Serial;
use blue_pill::prelude::*;
use blue_pill::serial::Event;
use blue_pill::time::Hertz;
use rtfm::{app, Threshold};

const BAUD_RATE: Hertz = Hertz(115_200);

app! {
    device: blue_pill::stm32f103xx,

    tasks: {
        USART1: {
            path: loopback,
            resources: [USART1],
        },
    },
}

fn init(p: init::Peripherals) {
    let serial = Serial(p.USART1);

    serial.init(BAUD_RATE.invert(), p.AFIO, None, p.GPIOA, p.RCC);
    serial.listen(Event::Rxne);
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn loopback(_t: &mut Threshold, r: USART1::Resources) {
    let serial = Serial(&**r.USART1);

    let byte = serial.read().unwrap();
    serial.write(byte).unwrap();
}

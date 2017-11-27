//! Loopback

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
#[macro_use]
extern crate nb;

use core::ptr;

use blue_pill::prelude::*;
use blue_pill::{Serial, stm32f103xx};
use rtfm::app;

const N: usize = 4;

app! {
    device: stm32f103xx,

    resources: {
        static BUFFER: [[u8; N]; 2] = [[0; N]; 2];
    },

    init: {
        resources: [BUFFER],
    },
}

fn init(p: init::Peripherals, r: init::Resources) {
    let mut rcc = p.device.RCC.split();

    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let channels = p.device.DMA1.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);

    // try commenting this out
    // rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa9 = gpioa.pa9.as_alt_push(&mut gpioa.crh);
    let pa10 = gpioa.pa10;

    let serial = Serial::new(
        p.device.USART1,
        (pa9, pa10),
        112_200.bps(),
        clocks,
        &mut rcc.enr,
        &mut afio.mapr,
    );

    let (_tx, rx) = serial.split();

    let mut trans = rx.circ_read(r.BUFFER, channels.5);
    loop {
        block!(trans.read(|buffer| unsafe { ptr::read_volatile(buffer) })).unwrap();

        rtfm::bkpt();
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

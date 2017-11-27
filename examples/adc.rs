//! FIXME doesn't work

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
#[macro_use]
extern crate nb;

use core::ptr;

use blue_pill::prelude::*;
use blue_pill::{adc, stm32f103xx};
use rtfm::app;

const N: usize = 16;

app! {
    device: stm32f103xx,

    resources: {
        static BUFFER: [[u16; N]; 2] = [[0; N]; 2];
    },

    init: {
        resources: [BUFFER],
    },
}

fn init(p: init::Peripherals, r: init::Resources) {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();
    let channels = p.device.DMA1.split(&mut rcc.enr);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa1 = gpioa.pa1.as_alt_push(&mut gpioa.crl);
    let t2c2 = p.device
        .TIM2
        .pwm(pa1, 16.hz(), clocks, &mut rcc.enr, &mut afio.mapr);

    let mut trans = adc::start(
        t2c2,
        r.BUFFER,
        p.device.ADC1,
        &mut rcc.enr,
        &mut gpioa.crl,
        channels.1,
    );

    loop {
        block!(trans.read(|samples| unsafe { ptr::read_volatile(samples) })).unwrap();

        rtfm::bkpt();
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

//! Using the DMA to receive bytes

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use blue_pill::Serial;
use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static BUFFER: [u8; 4] = [0; 4];
    },

    init: {
        resources: [BUFFER],
    },
}

fn init(p: init::Peripherals, r: init::Resources) {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split();
    let mut flash = p.device.FLASH.split();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);
    let channels = p.device.DMA1.split(&mut rcc.enr);

    // try commenting this out
    rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa9 = gpioa.pa9.as_alt_push(&mut gpioa.crh);
    let pa10 = gpioa.pa10;

    let (tx, rx) = Serial::new(
        p.device.USART1,
        (pa9, pa10),
        115_200.bps(),
        clocks,
        &mut rcc.enr,
        &mut afio.mapr,
    ).split();

    // challenge: turn this into a loop
    let (_, buffer, _) = rx.read_exact(channels.5, r.BUFFER).wait().unwrap();
    tx.write_all(channels.4, buffer);
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

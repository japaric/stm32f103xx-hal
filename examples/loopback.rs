//! Loopback

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use blue_pill::serial::{Event, Rx, Serial, Tx};
use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static RX: Rx;
        static TX: Tx;
    },

    tasks: {
        USART1: {
            path: usart1,
            resources: [RX, TX],
        },
    },
}

fn init(p: init::Peripherals) -> init::LateResources {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);

    // try commenting this out
    rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa9 = gpioa.pa9.as_alt_push(&mut gpioa.crh);
    let pa10 = gpioa.pa10;

    let mut serial = Serial::new(
        p.device.USART1,
        (pa9, pa10),
        115_200.bps(),
        clocks,
        &mut rcc.enr,
        &mut afio.mapr,
    );

    serial.listen(Event::Rxne);

    let (tx, rx) = serial.split();

    init::LateResources { RX: rx, TX: tx }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn usart1(_t: &mut Threshold, mut r: USART1::Resources) {
    let byte = r.RX.read().unwrap();
    r.TX.write(byte).unwrap();
}

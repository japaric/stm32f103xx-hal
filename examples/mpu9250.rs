//! MPU9250

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
#[macro_use]
extern crate nb;

use blue_pill::Spi;
use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use rtfm::app;

app! {
    device: stm32f103xx,
}

fn init(p: init::Peripherals) {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa4 = gpioa.pa4.as_alt_push(&mut gpioa.crl);
    let pa5 = gpioa.pa5.as_alt_push(&mut gpioa.crl);
    let pa6 = gpioa.pa6;
    let pa7 = gpioa.pa7.as_alt_push(&mut gpioa.crl);
    let mut spi = Spi::new(
        p.device.SPI1,
        (pa4, pa5, pa6, pa7),
        clocks,
        &mut rcc.enr,
        &mut afio.mapr,
    );

    const WHO_AM_I: u8 = 0x75;
    const R: u8 = 1 << 7;
    const JUNK: u8 = 0xAA;
    const ANS: u8 = 0x73;
    // const W: u8 = 0 << 7;

    spi.enable();
    block!(spi.send(WHO_AM_I | R)).unwrap();
    let _ = block!(spi.read()).unwrap();
    block!(spi.send(JUNK)).unwrap();
    let byte = block!(spi.read()).unwrap();
    spi.disable();

    rtfm::bkpt();
    assert_eq!(byte, ANS);
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

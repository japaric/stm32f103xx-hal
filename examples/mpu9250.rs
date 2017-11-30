//! MPU9250

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
extern crate mpu9250;

use blue_pill::Spi;
use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use mpu9250::Mpu9250;
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

    let sck = gpioa.pa5.as_alt_push(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.as_alt_push(&mut gpioa.crl);
    let spi = Spi::new(
        p.device.SPI1,
        (sck, miso, mosi),
        mpu9250::MODE,
        1.mhz(),
        clocks,
        &mut rcc.enr,
        &mut afio.mapr,
    );

    let ncs = gpioa.pa4.as_output(&mut gpioa.crl);
    let mut mpu9250 = Mpu9250::new(spi, ncs);

    let byte = mpu9250.who_am_i().unwrap();

    rtfm::bkpt();
    assert_eq!(byte, 0x71);

    let _measurements = mpu9250.all().unwrap();

    rtfm::bkpt();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

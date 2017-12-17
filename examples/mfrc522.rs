//! MFRC522

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate mfrc522;

use core::fmt::Write;

use blue_pill::Spi;
use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use cortex_m_semihosting::hio;
use mfrc522::Mfrc522;
use rtfm::app;

app! {
    device: stm32f103xx,
}

fn init(p: init::Peripherals) {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.enr);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let sck = gpioa.pa5.as_alt_push(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.as_alt_push(&mut gpioa.crl);
    let spi = Spi::new(
        p.device.SPI1,
        (sck, miso, mosi),
        mfrc522::MODE,
        1.mhz(),
        clocks,
        &mut rcc.enr,
        &mut afio.mapr,
    );

    let nss = gpioa.pa4.as_output(&mut gpioa.crl);
    let mut mfrc522 = Mfrc522::new(spi, nss).unwrap();

    let mut led = gpioc.pc13.as_output(&mut gpioc.crh);
    led.set_high();

    let mut hstdout = hio::hstdout().unwrap();
    loop {
        if let Ok(atqa) = mfrc522.reqa() {
            if let Ok(uid) = mfrc522.select(&atqa) {
                writeln!(hstdout, "{:?}", uid).unwrap();
            }
        }
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

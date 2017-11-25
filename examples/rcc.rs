//! Clock configuration

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use rtfm::app;

app! {
    device: stm32f103xx,
}

fn init(p: init::Peripherals) {
    let mut rcc = p.device.RCC.split();
    let mut flash = p.device.FLASH.split();

    rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());
    let _clocks = rcc.cfgr.freeze(&mut flash.acr);

    // inspect `_clocks`
    rtfm::bkpt();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

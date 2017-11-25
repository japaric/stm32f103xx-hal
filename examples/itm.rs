//! Turn the LED connected to pin PC13 on

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;

use rtfm::app;

app! {
    device: stm32f103xx,
}

fn init(p: init::Peripherals) {
    let rcc = p.device.RCC.split();
    let mut flash = p.device.FLASH.split();

    // IMPORTANT the HCLK frequency selected here must match the TRACECLKIN frequency selected in
    // .gdbinit
    let _clocks = rcc.cfgr.freeze(&mut flash.acr);

    iprintln!(&p.core.ITM.stim[0], "Hello, world!");
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

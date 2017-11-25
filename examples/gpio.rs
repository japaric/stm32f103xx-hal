//! Turn the LED connected to pin PC13 on

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

    let mut gpioc = p.device.GPIOC.split(&mut rcc.enr);
    let mut pc13 = gpioc.pc13.as_output(&mut gpioc.crh);

    pc13.set_low();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

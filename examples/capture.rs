//! Report the length of the periodic signal applied to PB6

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
#[macro_use]
extern crate nb;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use rtfm::app;

app! {
    device: stm32f103xx,
}

fn init(p: init::Peripherals) {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split();
    let mut gpiob = p.device.GPIOB.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();

    // try commenting out this line, but note that you'll have to change .gdbinit if you do so!
    // rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pb6 = gpiob.pb6.as_pull_up_input(&mut gpiob.crl);
    let mut t4c1 = p.device
        .TIM4
        .capture(pb6, 1.ms(), clocks, &mut rcc.enr, &mut afio.mapr);

    t4c1.enable();

    let mut prev = None;
    loop {
        let curr = block!(t4c1.capture()).unwrap();

        if let Some(prev) = prev {
            let elapsed = curr.wrapping_sub(prev);

            iprintln!(&p.core.ITM.stim[0], "{} ms", elapsed);
        }

        prev = Some(curr);
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

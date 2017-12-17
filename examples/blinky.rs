//! Blink the LED connected to pin PC13

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Timer;
use blue_pill::gpio::{Output, PC13};
use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static PC13: PC13<Output>;
        static TIMER: Timer;
    },

    tasks: {
        TIM2: {
            path: tim2,
            resources: [PC13, TIMER],
        },
    },
}

fn init(p: init::Peripherals) -> init::LateResources {
    let mut flash = p.device.FLASH.split();
    let mut rcc = p.device.RCC.split();

    rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioc = p.device.GPIOC.split(&mut rcc.enr);
    let pc13 = gpioc.pc13.as_output(&mut gpioc.crh);

    let mut timer = Timer::new(p.device.TIM2, &mut rcc.enr, clocks, 1.hz());

    timer.resume();

    init::LateResources {
        PC13: pc13,
        TIMER: timer,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn tim2(_t: &mut Threshold, mut r: TIM2::Resources) {
    r.TIMER.wait().unwrap();

    if r.PC13.is_high() {
        r.PC13.set_low();
    } else {
        r.PC13.set_high();
    }
}

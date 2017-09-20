//! Blinks the user LED
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::led::{self, PC13};
use cortex_m::peripheral::SystClkSource;
use rtfm::{app, Threshold};

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static ON: bool = false;
    },

    tasks: {
        SYS_TICK: {
            path: toggle,
            resources: [ON],
        },
    },
}

fn init(p: init::Peripherals, _r: init::Resources) {
    led::init(p.GPIOC, p.RCC);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000); // 1s
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

fn idle() -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
fn toggle(_t: &mut Threshold, r: SYS_TICK::Resources) {
    **r.ON = !**r.ON;

    if **r.ON {
        PC13.on();
    } else {
        PC13.off();
    }
}

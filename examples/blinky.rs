//! Blinks the user LED

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m;
#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::led::{self, Green};
use cortex_m::peripheral::SystClkSource;
use rtfm::{app, Threshold};

app! {
    device: blue_pill::stm32f103xx,

    tasks: {
        SYS_TICK: {
            priority: 1,
        },
    },
}

// INITIALIZATION PHASE
fn init(p: init::Peripherals) {
    led::init(p.GPIOC, p.RCC);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000); // 1s
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

// IDLE LOOP
fn idle() -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
task!(SYS_TICK, blink, Locals {
    static STATE: bool = false;
});

fn blink(_t: &mut Threshold, l: &mut Locals, _r: SYS_TICK::Resources) {
    *l.STATE = !*l.STATE;

    if *l.STATE {
        Green.on();
    } else {
        Green.off();
    }
}

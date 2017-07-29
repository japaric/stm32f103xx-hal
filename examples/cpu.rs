//! CPU usage monitor
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[macro_use(iprint, iprintln)]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Timer;
use blue_pill::time::Hertz;
use blue_pill::prelude::*;
use rtfm::{app, Resource, Threshold};

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static SLEEP_TIME: u32 = 0;
    },

    idle: {
        resources: [DWT, SLEEP_TIME],
    },

    tasks: {
        TIM2: {
            path: periodic,
            resources: [ITM, SLEEP_TIME, TIM2],
        },
    },
}

const FREQUENCY: Hertz = Hertz(1);

fn init(p: init::Peripherals, _r: init::Resources) {
    let timer = Timer(p.TIM2);

    p.DWT.enable_cycle_counter();

    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume();
}

fn idle(t: &mut Threshold, mut r: idle::Resources) -> ! {
    loop {
        // For the span of this critical section the processor will not service
        // interrupts (tasks)
        rtfm::atomic(t, |t| {
            let sleep_time = r.SLEEP_TIME.borrow_mut(t);

            // Sleep
            let before = r.DWT.cyccnt.read();
            rtfm::wfi();
            let after = r.DWT.cyccnt.read();

            let elapsed = after.wrapping_sub(before);

            // Accumulate sleep time
            **sleep_time += elapsed;
        });

        // Tasks are serviced at this point
    }
}

fn periodic(_t: &mut Threshold, r: TIM2::Resources) {
    let timer = Timer(&**r.TIM2);

    timer.wait().unwrap();

    // Report clock cycles spent sleeping
    iprintln!(&r.ITM.stim[0], "{}", **r.SLEEP_TIME);

    // Reset sleep time back to zero
    **r.SLEEP_TIME = 0;
}

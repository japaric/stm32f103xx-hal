//! CPU usage monitor

#![deny(warnings)]
#![feature(const_fn)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;

#[macro_use(iprint, iprintln)]
extern crate cortex_m;

#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Timer;
use blue_pill::stm32f103xx;
use blue_pill::time::Hertz;
use blue_pill::prelude::*;
use rtfm::Threshold;

rtfm! {
    device: stm32f103xx,

    resources: {
        SLEEP_TIME: u32 = 0;
    },

    init: {
        path: init,
    },

    idle: {
        path: idle,
        resources: [DWT, SLEEP_TIME],
    },

    tasks: {
        TIM2: {
            priority: 1,
            enabled: true,
            resources: [ITM, SLEEP_TIME, TIM2],
        },
    },
}

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1);

// INITIALIZATION PHASE
fn init(p: init::Peripherals, _r: init::Resources) {
    let timer = Timer(p.TIM2);

    p.DWT.enable_cycle_counter();

    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume();
}

// IDLE LOOP
fn idle(_t: Threshold, mut r: idle::Resources) -> ! {
    loop {
        // For the span of this critical section the processor will not service
        // interrupts (tasks)
        rtfm::atomic(|cs| {
            let sleep_time = r.SLEEP_TIME.borrow_mut(cs);

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

task!(TIM2, periodic);

fn periodic(_t: Threshold, r: TIM2::Resources) {
    let timer = Timer(r.TIM2);

    timer.wait().unwrap();

    // Report clock cycles spent sleeping
    iprintln!(&r.ITM.stim[0], "{}", **r.SLEEP_TIME);

    // Reset sleep time back to zero
    **r.SLEEP_TIME = 0;
}

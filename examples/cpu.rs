//! CPU usage monitor

#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

// version = "0.2.9"
#[macro_use]
extern crate cortex_m;

extern crate cortex_m_hal as hal;

// version = "0.2.4"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use core::cell::Cell;

use blue_pill::Timer;
use blue_pill::stm32f103xx;
use blue_pill::time::Hertz;
use hal::prelude::*;
use rtfm::{C1, P0, P1, Resource, T0, T1, TMax};
use stm32f103xx::interrupt::TIM1_UP_TIM10;

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1);

// RESOURCES
peripherals!(stm32f103xx, {
    DWT: Peripheral {
        ceiling: C0,
    },
    ITM: Peripheral {
        ceiling: C1,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM1: Peripheral {
        ceiling: C1,
    },
});

// Total sleep time (in clock cycles)
static SLEEP_TIME: Resource<Cell<u32>, C1> = Resource::new(Cell::new(0));

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let dwt = &DWT.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim1 = TIM1.access(prio, thr);

    let timer = Timer(&*tim1);

    dwt.enable_cycle_counter();

    timer.init(FREQUENCY.invert(), rcc);
    timer.resume();
}

// IDLE LOOP
fn idle(ref prio: P0, _thr: T0) -> ! {
    loop {
        // For the span of this critical section the processor will not service
        // interrupts (tasks)
        rtfm::atomic(|thr| {
            let dwt = DWT.access(prio, thr);
            let sleep_time = SLEEP_TIME.access(prio, thr);

            // Sleep
            let before = dwt.cyccnt.read();
            rtfm::wfi();
            let after = dwt.cyccnt.read();

            let elapsed = after.wrapping_sub(before);

            // Accumulate sleep time
            sleep_time.set(sleep_time.get() + elapsed);
        });

        // Tasks are serviced at this point
    }
}

// TASKS
tasks!(stm32f103xx, {
    periodic: Task {
        interrupt: TIM1_UP_TIM10,
        priority: P1,
        enabled: true,
    },
});

fn periodic(_task: TIM1_UP_TIM10, ref prio: P1, ref thr: T1) {
    let itm = ITM.access(prio, thr);
    let sleep_time = SLEEP_TIME.access(prio, thr);
    let tim1 = TIM1.access(prio, thr);

    let timer = Timer(&*tim1);

    // NOTE(unwrap) timeout should have already occurred
    timer.wait().unwrap_or_else(|_| unreachable!());

    // Report clock cycles spent sleeping
    iprintln!(&itm.stim[0], "{}", sleep_time.get());

    // Reset sleep time back to zero
    sleep_time.set(0);
}

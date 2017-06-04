//! CPU usage monitor

#![feature(const_fn)]
#![feature(used)]
#![no_std]

// version = "0.2.9"
#[macro_use]
extern crate cortex_m;

// version = "0.2.4"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate blue_pill;

use core::cell::Cell;

use blue_pill::stm32f103xx;
use blue_pill::timer::Timer;
use rtfm::{C1, P0, P1, Resource, T0, T1, TMax};
use stm32f103xx::interrupt::Tim1UpTim10;

// CONFIGURATION
const FREQUENCY: u32 = 1; // Hz

// RESOURCES
peripherals!(stm32f103xx, {
    DWT: Peripheral {
        register_block: Dwt,
        ceiling: C0,
    },
    ITM: Peripheral {
        register_block: Itm,
        ceiling: C1,
    },
    RCC: Peripheral {
        register_block: Rcc,
        ceiling: C0,
    },
    TIM1: Peripheral {
        register_block: Tim1,
        ceiling: C1,
    },
});

static SLEEP_CYCLES: Resource<Cell<u32>, C1> = Resource::new(Cell::new(0));

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let dwt = &DWT.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim1 = &TIM1.access(prio, thr);

    let timer = Timer(tim1);

    dwt.enable_cycle_counter();

    timer.init(FREQUENCY, rcc);
    timer.resume();
}

// IDLE LOOP
fn idle(ref prio: P0, _thr: T0) -> ! {
    // Sleep
    loop {
        rtfm::atomic(|thr| {
            let dwt = DWT.access(prio, thr);
            let sleep_cycles = SLEEP_CYCLES.access(prio, thr);

            let before = dwt.cyccnt.read();
            rtfm::wfi();
            let after = dwt.cyccnt.read();

            let elapsed = after.wrapping_sub(before);

            sleep_cycles.set(sleep_cycles.get() + elapsed);
        });

        // Tasks are serviced here
    }
}

// TASKS
tasks!(stm32f103xx, {
    periodic: Task {
        interrupt: Tim1UpTim10,
        priority: P1,
        enabled: true,
    },
});

fn periodic(_task: Tim1UpTim10, ref prio: P1, ref thr: T1) {
    let tim1 = &TIM1.access(prio, thr);
    let itm = ITM.access(prio, thr);
    let sleep_cycles = SLEEP_CYCLES.access(prio, thr);

    let timer = Timer(tim1);

    if timer.clear_update_flag().is_ok() {
        // Report clock cycles spent sleeping
        iprintln!(&itm.stim[0], "{}", sleep_cycles.get());

        // Reset sleep cycle counter to zero
        sleep_cycles.set(0);
    } else {
        // Only reachable via `rtfm::request(periodic)`
        unreachable!()
    }
}

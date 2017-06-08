//! Sends "Hello" and then "World" through the ITM port 0
//!
//! You'll need to run these lines in your GDB session
//!
//! ``` text
//! > monitor tpiu config external uart off 8000000 2000000
//! > monitor itm port 0 on
//! ```
//!
//! And connect the SWO (PB3) pin to an UART adapter, or read it by some other
//! means.
//!
//! Finally you should see the output if you monitor the UART adapter device
//! file.
//!
//! ``` console
//! $ cat /dev/ttyUSB0
//! Hello
//! World
//! ```

#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

// version = "0.2.9"
#[macro_use]
extern crate cortex_m;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use blue_pill::stm32f103xx;
use rtfm::{P0, T0, TMax};

// RESOURCES
peripherals!(stm32f103xx, {
    ITM: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let itm = ITM.access(prio, thr);

    iprintln!(&itm.stim[0], "Hello");
}

// IDLE LOOP
fn idle(ref prio: P0, ref thr: T0) -> ! {
    let itm = ITM.access(prio, thr);

    iprintln!(&itm.stim[0], "World");

    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

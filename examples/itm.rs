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

#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;
#[macro_use(iprint, iprintln)]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

rtfm! {
    device: blue_pill::stm32f103xx,

    init: {
        path: init,
    },

    idle: {
        path: idle,
        resources: [ITM],
    },
}

fn init(p: init::Peripherals) {
    iprintln!(&p.ITM.stim[0], "Hello");
}

fn idle(r: idle::Resources) -> ! {
    iprintln!(&r.ITM.stim[0], "World");

    // Sleep
    loop {
        rtfm::wfi();
    }
}

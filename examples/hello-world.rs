//! Prints "Hello" and then "World" in the OpenOCD console

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting as semihosting;

use core::fmt::Write;

use rtfm::app;
use semihosting::hio::{self, HStdout};

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static HSTDOUT: Option<HStdout> = None;
    },

    idle: {
        resources: [HSTDOUT],
    },
}

fn init(_p: init::Peripherals, r: init::Resources) {
    let mut hstdout = hio::hstdout().unwrap();

    writeln!(hstdout, "Hello").unwrap();

    **r.HSTDOUT = Some(hstdout);
}

fn idle(r: idle::Resources) -> ! {
    writeln!(r.HSTDOUT.as_mut().unwrap(), "World").unwrap();

    loop {
        rtfm::wfi();
    }
}

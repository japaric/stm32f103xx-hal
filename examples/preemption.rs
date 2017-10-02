//! Running two tasks that share data concurrently
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Serial;
use blue_pill::led::{self, LED};
use blue_pill::prelude::*;
use blue_pill::serial::Event;
use blue_pill::time::Hertz;
use cortex_m::peripheral::SystClkSource;
use rtfm::{app, Resource, Threshold};

const BAUD_RATE: Hertz = Hertz(115_200);

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static CONTEXT_SWITCHES: u32 = 0;
        static ON: bool = false;
    },

    tasks: {
        SYS_TICK: {
            path: toggle,
            priority: 1,
            resources: [CONTEXT_SWITCHES, ON],
        },

        USART1: {
            path: loopback,
            priority: 2,
            resources: [CONTEXT_SWITCHES, USART1],
        },
    },
}

fn init(p: init::Peripherals, _r: init::Resources) {
    let serial = Serial(p.USART1);

    led::init(p.GPIOC, p.RCC);

    serial.init(BAUD_RATE.invert(), p.AFIO, None, p.GPIOA, p.RCC);
    serial.listen(Event::Rxne);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000); // 1s
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn loopback(_t: &mut Threshold, r: USART1::Resources) {
    **r.CONTEXT_SWITCHES += 1;

    let serial = Serial(&**r.USART1);

    let byte = serial.read().unwrap();
    serial.write(byte).unwrap();
}

fn toggle(t: &mut Threshold, mut r: SYS_TICK::Resources) {
    r.CONTEXT_SWITCHES.claim_mut(t, |context_switches, _t| {
        // inside a critical section
        **context_switches += 1;
    });

    **r.ON = !**r.ON;

    if **r.ON {
        LED.on();
    } else {
        LED.off();
    }
}

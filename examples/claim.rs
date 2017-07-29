#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::stm32f103xx::Interrupt;
use rtfm::{app, Resource, Threshold};

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static R1: bool = false;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [R1],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [R1],
        },

        EXTI2: {
            path: exti2,
            priority: 3,
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn exti0(t: &mut Threshold, r: EXTI0::Resources) {
    // Threshold == 1

    rtfm::set_pending(Interrupt::EXTI1); // ~> exti1

    r.R1.claim(t, |_r1, _t| {
        // Threshold = 2
        rtfm::set_pending(Interrupt::EXTI1);

        rtfm::set_pending(Interrupt::EXTI2); // ~> exti2
    }); // Threshold = 1

    // ~> exti1

    rtfm::atomic(t, |t| {
        // Threshold = MAX
        let _r1 = r.R1.borrow(t);

        rtfm::set_pending(Interrupt::EXTI1);

        rtfm::set_pending(Interrupt::EXTI2);
    }); // Threshold = 1

    // ~> exti2, exti1
}

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {
    // .. modify R1 ..
}

fn exti2() {}

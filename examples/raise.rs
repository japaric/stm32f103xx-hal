#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use cortex_m::asm;
use rtfm::{app, Resource, Threshold};

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static R1: bool = false;
        static R2: bool = false;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [R1, R2],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [R1],
        },

        EXTI2: {
            path: exti2,
            priority: 3,
            resources: [R2],
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
    r.R1.claim(t, |_r1, t| {
        asm::nop();

        r.R2.claim(t, |_r2, _t| {
            asm::nop();
        });

        asm::nop();
    });
}

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {}

fn exti2(_t: &mut Threshold, _r: EXTI2::Resources) {}

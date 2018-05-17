//! Blinks an LED using RTFM on-demand tasks

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]
#![no_main]

#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;

use hal::gpio::gpioc::PC13;
use hal::gpio::{Output, PushPull};
use hal::prelude::*;
use hal::stm32f103xx;
use rt::ExceptionFrame;
use rtfm::app;

type Led = PC13<Output<PushPull>>;

app! {
    device: stm32f103xx,

    resources: {
        static LED: Led;
        static ON: bool = false;
    },

    init: {
        schedule_now: [periodic],
    },

    free_interrupts: [EXTI0],

    tasks: {
        periodic: {
            resources: [LED, ON],
            schedule_after: [periodic],
        },
    },
}

const PERIOD: u32 = 8_000_000; // 1 s

fn init(mut ctxt: init::Context) -> init::LateResources {
    let p = &mut ctxt.priority;
    let mut rcc = ctxt.device.RCC.constrain();
    let mut gpioc = ctxt.device.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high(); // off

    ctxt.tasks.periodic.schedule_now(p).unwrap();

    init::LateResources { LED: led }
}

fn periodic(mut ctxt: periodic::Context) {
    let p = &mut ctxt.priority;

    if *ctxt.resources.ON {
        ctxt.resources.LED.set_low();
    } else {
        ctxt.resources.LED.set_high();
    }

    *ctxt.resources.ON = !*ctxt.resources.ON;

    ctxt.tasks.periodic.schedule_after(p, PERIOD).unwrap();
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

//! Continuously reads serial data using the DMA and processes the data in a separate RTFM task

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_rtfm as rtfm;
extern crate heapless;
extern crate panic_itm;
extern crate stm32f103xx_hal as hal;

use core::str;

use cortex_m::peripheral::ITM;
use hal::dma::{dma1, Event, Transfer, W};
use hal::prelude::*;
use hal::serial::{Rx, Serial};
use hal::stm32f103xx::{self, USART1};
use heapless::consts::*;
use heapless::object_pool::{Object, ObjectPool};
use heapless::GenericArray;
use rt::ExceptionFrame;
use rtfm::app;

type T1 = Transfer<W, Object<P1>, dma1::C5, Rx<USART1>>;

app! {
    device: stm32f103xx,

    resources: {
        static ITM: ITM;
        static POOL: ObjectPool<P1>;
        static PREV_TRANSFER: Option<T1>;
        static P1: GenericArray<[u8; 16], U2>;
    },

    free_interrupts: [EXTI0],

    init: {
        resources: [P1],
    },

    tasks: {
        rx: {
            priority: 2,
            interrupt: DMA1_CHANNEL5,
            resources: [PREV_TRANSFER, POOL],
            schedule_now: [process],
        },

        process: {
            // priority: 1,
            input: Object<P1>,
            instances: 2,
            resources: [ITM, POOL],
        },
    },
}

fn init(ctxt: init::Context) -> init::LateResources {
    let mut flash = ctxt.device.FLASH.constrain();
    let mut rcc = ctxt.device.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = ctxt.device.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = ctxt.device.GPIOA.split(&mut rcc.apb2);

    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let serial = Serial::usart1(
        ctxt.device.USART1,
        (tx, rx),
        &mut afio.mapr,
        115_200.bps(),
        clocks,
        &mut rcc.apb2,
    );

    let rx = serial.split().1;

    let mut channels = ctxt.device.DMA1.split(&mut rcc.ahb);
    channels.5.listen(Event::TransferComplete);

    let mut op = ObjectPool::new(ctxt.resources.P1);

    init::LateResources {
        PREV_TRANSFER: Some(rx.read_exact(channels.5, op.get().unwrap().noinit())),
        POOL: op,
        ITM: ctxt.core.ITM,
    }
}

fn rx(mut ctxt: rx::Context) {
    let p = &mut ctxt.priority;
    let trans = ctxt.resources.PREV_TRANSFER.take().expect("unreachable");

    debug_assert!(trans.is_done()); // sanity check
    let (buf, chan, rx) = trans.wait();

    // start a new transfer
    *ctxt.resources.PREV_TRANSFER = Some(
        rx.read_exact(
            chan,
            ctxt.resources
                .POOL
                .borrow_mut(p)
                .get()
                .expect("pool exhausted")
                .noinit(),
        ),
    );

    ctxt.tasks.process.schedule_now(p, buf).ok().expect("too many tasks scheduled");
}

fn process(mut ctxt: process::Context) {
    let p = &mut ctxt.priority;
    let buf = ctxt.input;

    // .. do stuff with buf ..
    let stim = &mut ctxt.resources.ITM.stim[0];
    if let Ok(s) = str::from_utf8(&*buf) {
        iprintln!(stim, "{}", s);
    } else {
        iprintln!(stim, "not UTF-8");
    }

    // return the buffer to the pool
    ctxt.resources.POOL.claim_mut(p, |p, _| {
        p.free(buf);
    });
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

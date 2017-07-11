//! Test sending serial data using the DMA

#![deny(warnings)]
#![feature(const_fn)]
#![feature(plugin)]
#![no_std]
#![plugin(cortex_m_rtfm_macros)]

extern crate blue_pill;
#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate nb;

use blue_pill::Serial;
use blue_pill::dma::{Buffer, Dma1Channel4};
use blue_pill::time::Hertz;
use rtfm::Threshold;

// CONFIGURATION
pub const BAUD_RATE: Hertz = Hertz(115_200);

rtfm! {
    device: blue_pill::stm32f103xx,

    resources: {
        BUFFER: Buffer<[u8; 14], Dma1Channel4> = Buffer::new([0; 14]);
    },

    init: {
        path: init,
    },

    idle: {
        path: idle,
    },

    tasks: {
        DMA1_CHANNEL4: {
            enabled: true,
            priority: 1,
            resources: [BUFFER, DMA1],
        },
    },
}

fn init(p: init::Peripherals, r: init::Resources) {
    let serial = Serial(p.USART1);

    serial.init(BAUD_RATE.invert(), p.AFIO, Some(p.DMA1), p.GPIOA, p.RCC);
    r.BUFFER.borrow_mut().clone_from_slice(b"Hello, world!\n");

    serial.write_all(p.DMA1, r.BUFFER).unwrap();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

task!(DMA1_CHANNEL4, transfer_done);

fn transfer_done(_t: Threshold, r: DMA1_CHANNEL4::Resources) {
    r.BUFFER.release(r.DMA1).unwrap();

    rtfm::bkpt();
}

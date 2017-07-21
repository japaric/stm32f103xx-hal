//! Test receiving serial data using the DMA

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate nb;

use blue_pill::Serial;
use blue_pill::dma::{Buffer, Dma1Channel5};
use blue_pill::time::Hertz;
use rtfm::{app, Threshold};

pub const BAUD_RATE: Hertz = Hertz(115_200);

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static BUFFER: Buffer<[u8; 8], Dma1Channel5> = Buffer::new([0; 8]);
    },

    tasks: {
        DMA1_CHANNEL5: {
            enabled: true,
            priority: 1,
            resources: [BUFFER, DMA1],
        },
    },
}

fn init(p: init::Peripherals, r: init::Resources) {
    let serial = Serial(p.USART1);

    serial.init(BAUD_RATE.invert(), p.AFIO, Some(p.DMA1), p.GPIOA, p.RCC);

    serial.read_exact(p.DMA1, r.BUFFER).unwrap();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

task!(DMA1_CHANNEL5, transfer_done);

fn transfer_done(_t: &mut Threshold, r: DMA1_CHANNEL5::Resources) {
    r.BUFFER.release(r.DMA1).unwrap();

    rtfm::bkpt();
}

//! Test the USART1 instance
//!
//! Connect the TX and RX pins to run this test

#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

extern crate embedded_hal as hal;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate nb;

use blue_pill::dma::{Buffer, Dma1Channel5};
use blue_pill::time::Hertz;
use blue_pill::{Serial, stm32f103xx};
use rtfm::{C1, P0, P1, Resource, T0, T1, TMax};
use stm32f103xx::interrupt::DMA1_CHANNEL5;

// CONFIGURATION
pub const BAUD_RATE: Hertz = Hertz(115_200);

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    DMA1: Peripheral {
        ceiling: C1,
    },
    GPIOA: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    USART1: Peripheral {
        ceiling: C1,
    },
});

static BUFFER: Resource<Buffer<[u8; 8], Dma1Channel5>, C1> =
    Resource::new(Buffer::new([0; 8]));

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let dma1 = &DMA1.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let usart1 = USART1.access(prio, thr);
    let buffer = BUFFER.access(prio, thr);

    let serial = Serial(&*usart1);

    serial.init(BAUD_RATE.invert(), afio, Some(dma1), gpioa, rcc);

    serial.read_exact(dma1, buffer).unwrap();
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {
    done: Task {
        interrupt: DMA1_CHANNEL5,
        priority: P1,
        enabled: true,
    },
});

fn done(_task: DMA1_CHANNEL5, ref prio: P1, ref thr: T1) {
    let buffer = BUFFER.access(prio, thr);
    let dma1 = &DMA1.access(prio, thr);

    buffer.free(dma1).unwrap();

    rtfm::bkpt();
}

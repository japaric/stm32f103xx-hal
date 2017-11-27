//! Loopback using the DMA (lockless version)

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;
extern crate heapless;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use blue_pill::dma::{D1C4, D1C5, Read, Static, Transfer, Write};
use blue_pill::serial::{Rx, Serial, Tx};
use rtfm::{app, Threshold};
use heapless::ring_buffer::{Consumer, Producer, RingBuffer};

const N: usize = 4;
type T1 = Transfer<Write, D1C5, [u8; N], Rx>;
type T2 = Transfer<Read, D1C4, [u8; N], Tx>;
type M = (D1C4, Static<[u8; N]>, Tx);

app! {
    device: stm32f103xx,

    resources: {
        static RX_BUFFER: [u8; N] = [0; N];
        static TX_BUFFER: [u8; N] = [0; N];
        static T1: Option<T1>;

        static RB1: RingBuffer<M, [M; 2]> = RingBuffer::new();
        static P1: Producer<'static, M, [M; 2]>;
        static C1: Consumer<'static, M, [M; 2]>;

        static RB2: RingBuffer<T2, [T2; 2]> = RingBuffer::new();
        static P2: Producer<'static, T2, [T2; 2]>;
        static C2: Consumer<'static, T2, [T2; 2]>;
    },

    init: {
        resources: [RX_BUFFER, TX_BUFFER, RB1, RB2],
    },

    tasks: {
        DMA1_CHANNEL4: {
            path: tx_end,
            resources: [P1, C2],
        },

        DMA1_CHANNEL5: {
            path: rx_end,
            resources: [T1, C1, P2],
        },
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResourceValues {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);
    let channels = p.device.DMA1.split(&mut rcc.enr);

    // try commenting this out
    rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa9 = gpioa.pa9.as_alt_push(&mut gpioa.crh);
    let pa10 = gpioa.pa10;

    let (tx, rx) = Serial::new(
        p.device.USART1,
        (pa9, pa10),
        115_200.bps(),
        clocks,
        &mut rcc.enr,
        &mut afio.mapr,
    ).split();

    let (mut p1, c1) = r.RB1.split();

    p1.enqueue((channels.4, r.TX_BUFFER, tx)).unwrap();

    let (p2, c2) = r.RB2.split();

    init::LateResourceValues {
        T1: Some(rx.read_exact(channels.5, r.RX_BUFFER)),
        C1: c1,
        P1: p1,
        C2: c2,
        P2: p2,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn rx_end(_t: &mut Threshold, r: DMA1_CHANNEL5::Resources) {
    if let Some(transfer) = r.T1.take() {
        // sanity check: this task starts when the transfer is done
        debug_assert!(transfer.is_done().unwrap());

        let (rx_chan, rx_buf, rx) = transfer.wait().unwrap();

        let (tx_chan, tx_buf, tx) = r.C1.dequeue().expect("scheduling problem");

        tx_buf.copy_from_slice(rx_buf);
        r.P2.enqueue(tx.write_all(tx_chan, tx_buf)).unwrap();

        **r.T1 = Some(rx.read_exact(rx_chan, rx_buf));
    } else {
        // NOTE(unreachable!) `T1` is always populated
        #[cfg(debug_assertions)]
        unreachable!()
    }
}

fn tx_end(_t: &mut Threshold, r: DMA1_CHANNEL4::Resources) {
    if let Some(transfer) = r.C2.dequeue() {
        // sanity check: this task starts when the transfer is done
        debug_assert!(transfer.is_done().unwrap());

        r.P1.enqueue(transfer.wait().unwrap()).unwrap();
    } else {
        // NOTE(unreachable!) `C2` always contains the message send by `rx_end`
        #[cfg(debug_assertions)]
        unreachable!()
    }
}

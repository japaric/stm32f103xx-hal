#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(use_extern_macros)]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate nb;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;

use hal::prelude::*;
use hal::stm32f103xx;
use rt::ExceptionFrame;

extern crate cortex_m_semihosting as sh;
use hal::can::*;

use core::fmt::Write;
use sh::hio;

entry!(main);

fn main() -> ! {
    let config = Configuration {
        time_triggered_communication_mode: false,
        automatic_bus_off_management: true,
        automatic_wake_up_mode: true,
        no_automatic_retransmission: false,
        receive_fifo_locked_mode: false,
        transmit_fifo_priority: false,
        silent_mode: false,
        loopback_mode: true,
        synchronisation_jump_width: 1,
        bit_segment_1: 3,
        bit_segment_2: 2,
        time_quantum_length: 6,
    };

    let dp = stm32f103xx::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let canrx = gpioa.pa11.into_floating_input(&mut gpioa.crh);
    let cantx = gpioa.pa12.into_alternate_push_pull(&mut gpioa.crh);

    //remapped version:
    //let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    //let canrx = gpiob.pb8.into_floating_input(&mut gpiob.crh);
    //let cantx = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    //USB is needed here because it can not be used at the same time as CAN since they share memory:
    let mut can = Can::can1(
        dp.CAN,
        (cantx, canrx),
        &mut afio.mapr,
        &mut rcc.apb1,
        dp.USB,
    );

    can.configure(&config);
    nb::block!(can.to_normal()).unwrap(); //just to be sure

    let id10: Id = Id::new_standard(10);
    let id11: Id = Id::new_standard(11);
    let id12: Id = Id::new_standard(12);
    let id13: Id = Id::new_standard(13);
    let id14: Id = Id::new_standard(14);
    let id15: Id = Id::new_standard(15);

    let filterbank0_config = FilterBankConfiguration {
        mode: FilterMode::List,
        info: FilterInfo::Whole(FilterData {
            id: id10.clone(),
            mask_or_id2: id11.clone(),
        }),
        fifo_assignment: 0,
        active: true,
    };
    let filterbank1_config = FilterBankConfiguration {
        mode: FilterMode::List,
        info: FilterInfo::Whole(FilterData {
            id: id12.clone(),
            mask_or_id2: id13.clone(),
        }),
        fifo_assignment: 1,
        active: true,
    };
    let filterbank2_config = FilterBankConfiguration {
        mode: FilterMode::List,
        info: FilterInfo::Whole(FilterData {
            id: id14.with_rtr(),
            mask_or_id2: id14.clone(),
        }),
        fifo_assignment: 0,
        active: true,
    };
    let filterbank3_config = FilterBankConfiguration {
        mode: FilterMode::List,
        info: FilterInfo::Whole(FilterData {
            id: id15.with_rtr(),
            mask_or_id2: id15.clone(),
        }),
        fifo_assignment: 1,
        active: true,
    };
    can.configure_filter_bank(0, &filterbank0_config);
    can.configure_filter_bank(1, &filterbank1_config);
    can.configure_filter_bank(2, &filterbank2_config);
    can.configure_filter_bank(3, &filterbank3_config);

    let mut hstdout = hio::hstdout().unwrap();

    let (tx, rx) = can.split();

    let (mut tx0, mut tx1, mut tx2) = tx.split();

    let txresult0 = tx0.request_transmit(&Frame::new(id10, Payload::new(b"0")));
    let txresult1 = tx1.request_transmit(&Frame::new(id11, Payload::new(b"1")));
    let txresult2 = tx2.request_transmit(&Frame::new(id12, Payload::new(b"2")));
    writeln!(
        hstdout,
        "tx: {:?} {:?} {:?}",
        &txresult0, &txresult1, &txresult2
    ).unwrap(); //while this printing slowly, all 3 messages are transfered
    let txresult0 = tx0.request_transmit(&Frame::new(id13, Payload::new(b"3")));
    let txresult1 = tx1.request_transmit(&Frame::new(id14, Payload::new(b"4")));
    let txresult2 = tx2.request_transmit(&Frame::new(id15, Payload::new(b"5")));
    writeln!(
        hstdout,
        "tx: {:?} {:?} {:?}",
        &txresult0, &txresult1, &txresult2
    ).unwrap(); //while this printing slowly, all 3 messages are transfered

    let (mut rx0, mut rx1) = rx.split();
    loop {
        if let Ok((filter_match_index, time, frame)) = rx0.read() {
            writeln!(
                hstdout,
                "rx0: {} {} {} {} {}",
                filter_match_index,
                frame.id().standard(),
                time,
                frame.data().len(),
                frame.data().data_as_u64()
            ).unwrap();
        };

        if let Ok((filter_match_index, time, frame)) = rx1.read() {
            writeln!(
                hstdout,
                "rx1: {} {} {} {} {}",
                filter_match_index,
                frame.id().standard(),
                time,
                frame.data().len(),
                frame.data().data_as_u64()
            ).unwrap();
        };
    }

    //writeln!(hstdout, "done.").unwrap();
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

//! Blinks an LED

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::asm;
use stm32f103xx_hal::{
    prelude::*,
    dma::{dma1, CircBuffer, Event},
    serial::Serial,
};
use rtfm::app;

#[app(device = stm32f103xx_hal::device)]
const APP: () = {
    static mut BUFFER: [[u8; 8]; 2] = [[0; 8]; 2];
    static mut CB: CircBuffer<[u8; 8], dma1::C5> = ();

    #[init(resources = [BUFFER])]
    fn init() {
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let mut afio = device.AFIO.constrain(&mut rcc.apb2);

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);

        // USART1
        let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
        let rx = gpioa.pa10;

        // USART1
        // let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        // let rx = gpiob.pb7;

        // USART2
        // let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        // let rx = gpioa.pa3;

        // USART3
        // let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
        // let rx = gpiob.pb11;

        let serial = Serial::usart1(
            device.USART1,
            (tx, rx),
            &mut afio.mapr,
            9_600.bps(),
            clocks,
            &mut rcc.apb2,
        );

        let rx = serial.split().1;

        let mut channels = device.DMA1.split(&mut rcc.ahb);
        channels.5.listen(Event::HalfTransfer);
        channels.5.listen(Event::TransferComplete);
        
        CB = rx.circ_read(channels.5, resources.BUFFER);
    }

    #[interrupt(resources = [CB])]
    fn DMA1_CHANNEL5() {
        resources.CB
            .peek(|_buf, _half| {
                asm::bkpt();
            })
            .unwrap();
    }
};

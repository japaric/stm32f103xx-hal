//! Log IMU data from MPU9250

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate byteorder;
extern crate cobs;
extern crate cortex_m_rtfm as rtfm;
extern crate crc16;
extern crate either;
extern crate mpu9250;
extern crate panic_abort;
extern crate stm32f103xx_hal as hal;

use byteorder::{ByteOrder, LE};
use crc16::{State, ARC};
use either::Either;
use hal::delay::Delay;
use hal::dma::{dma1, Transfer, R};
use hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use hal::gpio::{Alternate, Floating, Input, Output, PushPull};
use hal::prelude::*;
use hal::serial::{Serial, Tx};
use hal::spi::Spi;
use hal::stm32f103xx;
use hal::timer::{self, Timer};
use mpu9250::{Imu, Mpu9250};
use rtfm::{app, Threshold};
use stm32f103xx::{SPI1, USART1};

// CONNECTIONS
#[allow(non_camel_case_types)]
type TX_BUF = &'static mut [u8; TX_SZ];

type MPU9250 = Mpu9250<
    Spi<
        SPI1,
        (
            PA5<Alternate<PushPull>>,
            PA6<Input<Floating>>,
            PA7<Alternate<PushPull>>,
        ),
    >,
    PA4<Output<PushPull>>,
    Imu,
>;

type TX = Option<Either<(TX_BUF, dma1::C4, Tx<USART1>), Transfer<R, TX_BUF, dma1::C4, Tx<USART1>>>>;

const TX_SZ: usize = 10;
const FREQ: u32 = 1024;
const SAMPLES: u32 = FREQ * 10;

// RESOURCES
app! {
    device: stm32f103xx,

    resources: {
        static MPU9250: MPU9250;
        static TX: TX;

        static SAMPLES: u32 = 0;
        static TX_BUF: [u8; TX_SZ] = [0; TX_SZ];
    },

    init: {
        resources: [TX_BUF],
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [MPU9250, SAMPLES, TX],
        },
    },
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {
    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();

    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let clocks = rcc.cfgr
        .sysclk(64.mhz())
        .pclk1(32.mhz())
        .freeze(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);

    // SERIAL
    let pa9 = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pa10 = gpioa.pa10;

    let serial = Serial::usart1(
        p.device.USART1,
        (pa9, pa10),
        &mut afio.mapr,
        115_200.bps(),
        clocks,
        &mut rcc.apb2,
    );

    let mut tx = serial.split().0;

    // start COBS frame
    tx.write(0x00).unwrap();

    // DMA
    let channels = p.device.DMA1.split(&mut rcc.ahb);

    // SPI
    let nss = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    let spi = Spi::spi1(
        p.device.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        mpu9250::MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    // MPU9250
    let mut delay = Delay::new(p.core.SYST, clocks);

    let mpu9250 = Mpu9250::imu(spi, nss, &mut delay).unwrap();

    Timer::syst(delay.free(), FREQ.hz(), clocks).listen(timer::Event::Update);

    init::LateResources {
        MPU9250: mpu9250,
        TX: Some(Either::Left((r.TX_BUF, channels.4, tx))),
    }
}

// TASKS
fn idle() -> ! {
    loop {
        rtfm::wfi()
    }
}

fn tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    let (ary, arz, _, gx) = r.MPU9250.aryz_t_gx().unwrap();

    let (buf, c, tx) = match r.TX.take().unwrap() {
        Either::Left((buf, c, tx)) => (buf, c, tx),
        Either::Right(trans) => trans.wait(),
    };

    let mut data = [0; TX_SZ - 2];

    LE::write_i16(&mut data[0..2], ary);
    LE::write_i16(&mut data[2..4], arz);
    LE::write_i16(&mut data[4..6], gx);

    let crc = State::<ARC>::calculate(&data[..TX_SZ - 4]);
    LE::write_u16(&mut data[TX_SZ - 4..], crc);

    cobs::encode(&data, buf);

    *r.TX = Some(Either::Right(tx.write_all(c, buf)));

    *r.SAMPLES += 1;
    if *r.SAMPLES >= SAMPLES {
        *r.SAMPLES = 0;
        rtfm::bkpt();
    }
}

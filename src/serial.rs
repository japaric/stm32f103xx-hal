use core::marker::PhantomData;
use core::ptr;

use hal;
use nb;
use stm32f103xx::{USART1, USART2, USART3};

use afio::MAPR;
use gpio::gpioa::{PA10, PA2, PA3, PA9};
use gpio::gpiob::{PB10, PB11, PB6, PB7};
use gpio::{Alternate, Floating, Input, PushPull};
use rcc::{APB1, APB2, Clocks};
use time::Bps;

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
}

/// Serial error
#[derive(Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    #[doc(hidden)] _Extensible,
}

pub trait Pins<USART> {
    const REMAP: u8;
}

impl Pins<USART1> for (PA9<Alternate<PushPull>>, PA10<Input<Floating>>) {
    const REMAP: u8 = 0;
}

impl Pins<USART1> for (PB6<Alternate<PushPull>>, PB7<Input<Floating>>) {
    const REMAP: u8 = 1;
}

impl Pins<USART2> for (PA2<Alternate<PushPull>>, PA3<Input<Floating>>) {
    const REMAP: u8 = 0;
}

// impl Pins<USART2> for (PD5<Alternate<PushPull>>, PD6<Input<Floating>>) {
//     const REMAP: u8 = 0;
// }

impl Pins<USART3> for (PB10<Alternate<PushPull>>, PB11<Input<Floating>>) {
    const REMAP: u8 = 0;
}

// impl Pins<USART3> for (PC10<Alternate<PushPull>>, PC11<Input<Floating>>) {
//     const REMAP: u8 = 1;
// }

// impl Pins<USART3> for (PD8<Alternate<PushPull>>, PD9<Input<Floating>>) {
//     const REMAP: u8 = 0b11;
// }

/// Serial abstraction
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

/// Serial receiver
pub struct Rx<USART> {
    _usart: PhantomData<USART>,
}

/// Serial transmitter
pub struct Tx<USART> {
    _usart: PhantomData<USART>,
}

macro_rules! hal {
    ($(
        $USARTX:ident: (
            $usartX:ident,
            $usartXen:ident,
            $usartXrst:ident,
            $usartX_remap:ident,
            $bit:ident,
            $closure:expr,
            $APB:ident
        ),
    )+) => {
        $(
            impl<PINS> Serial<$USARTX, PINS> {
                pub fn $usartX(
                    usart: $USARTX,
                    pins: PINS,
                    mapr: &mut MAPR,
                    baud_rate: Bps,
                    clocks: Clocks,
                    apb: &mut $APB,
                ) -> Self
                where
                    PINS: Pins<$USARTX>,
                {
                    // enable and reset $USARTX
                    apb.enr().modify(|_, w| w.$usartXen().enabled());
                    apb.rstr().modify(|_, w| w.$usartXrst().set_bit());
                    apb.rstr().modify(|_, w| w.$usartXrst().clear_bit());

                    #[allow(unused_unsafe)]
                    mapr.mapr()
                        .modify(|_, w| unsafe{
                            w.$usartX_remap().$bit(($closure)(PINS::REMAP))
                        });

                    // disable hardware flow control
                    // TODO enable DMA
                    // usart.cr3.write(|w| w.rtse().clear_bit().ctse().clear_bit());

                    let brr = clocks.pclk2().0 / baud_rate.0;
                    assert!(brr >= 16, "impossible baud rate");
                    usart.brr.write(|w| unsafe { w.bits(brr) });

                    // UE: enable USART
                    // RE: enable receiver
                    // TE: enable transceiver
                    usart
                        .cr1
                        .write(|w| w.ue().set_bit().re().set_bit().te().set_bit());

                    Serial { usart, pins }
                }

                pub fn release(self) -> ($USARTX, PINS) {
                    (self.usart, self.pins)
                }

                pub fn split(self) -> (Tx<$USARTX>, Rx<$USARTX>) {
                    (
                        Tx {
                            _usart: PhantomData,
                        },
                        Rx {
                            _usart: PhantomData,
                        },
                    )
                }
            }

            impl hal::serial::Read<u8> for Rx<$USARTX> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    // NOTE(unsafe) atomic read with no side effects
                    let sr = unsafe { (*$USARTX::ptr()).sr.read() };

                    Err(if sr.pe().bit_is_set() {
                        nb::Error::Other(Error::Parity)
                    } else if sr.fe().bit_is_set() {
                        nb::Error::Other(Error::Framing)
                    } else if sr.ne().bit_is_set() {
                        nb::Error::Other(Error::Noise)
                    } else if sr.ore().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.rxne().bit_is_set() {
                        // NOTE(read_volatile) see `write_volatile` below
                        return Ok(unsafe {
                            ptr::read_volatile(&(*$USARTX::ptr()).dr as *const _ as *const _)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl hal::serial::Write<u8> for Tx<$USARTX> {
                type Error = !;

                fn flush(&mut self) -> nb::Result<(), !> {
                    // NOTE(unsafe) atomic read with no side effects
                    let sr = unsafe { (*$USARTX::ptr()).sr.read() };

                    if sr.tc().bit_is_set() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn write(&mut self, byte: u8) -> nb::Result<(), !> {
                    // NOTE(unsafe) atomic read with no side effects
                    let sr = unsafe { (*$USARTX::ptr()).sr.read() };

                    if sr.txe().bit_is_set() {
                        // NOTE(unsafe) atomic write to stateless register
                        // NOTE(write_volatile) 8-bit write that's not possible through the svd2rust API
                        unsafe {
                            ptr::write_volatile(&(*$USARTX::ptr()).dr as *const _ as *mut _, byte)
                        }
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }
        )+
    }
}

hal! {
    USART1: (usart1, usart1en, usart1rst, usart1_remap, bit, |remap| remap == 1, APB2),
    USART2: (usart2, usart2en, usart2rst, usart2_remap, bit, |remap| remap == 1, APB1),
    USART3: (usart3, usart3en, usart3rst, usart3_remap, bits, |remap| remap, APB1),
}

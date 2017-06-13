//! Serial interface
//!
//! You can use the `Serial` interface with these USART instances
//!
//! # USART1
//!
//! - TX = PA9
//! - RX = PA10
//! - Interrupt = USART1
//!
//! # USART2
//!
//! - TX = PA2
//! - RX = PA3
//! - Interrupt = USART2
//!
//! # USART3
//!
//! - TX = PB10
//! - RX = PB11
//! - Interrupt = USART3

use core::any::{Any, TypeId};
use core::ops::Deref;
use core::ptr;

use cast::u16;
use hal;
use nb;
use static_ref::Ref;
use stm32f103xx::{AFIO, DMA1, GPIOA, GPIOB, RCC, USART1, USART2, USART3,
                  gpioa, usart1};

use dma::{self, Buffer, Dma1Channel4, Dma1Channel5};

/// Specialized `Result` type
pub type Result<T> = ::core::result::Result<T, nb::Error<Error>>;

/// IMPLEMENTATION DETAIL
pub unsafe trait Usart: Deref<Target = usart1::RegisterBlock> {
    /// IMPLEMENTATION DETAIL
    type GPIO: Deref<Target = gpioa::RegisterBlock>;
    /// IMPLEMENTATION DETAIL
    type Ticks: Into<u32>;
}

unsafe impl Usart for USART1 {
    type GPIO = GPIOA;
    type Ticks = ::apb2::Ticks;
}

unsafe impl Usart for USART2 {
    type GPIO = GPIOA;
    type Ticks = ::apb1::Ticks;
}

unsafe impl Usart for USART3 {
    type GPIO = GPIOB;
    type Ticks = ::apb1::Ticks;
}

/// An error
#[derive(Debug)]
pub enum Error {
    /// De-synchronization, excessive noise or a break character detected
    Framing,
    /// Noise detected in the received frame
    Noise,
    /// RX buffer overrun
    Overrun,
    #[doc(hidden)]
    _Extensible,
}

/// Interrupt event
pub enum Event {
    /// RX buffer Not Empty (new data available)
    Rxne,
    /// Transmission Complete
    Tc,
    /// TX buffer Empty (more data can be send)
    Txe,
}

/// Serial interface
///
/// # Interrupts
///
/// - RXNE
pub struct Serial<'a, U>(pub &'a U)
where
    U: Any + Usart;

impl<'a, U> Clone for Serial<'a, U>
where
    U: Any + Usart,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, U> Copy for Serial<'a, U>
where
    U: Any + Usart,
{
}

impl<'a, U> Serial<'a, U>
where
    U: Any + Usart,
{
    /// Initializes the serial interface with a baud rate of `baut_rate` bits
    /// per second
    ///
    /// The serial interface will be configured to use 8 bits of data, 1 stop
    /// bit, no hardware control and to omit parity checking
    pub fn init<B>(
        &self,
        baud_rate: B,
        afio: &AFIO,
        dma1: Option<&DMA1>,
        gpio: &U::GPIO,
        rcc: &RCC,
    ) where
        B: Into<U::Ticks>,
    {
        self._init(baud_rate.into(), afio, dma1, gpio, rcc)
    }

    fn _init(
        &self,
        baud_rate: U::Ticks,
        afio: &AFIO,
        dma1: Option<&DMA1>,
        gpio: &U::GPIO,
        rcc: &RCC,
    ) {
        let usart = self.0;

        // power up peripherals
        if dma1.is_some() {
            rcc.ahbenr.modify(|_, w| w.dma1en().enabled());
        }
        if usart.get_type_id() == TypeId::of::<USART1>() {
            rcc.apb2enr.modify(|_, w| {
                w.afioen().enabled().iopaen().enabled().usart1en().enabled()
            });
        } else if usart.get_type_id() == TypeId::of::<USART2>() {
            rcc.apb1enr.modify(|_, w| w.usart2en().enabled());
            rcc.apb2enr.modify(
                |_, w| w.afioen().enabled().iopaen().enabled(),
            );
        } else if usart.get_type_id() == TypeId::of::<USART3>() {
            rcc.apb1enr.modify(|_, w| w.usart3en().enabled());
            rcc.apb2enr.modify(
                |_, w| w.afioen().enabled().iopben().enabled(),
            );
        }

        if usart.get_type_id() == TypeId::of::<USART1>() {
            // PA9 = TX, PA10 = RX
            afio.mapr.modify(|_, w| w.usart1_remap().clear());
            gpio.crh.modify(|_, w| {
                w.mode9()
                    .output()
                    .cnf9()
                    .alt_push()
                    .mode10()
                    .input()
                    .cnf10()
                    .bits(0b01)
            });
        } else if usart.get_type_id() == TypeId::of::<USART2>() {
            // PA2 = TX, PA3 = RX
            afio.mapr.modify(|_, w| w.usart2_remap().clear());
            gpio.crl.modify(|_, w| {
                w.mode2()
                    .output()
                    .cnf2()
                    .alt_push()
                    .mode3()
                    .input()
                    .cnf3()
                    .bits(0b01)
            });
        } else if usart.get_type_id() == TypeId::of::<USART3>() {
            // PB10 = TX, PB11 = RX
            afio.mapr.modify(
                |_, w| unsafe { w.usart3_remap().bits(0b00) },
            );
            gpio.crh.modify(|_, w| {
                w.mode10()
                    .output()
                    .cnf10()
                    .alt_push()
                    .mode11()
                    .input()
                    .cnf11()
                    .bits(0b01)
            });
        }

        if let Some(dma1) = dma1 {
            if usart.get_type_id() == TypeId::of::<USART1>() {
                // TX DMA transfer
                // mem2mem: Memory to memory mode disabled
                // pl: Medium priority
                // msize: Memory size = 8 bits
                // psize: Peripheral size = 8 bits
                // minc: Memory increment mode enabled
                // pinc: Peripheral increment mode disabled
                // circ: Circular mode disabled
                // dir: Transfer from memory to peripheral
                // tceie: Transfer complete interrupt enabled
                // en: Disabled
                dma1.ccr4.write(|w| unsafe {
                    w.mem2mem()
                        .clear()
                        .pl()
                        .bits(0b01)
                        .msize()
                        .bits(0b00)
                        .psize()
                        .bits(0b00)
                        .minc()
                        .set()
                        .circ()
                        .clear()
                        .pinc()
                        .clear()
                        .dir()
                        .set()
                        .tcie()
                        .set()
                        .en()
                        .clear()
                });

                // RX DMA transfer
                // mem2mem: Memory to memory mode disabled
                // pl: Medium priority
                // msize: Memory size = 8 bits
                // psize: Peripheral size = 8 bits
                // minc: Memory increment mode enabled
                // pinc: Peripheral increment mode disabled
                // circ: Circular mode disabled
                // dir: Transfer from peripheral to memory
                // tceie: Transfer complete interrupt enabled
                // en: Disabled
                dma1.ccr5.write(|w| unsafe {
                    w.mem2mem()
                        .clear()
                        .pl()
                        .bits(0b01)
                        .msize()
                        .bits(0b00)
                        .psize()
                        .bits(0b00)
                        .minc()
                        .set()
                        .circ()
                        .clear()
                        .pinc()
                        .clear()
                        .dir()
                        .clear()
                        .tcie()
                        .set()
                        .en()
                        .clear()
                });
            } else {
                // TODO enable DMA for USART{2,3}
                unimplemented!()
            }
        }

        // 8N1
        usart.cr2.write(|w| unsafe { w.stop().bits(0b00) });

        // baud rate
        let brr = baud_rate.into();

        assert!(brr >= 16, "impossible baud rate");

        usart.brr.write(|w| unsafe { w.bits(brr) });

        // disable hardware flow control
        // enable DMA TX and RX transfers
        usart.cr3.write(|w| {
            w.rtse().clear().ctse().clear().dmat().set().dmar().set()
        });

        // enable TX, RX; disable parity checking
        usart.cr1.write(|w| {
            w.ue()
                .set()
                .re()
                .set()
                .te()
                .set()
                .m()
                .clear()
                .pce()
                .clear()
                .rxneie()
                .clear()
        });
    }

    /// Starts listening for an interrupt `event`
    pub fn listen(&self, event: Event) {
        let usart = self.0;

        match event {
            Event::Rxne => usart.cr1.modify(|_, w| w.rxneie().set()),
            Event::Tc => usart.cr1.modify(|_, w| w.tcie().set()),
            Event::Txe => usart.cr1.modify(|_, w| w.txeie().set()),
        }
    }

    /// Stops listening for an interrupt `event`
    pub fn unlisten(&self, event: Event) {
        let usart = self.0;

        match event {
            Event::Rxne => usart.cr1.modify(|_, w| w.rxneie().clear()),
            Event::Tc => usart.cr1.modify(|_, w| w.tcie().clear()),
            Event::Txe => usart.cr1.modify(|_, w| w.txeie().clear()),
        }
    }
}

impl<'a, U> hal::Serial for Serial<'a, U>
where
    U: Any + Usart,
{
    type Error = Error;

    fn read(&self) -> Result<u8> {
        let usart1 = self.0;
        let sr = usart1.sr.read();

        if sr.ore().is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.ne().is_set() {
            Err(nb::Error::Other(Error::Noise))
        } else if sr.fe().is_set() {
            Err(nb::Error::Other(Error::Framing))
        } else if sr.rxne().is_set() {
            // NOTE(read_volatile) the register is 9 bits big but we'll only
            // work with the first 8 bits
            Ok(unsafe {
                ptr::read_volatile(&usart1.dr as *const _ as *const u8)
            })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write(&self, byte: u8) -> Result<()> {
        let usart1 = self.0;
        let sr = usart1.sr.read();

        if sr.ore().is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.ne().is_set() {
            Err(nb::Error::Other(Error::Noise))
        } else if sr.fe().is_set() {
            Err(nb::Error::Other(Error::Framing))
        } else if sr.txe().is_set() {
            // NOTE(write_volatile) see NOTE in the `read` method
            unsafe {
                ptr::write_volatile(&usart1.dr as *const _ as *mut u8, byte)
            }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<'a> Serial<'a, USART1> {
    /// Starts a DMA transfer to receive serial data into a `buffer`
    // TODO support circular mode + half transfer interrupt as a double
    // buffering mode
    pub fn read_exact<B>(
        &self,
        dma1: &DMA1,
        buffer: Ref<Buffer<B, Dma1Channel5>>,
    ) -> ::core::result::Result<(), dma::Error>
    where
        B: AsMut<[u8]>,
    {
        let usart1 = self.0;

        if dma1.ccr5.read().en().is_set() {
            return Err(dma::Error::InUse);
        }

        let buffer = buffer.lock_mut().as_mut();

        dma1.cndtr5.write(|w| unsafe {
            w.ndt().bits(u16(buffer.len()).unwrap())
        });
        dma1.cpar5.write(|w| unsafe {
            w.bits(&usart1.dr as *const _ as u32)
        });
        dma1.cmar5.write(
            |w| unsafe { w.bits(buffer.as_ptr() as u32) },
        );
        dma1.ccr5.modify(|_, w| w.en().set());

        Ok(())
    }

    /// Starts a DMA transfer to send `buffer` through this serial port
    pub fn write_all<B>(
        &self,
        dma1: &DMA1,
        buffer: Ref<Buffer<B, Dma1Channel4>>,
    ) -> ::core::result::Result<(), dma::Error>
    where
        B: AsRef<[u8]>,
    {
        let usart1 = self.0;

        if dma1.ccr4.read().en().is_set() {
            return Err(dma::Error::InUse);
        }

        let buffer = buffer.lock().as_ref();

        dma1.cndtr4.write(|w| unsafe {
            w.ndt().bits(u16(buffer.len()).unwrap())
        });
        dma1.cpar4.write(|w| unsafe {
            w.bits(&usart1.dr as *const _ as u32)
        });
        dma1.cmar4.write(
            |w| unsafe { w.bits(buffer.as_ptr() as u32) },
        );
        dma1.ccr4.modify(|_, w| w.en().set());

        Ok(())
    }
}

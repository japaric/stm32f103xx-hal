use core::marker::Unsize;
use core::ptr;

use cast::u16;
use hal;
use nb;
use stm32f103xx::{DMA1, USART1};

use afio::MAPR;
use dma::{CircTransfer, D1C4, D1C5, Read, Static, Transfer, Write};
use gpio::{AltPush, InputFloating, PA10, PA9, PB6, PB7};
use rcc::{Clocks, ENR};
use time::Bps;

#[derive(Debug)]
pub enum Error {
    /// De-synchronization, excessive noise or a break character detected
    Framing,
    /// Noise detected in the received frame
    Noise,
    /// RX buffer overrun
    Overrun,
    #[doc(hidden)] _Extensible,
}

/// Interrupt event
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// RX buffer Not Empty (new data available)
    Rxne,
    /// Transmission Complete
    Tc,
    /// TX buffer Empty (more data can be send)
    Txe,
}

pub struct Serial {
    usart: USART1,
}

pub enum Pins {
    NoRemap(PA9<AltPush>, PA10<InputFloating>),
    Remapped(PB6<AltPush>, PB7<InputFloating>),
}

impl From<(PA9<AltPush>, PA10<InputFloating>)> for Pins {
    fn from((tx, rx): (PA9<AltPush>, PA10<InputFloating>)) -> Pins {
        Pins::NoRemap(tx, rx)
    }
}

impl From<(PB6<AltPush>, PB7<InputFloating>)> for Pins {
    fn from((tx, rx): (PB6<AltPush>, PB7<InputFloating>)) -> Pins {
        Pins::Remapped(tx, rx)
    }
}

impl Serial {
    pub fn new<P>(
        usart: USART1,
        pins: P,
        bps: Bps,
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Serial
    where
        P: Into<Pins>,
    {
        enr.apb2().modify(|_, w| w.usart1en().enabled());

        match pins.into() {
            Pins::NoRemap(..) => mapr.mapr().modify(|_, w| w.usart1_remap().clear_bit()),
            Pins::Remapped(..) => mapr.mapr().modify(|_, w| w.usart1_remap().set_bit()),
        }

        // 1 stop bit
        usart.cr2.write(|w| unsafe { w.stop().bits(0b00) });

        let brr = clocks.pclk2().0 / bps.0;

        assert!(brr > 16, "impossible baud rate");

        usart.brr.write(|w| unsafe { w.bits(brr) });

        // disable hardware flow control
        // enable DMA TX and RX transfers
        usart.cr3.write(|w| {
            w.rtse()
                .clear_bit()
                .ctse()
                .clear_bit()
                .dmat()
                .set_bit()
                .dmar()
                .set_bit()
        });

        // enable TX, RX
        // disable parity checking
        usart.cr1.write(|w| {
            w.ue()
                .set_bit()
                .re()
                .set_bit()
                .te()
                .set_bit()
                .m()
                .clear_bit()
                .pce()
                .clear_bit()
                .rxneie()
                .clear_bit()
        });

        Serial { usart }
    }

    pub fn listen(&mut self, event: Event) {
        match event {
            Event::Rxne => self.usart.cr1.modify(|_, w| w.rxneie().set_bit()),
            Event::Tc => self.usart.cr1.modify(|_, w| w.tcie().set_bit()),
            Event::Txe => self.usart.cr1.modify(|_, w| w.txeie().set_bit()),
        }
    }

    pub fn unlisten(&mut self, event: Event) {
        match event {
            Event::Rxne => self.usart.cr1.modify(|_, w| w.rxneie().clear_bit()),
            Event::Tc => self.usart.cr1.modify(|_, w| w.tcie().clear_bit()),
            Event::Txe => self.usart.cr1.modify(|_, w| w.txeie().clear_bit()),
        }
    }

    pub fn split(self) -> (Tx, Rx) {
        (Tx { _0: () }, Rx { _0: () })
    }

    pub fn unwrap(self) -> USART1 {
        self.usart
    }
}

pub struct Rx {
    _0: (),
}

impl Rx {
    pub fn read_exact<B>(self, chan: D1C5, bytes: Static<B>) -> Transfer<Write, D1C5, B, Rx>
    where
        B: Unsize<[u8]>,
    {
        let dma = unsafe { &*DMA1::ptr() };

        // This is a sanity check. Due to move semantics the channel is *never* in use at this point
        debug_assert!(dma.ccr5.read().en().is_disabled());

        {
            let slice: &mut [u8] = bytes;

            dma.cndtr5
                .write(|w| unsafe { w.ndt().bits(u16(slice.len()).unwrap()) });

            dma.cmar5
                .write(|w| unsafe { w.bits(slice.as_ptr() as u32) });
        }

        dma.cpar5.write(|w| unsafe {
            w.bits(&(*USART1::ptr()).dr as *const _ as u32)
        });

        // MEM2MEM: memory to memory mode disabled
        // MSIZE: memory is 8 bit wide
        // PSIZE: peripheral is 8 bit wide
        // MINC: increment the memory pointer
        // PINC: don't increment the peripheral pointer
        // CIRC: circular mode disabled
        // DIR: read from peripheral
        // TCIE: enable transfer complete interrupt
        // EN: start the transfer
        dma.ccr5.write(|w| unsafe {
            w.mem2mem()
                .clear_bit()
                .msize()
                .bits(0b00)
                .psize()
                .bits(0b00)
                .minc()
                .set_bit()
                .pinc()
                .clear_bit()
                .circ()
                .clear_bit()
                .dir()
                .clear_bit()
                .tcie()
                .set_bit()
                .en()
                .set_bit()
        });

        Transfer::new(chan, bytes, self)
    }

    pub fn circ_read<B>(self, buffer: Static<[B; 2]>, chan: D1C5) -> CircTransfer<B, D1C5>
    where
        B: Unsize<[u8]>,
    {
        let dma = unsafe { &*DMA1::ptr() };

        // This is a sanity check. Due to move semantics the channel is *never* in use at this point
        debug_assert!(dma.ccr5.read().en().is_disabled());

        {
            let slice: &mut [u8] = &mut buffer[0];

            dma.cndtr5
                .write(|w| unsafe { w.ndt().bits(u16(slice.len() * 2).unwrap()) });

            dma.cmar5
                .write(|w| unsafe { w.bits(slice.as_ptr() as u32) });
        }

        dma.cpar5.write(|w| unsafe {
            w.bits(&(*USART1::ptr()).dr as *const _ as u32)
        });

        // MEM2MEM: memory to memory mode disabled
        // MSIZE: memory is 8 bit wide
        // PSIZE: peripheral is 8 bit wide
        // MINC: increment the memory pointer
        // PINC: don't increment the peripheral pointer
        // CIRC: circular mode enabled
        // DIR: read from peripheral
        // TCIE: enable transfer complete interrupt
        // EN: start the transfer
        dma.ccr5.write(|w| unsafe {
            w.mem2mem()
                .clear_bit()
                .msize()
                .bits(0b00)
                .psize()
                .bits(0b00)
                .minc()
                .set_bit()
                .pinc()
                .clear_bit()
                .circ()
                .set_bit()
                .dir()
                .clear_bit()
                .tcie()
                .set_bit()
                .en()
                .set_bit()
        });

        unsafe { CircTransfer::new(buffer, chan) }
    }
}

impl hal::serial::Read<u8> for Rx {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        let usart = unsafe { &*USART1::ptr() };
        let sr = usart.sr.read();

        if sr.ore().bit_is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.ne().bit_is_set() {
            Err(nb::Error::Other(Error::Noise))
        } else if sr.fe().bit_is_set() {
            Err(nb::Error::Other(Error::Framing))
        } else if sr.rxne().bit_is_set() {
            // NOTE(read_volatile) the register is 9 bits big but we'll only
            // work with the first 8 bits
            Ok(unsafe {
                ptr::read_volatile(&usart.dr as *const _ as *const u8)
            })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

pub struct Tx {
    _0: (),
}

impl Tx {
    pub fn write_all<B>(self, chan: D1C4, bytes: Static<B>) -> Transfer<Read, D1C4, B, Tx>
    where
        B: Unsize<[u8]>,
    {
        let dma = unsafe { &*DMA1::ptr() };

        // This is a sanity check. Due to move semantics the channel is *never* in use at this point
        debug_assert!(dma.ccr4.read().en().is_disabled());

        {
            let slice: &mut [u8] = bytes;

            dma.cndtr4
                .write(|w| unsafe { w.ndt().bits(u16(slice.len()).unwrap()) });

            dma.cmar4
                .write(|w| unsafe { w.bits(slice.as_ptr() as u32) });
        }

        dma.cpar4.write(|w| unsafe {
            w.bits(&(*USART1::ptr()).dr as *const _ as u32)
        });

        // MEM2MEM: memory to memory mode disabled
        // MSIZE: memory is 8 bit wide
        // PSIZE: peripheral is 8 bit wide
        // MINC: increment the memory pointer
        // PINC: don't increment the peripheral pointer
        // CIRC: circular mode disabled
        // DIR: read from memory
        // TCIE: enable transfer complete interrupt
        // EN: start the transfer
        dma.ccr4.write(|w| unsafe {
            w.mem2mem()
                .clear_bit()
                .msize()
                .bits(0b00)
                .psize()
                .bits(0b00)
                .minc()
                .set_bit()
                .pinc()
                .clear_bit()
                .circ()
                .clear_bit()
                .dir()
                .set_bit()
                .tcie()
                .set_bit()
                .en()
                .set_bit()
        });

        Transfer::new(chan, bytes, self)
    }
}

impl hal::serial::Write<u8> for Tx {
    type Error = Error;

    fn write(&mut self, byte: u8) -> nb::Result<(), Error> {
        let usart = unsafe { &*USART1::ptr() };
        let sr = usart.sr.read();

        if sr.ore().bit_is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.ne().bit_is_set() {
            Err(nb::Error::Other(Error::Noise))
        } else if sr.fe().bit_is_set() {
            Err(nb::Error::Other(Error::Framing))
        } else if sr.txe().bit_is_set() {
            // NOTE(write_volatile) see NOTE in the `read` method
            unsafe { ptr::write_volatile(&usart.dr as *const _ as *mut u8, byte) }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

use core::marker::PhantomData;
use core::ops;
use core::sync::atomic::{self, Ordering};

use nb;
use stm32f103xx::DMA1;

use rcc::ENR;

pub type Static<T> = &'static mut T;

#[derive(Debug)]
pub enum Error {
    /// Previous data got overwritten before it could be read because it was
    /// not accessed in a timely fashion
    Overrun,
    /// Transfer error
    Transfer,
}

pub struct Read;
pub struct Write;

/// A circular DMA transfer
pub struct CircTransfer<B, C>
where
    B: 'static,
{
    buffer: Static<[B; 2]>,
    channel: PhantomData<C>,
    /// Half that's currently being mutated
    half: Half,
}

impl<B, C> CircTransfer<B, C> {
    pub(crate) unsafe fn new(buffer: Static<[B; 2]>, _: C) -> Self {
        CircTransfer {
            buffer,
            channel: PhantomData,
            half: Half::First,
        }
    }
}

enum Half {
    First,
    Second,
}

impl<B> CircTransfer<B, D1C1> {
    pub fn read<R, F>(&mut self, f: F) -> nb::Result<R, Error>
    where
        F: FnOnce(&B) -> R,
    {
        // NOTE atomic read with no side effects
        let isr = unsafe { (*DMA1::ptr()).isr.read() };

        if isr.teif1().bit_is_set() {
            Err(nb::Error::Other(Error::Transfer))
        } else {
            match self.half {
                Half::First => if isr.tcif1().bit_is_set() {
                    Err(nb::Error::Other(Error::Overrun))
                } else if isr.htif1().bit_is_set() {
                    // NOTE atomic write to stateless register
                    unsafe { (*DMA1::ptr()).ifcr.write(|w| w.chtif1().set_bit()) }

                    atomic::compiler_fence(Ordering::SeqCst);

                    let r = f(&self.buffer[0]);

                    atomic::compiler_fence(Ordering::SeqCst);

                    let isr = unsafe { (*DMA1::ptr()).isr.read() };

                    if isr.tcif1().bit_is_set() {
                        // took too long the first half is being written again
                        Err(nb::Error::Other(Error::Overrun))
                    } else {
                        self.half = Half::Second;
                        Ok(r)
                    }
                } else {
                    Err(nb::Error::WouldBlock)
                },
                Half::Second => if isr.htif1().bit_is_set() {
                    Err(nb::Error::Other(Error::Overrun))
                } else if isr.tcif1().bit_is_set() {
                    // NOTE atomic write to stateless register
                    unsafe { (*DMA1::ptr()).ifcr.write(|w| w.ctcif1().set_bit()) }

                    atomic::compiler_fence(Ordering::SeqCst);

                    let r = f(&self.buffer[1]);

                    atomic::compiler_fence(Ordering::SeqCst);

                    let isr = unsafe { (*DMA1::ptr()).isr.read() };

                    if isr.htif1().bit_is_set() {
                        // took too long the second half is being written again
                        Err(nb::Error::Other(Error::Overrun))
                    } else {
                        self.half = Half::First;
                        Ok(r)
                    }
                } else {
                    Err(nb::Error::WouldBlock)
                },
            }
        }
    }

    pub fn resume(&mut self) {
        unsafe { (*DMA1::ptr()).ccr1.modify(|_, w| w.en().set_bit()) }
    }

    pub fn pause(&mut self) {
        unsafe { (*DMA1::ptr()).ccr1.modify(|_, w| w.en().clear_bit()) }
    }
}

impl<B> CircTransfer<B, D1C5> {
    pub fn read<R, F>(&mut self, f: F) -> nb::Result<R, Error>
    where
        F: FnOnce(&B) -> R,
    {
        // NOTE atomic read with no side effects
        let isr = unsafe { (*DMA1::ptr()).isr.read() };

        if isr.teif5().bit_is_set() {
            Err(nb::Error::Other(Error::Transfer))
        } else {
            match self.half {
                Half::First => if isr.tcif5().bit_is_set() {
                    Err(nb::Error::Other(Error::Overrun))
                } else if isr.htif5().bit_is_set() {
                    // NOTE atomic write to stateless register
                    unsafe { (*DMA1::ptr()).ifcr.write(|w| w.chtif5().set_bit()) }

                    atomic::compiler_fence(Ordering::SeqCst);

                    let r = f(&self.buffer[0]);

                    atomic::compiler_fence(Ordering::SeqCst);

                    let isr = unsafe { (*DMA1::ptr()).isr.read() };

                    if isr.tcif5().bit_is_set() {
                        // took too long the first half is being written again
                        Err(nb::Error::Other(Error::Overrun))
                    } else {
                        self.half = Half::Second;
                        Ok(r)
                    }
                } else {
                    Err(nb::Error::WouldBlock)
                },
                Half::Second => if isr.htif5().bit_is_set() {
                    Err(nb::Error::Other(Error::Overrun))
                } else if isr.tcif5().bit_is_set() {
                    // NOTE atomic write to stateless register
                    unsafe { (*DMA1::ptr()).ifcr.write(|w| w.ctcif5().set_bit()) }

                    atomic::compiler_fence(Ordering::SeqCst);

                    let r = f(&self.buffer[1]);

                    atomic::compiler_fence(Ordering::SeqCst);

                    let isr = unsafe { (*DMA1::ptr()).isr.read() };

                    if isr.htif5().bit_is_set() {
                        // took too long the second half is being written again
                        Err(nb::Error::Other(Error::Overrun))
                    } else {
                        self.half = Half::First;
                        Ok(r)
                    }
                } else {
                    Err(nb::Error::WouldBlock)
                },
            }
        }
    }

    pub fn resume(&mut self) {
        unsafe { (*DMA1::ptr()).ccr1.modify(|_, w| w.en().set_bit()) }
    }

    pub fn pause(&mut self) {
        unsafe { (*DMA1::ptr()).ccr1.modify(|_, w| w.en().clear_bit()) }
    }
}


/// An on-going DMA transfer
// This is bit like a `Future` minus the panicking `poll` method
pub struct Transfer<M, C, B, P>
where
    B: 'static,
{
    _mode: PhantomData<M>,
    buffer: Static<B>,
    channel: C,
    payload: P,
}

impl<M, C, B, P> Transfer<M, C, B, P> {
    pub(crate) fn new(channel: C, buffer: Static<B>, payload: P) -> Self {
        Transfer {
            _mode: PhantomData,
            buffer,
            channel,
            payload,
        }
    }
}

impl<C, B, P> ops::Deref for Transfer<Read, C, B, P> {
    type Target = B;

    fn deref(&self) -> &B {
        self.buffer
    }
}

macro_rules! channels {
    ($DMA:ident, $dmaen:ident, {
        $($CHAN:ident: ($ccr:ident, $teif:ident, $tcif:ident, $ctcif:ident),)+
    }) => {
        impl DmaExt for $DMA {
            type Channels = ((), $($CHAN),+);

            fn split(self, enr: &mut ENR) -> ((), $($CHAN),+) {
                enr.ahb().modify(|_, w| w.$dmaen().enabled());

                ((), $($CHAN { _0: () }),+)
            }
        }

        $(
            pub struct $CHAN { _0: () }

            impl<M, B, P> Transfer<M, $CHAN, B, P> {
                pub fn is_done(&self) -> Result<bool, Error> {
                    let dma = unsafe { &*$DMA::ptr() };
                    let isr = dma.isr.read();

                    if isr.$teif().bit_is_set() {
                        return Err(Error::Transfer);
                    } else {
                        return Ok(isr.$tcif().bit_is_set());
                    }
                }

                pub fn wait(self) -> Result<($CHAN, Static<B>, P), Error> {
                    while !self.is_done()? {}

                    atomic::compiler_fence(Ordering::SeqCst);

                    let dma = unsafe { &*$DMA::ptr() };

                    // clear the "transfer complete" flag
                    dma.ifcr.write(|w| w.$ctcif().set_bit());

                    // disable this channel
                    // XXX maybe not required?
                    dma.$ccr.modify(|_, w| w.en().clear_bit());

                    Ok((self.channel, self.buffer, self.payload))
                }
            }
        )+
    }
}

channels!(DMA1, dma1en, {
    D1C1: (ccr1, teif1, tcif1, ctcif1),
    D1C2: (ccr2, teif2, tcif2, ctcif2),
    D1C3: (ccr3, teif3, tcif3, ctcif3),
    D1C4: (ccr4, teif4, tcif4, ctcif4),
    D1C5: (ccr5, teif5, tcif5, ctcif5),
    D1C6: (ccr6, teif6, tcif6, ctcif6),
    D1C7: (ccr7, teif7, tcif7, ctcif7),
});

pub trait DmaExt {
    type Channels;

    fn split(self, enr: &mut ENR) -> Self::Channels;
}

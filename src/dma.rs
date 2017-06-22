//! Direct Memory Access (DMA)

use core::cell::{Cell, UnsafeCell};
use core::marker::{PhantomData, Unsize};
use core::{ops, slice};

use nb;
use stm32f103xx::DMA1;
use volatile_register::RO;

/// DMA error
#[derive(Debug)]
pub enum Error {
    /// DMA channel in use
    InUse,
    /// Previous data got overwritten before it could be read because it was
    /// not accessed in a timely fashion
    Overrun,
    /// Transfer error
    Transfer,
}

/// Channel 1 of DMA1
pub struct Dma1Channel1 {
    _0: (),
}

/// Channel 2 of DMA1
pub struct Dma1Channel2 {
    _0: (),
}

/// Channel 4 of DMA1
pub struct Dma1Channel4 {
    _0: (),
}

/// Channel 5 of DMA1
pub struct Dma1Channel5 {
    _0: (),
}

/// Buffer to be used with a certain DMA `CHANNEL`
pub struct Buffer<T, CHANNEL> {
    _marker: PhantomData<CHANNEL>,
    data: UnsafeCell<T>,
    flag: Cell<BorrowFlag>,
    status: Cell<Status>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Locked,
    MutLocked,
    Unlocked,
}

type BorrowFlag = usize;

const UNUSED: BorrowFlag = 0;
const WRITING: BorrowFlag = !0;

/// Wraps a borrowed reference to a value in a `Buffer`
pub struct Ref<'a, T>
where
    T: 'a,
{
    data: &'a T,
    flag: &'a Cell<BorrowFlag>,
}

impl<'a, T> ops::Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        self.flag.set(self.flag.get() - 1);
    }
}

/// A wrapper type for a mutably borrowed value from a `Buffer``
pub struct RefMut<'a, T>
where
    T: 'a,
{
    data: &'a mut T,
    flag: &'a Cell<BorrowFlag>,
}

impl<'a, T> ops::Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        self.flag.set(UNUSED);
    }
}

impl<T, CHANNEL> Buffer<T, CHANNEL> {
    /// Creates a new buffer
    pub const fn new(data: T) -> Self {
        Buffer {
            _marker: PhantomData,
            data: UnsafeCell::new(data),
            flag: Cell::new(0),
            status: Cell::new(Status::Unlocked),
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    pub fn borrow(&self) -> Ref<T> {
        assert_ne!(self.flag.get(), WRITING);

        self.flag.set(self.flag.get() + 1);

        Ref {
            data: unsafe { &*self.data.get() },
            flag: &self.flag,
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value
    /// cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn borrow_mut(&self) -> RefMut<T> {
        assert_eq!(self.flag.get(), UNUSED);

        self.flag.set(WRITING);

        RefMut {
            data: unsafe { &mut *self.data.get() },
            flag: &self.flag,
        }
    }

    pub(crate) fn lock(&self) -> &T {
        assert_eq!(self.status.get(), Status::Unlocked);
        assert_ne!(self.flag.get(), WRITING);

        self.flag.set(self.flag.get() + 1);
        self.status.set(Status::Locked);

        unsafe { &*self.data.get() }
    }

    pub(crate) fn lock_mut(&self) -> &mut T {
        assert_eq!(self.status.get(), Status::Unlocked);
        assert_eq!(self.flag.get(), UNUSED);

        self.flag.set(WRITING);
        self.status.set(Status::MutLocked);

        unsafe { &mut *self.data.get() }
    }

    unsafe fn unlock(&self, status: Status) {
        match status {
            Status::Locked => self.flag.set(self.flag.get() - 1),
            Status::MutLocked => self.flag.set(UNUSED),
            _ => { /* unreachable!() */ }
        }

        self.status.set(Status::Unlocked);
    }
}

// FIXME these `release` methods probably want some of sort of barrier
impl<T> Buffer<T, Dma1Channel2> {
    /// Waits until the DMA releases this buffer
    pub fn release(&self, dma1: &DMA1) -> nb::Result<(), Error> {
        let status = self.status.get();

        if status == Status::Unlocked {
            return Ok(());
        }

        if dma1.isr.read().teif2().is_set() {
            Err(nb::Error::Other(Error::Transfer))
        } else if dma1.isr.read().tcif2().is_set() {
            unsafe { self.unlock(status) }
            dma1.ifcr.write(|w| w.ctcif2().set());
            dma1.ccr2.modify(|_, w| w.en().clear());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T> Buffer<T, Dma1Channel4> {
    /// Waits until the DMA releases this buffer
    pub fn release(&self, dma1: &DMA1) -> nb::Result<(), Error> {
        let status = self.status.get();

        if status == Status::Unlocked {
            return Ok(());
        }

        if dma1.isr.read().teif4().is_set() {
            Err(nb::Error::Other(Error::Transfer))
        } else if dma1.isr.read().tcif4().is_set() {
            unsafe { self.unlock(status) }
            dma1.ifcr.write(|w| w.ctcif4().set());
            dma1.ccr4.modify(|_, w| w.en().clear());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T> Buffer<T, Dma1Channel5> {
    /// Waits until the DMA releases this buffer
    pub fn release(&self, dma1: &DMA1) -> nb::Result<(), Error> {
        let status = self.status.get();

        if status == Status::Unlocked {
            return Ok(());
        }

        if dma1.isr.read().teif5().is_set() {
            Err(nb::Error::Other(Error::Transfer))
        } else if dma1.isr.read().tcif5().is_set() {
            unsafe { self.unlock(status) }
            dma1.ifcr.write(|w| w.ctcif5().set());
            dma1.ccr5.modify(|_, w| w.en().clear());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// A circular buffer associated to a DMA `CHANNEL`
pub struct CircBuffer<T, B, CHANNEL>
where
    B: Unsize<[T]>,
{
    _marker: PhantomData<CHANNEL>,
    _t: PhantomData<[T]>,
    buffer: UnsafeCell<[B; 2]>,
    status: Cell<CircStatus>,
}

impl<T, B, CHANNEL> CircBuffer<T, B, CHANNEL>
where
    B: Unsize<[T]>,
{
    pub(crate) fn lock(&self) -> &[B; 2] {
        assert_eq!(self.status.get(), CircStatus::Free);

        self.status.set(CircStatus::MutatingFirstHalf);

        unsafe { &*self.buffer.get() }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CircStatus {
    /// Not in use by the DMA
    Free,
    /// The DMA is mutating the first half of the buffer
    MutatingFirstHalf,
    /// The DMA is mutating the second half of the buffer
    MutatingSecondHalf,
}

impl<T, B> CircBuffer<T, B, Dma1Channel1>
where
    B: Unsize<[T]>,
    T: Atomic,
{
    /// Constructs a circular buffer from two halves
    pub const fn new(buffer: [B; 2]) -> Self {
        CircBuffer {
            _t: PhantomData,
            _marker: PhantomData,
            buffer: UnsafeCell::new(buffer),
            status: Cell::new(CircStatus::Free),
        }
    }

    /// Yields read access to the half of the circular buffer that's not
    /// currently being mutated by the DMA
    pub fn read(&self, dma1: &DMA1) -> nb::Result<&[RO<T>], Error> {
        let status = self.status.get();

        assert_ne!(status, CircStatus::Free);

        let isr = dma1.isr.read();

        if isr.teif1().is_set() {
            Err(nb::Error::Other(Error::Transfer))
        } else {
            match status {
                CircStatus::MutatingFirstHalf => {
                    if isr.tcif1().is_set() {
                        Err(nb::Error::Other(Error::Overrun))
                    } else if isr.htif1().is_set() {
                        dma1.ifcr.write(|w| w.chtif1().set());

                        self.status.set(CircStatus::MutatingSecondHalf);

                        unsafe {
                            let half: &[T] = &(*self.buffer.get())[0];
                            Ok(slice::from_raw_parts(
                                half.as_ptr() as *const _,
                                half.len(),
                            ))
                        }
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
                CircStatus::MutatingSecondHalf => {
                    if isr.htif1().is_set() {
                        Err(nb::Error::Other(Error::Overrun))
                    } else if isr.tcif1().is_set() {
                        dma1.ifcr.write(|w| w.ctcif1().set());

                        self.status.set(CircStatus::MutatingFirstHalf);

                        unsafe {
                            let half: &[T] = &(*self.buffer.get())[1];
                            Ok(slice::from_raw_parts(
                                half.as_ptr() as *const _,
                                half.len(),
                            ))
                        }
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Values that can be atomically read
pub trait Atomic: Copy {}

impl Atomic for u8 {}
impl Atomic for u16 {}
impl Atomic for u32 {}

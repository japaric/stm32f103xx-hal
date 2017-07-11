//! Timer

use core::any::{Any, TypeId};
use core::ops::Deref;

use cast::{u16, u32};
use hal;
use nb::{self, Error};
use stm32f103xx::{GPIOA, GPIOB, RCC, TIM1, TIM2, TIM3, TIM4, gpioa, tim2};

/// Channel associated to a timer
#[derive(Clone, Copy, Debug)]
pub enum Channel {
    /// TxC1
    _1,
    /// TxC2
    _2,
    /// TxC3
    _3,
    /// TxC4
    _4,
}

/// IMPLEMENTATION DETAIL
pub unsafe trait TIM: Deref<Target = tim2::RegisterBlock> {
    /// IMPLEMENTATION DETAIL
    type GPIO: Deref<Target = gpioa::RegisterBlock>;
}

unsafe impl TIM for TIM2 {
    type GPIO = GPIOA;
}

unsafe impl TIM for TIM3 {
    type GPIO = GPIOA;
}

unsafe impl TIM for TIM4 {
    type GPIO = GPIOB;
}

/// `hal::Timer` implementation
pub struct Timer<'a, T>(pub &'a T)
where
    T: 'a;

impl<'a, T> Clone for Timer<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Timer<'a, T> {}

impl<'a> Timer<'a, TIM1> {
    /// Initializes the timer with a periodic timeout of `frequency` Hz
    ///
    /// NOTE After initialization, the timer will be in the paused state.
    pub fn init<P>(&self, period: P, rcc: &RCC)
    where
        P: Into<::apb2::Ticks>,
    {
        self._init(period.into(), rcc)
    }

    fn _init(&self, period: ::apb2::Ticks, rcc: &RCC) {
        let tim1 = self.0;

        // Enable TIM1
        rcc.apb2enr.modify(|_, w| w.tim1en().enabled());

        // Configure periodic update event
        self._set_timeout(period);

        // Continuous mode
        tim1.cr1.write(|w| w.opm().continuous());

        // Enable update event interrupt
        tim1.dier.modify(|_, w| w.uie().set_bit());
    }

    fn _set_timeout(&self, timeout: ::apb2::Ticks) {
        let period = timeout.0;

        let psc = u16((period - 1) / (1 << 16)).unwrap();
        self.0.psc.write(|w| w.psc().bits(psc));

        let arr = u16(period / u32(psc + 1)).unwrap();
        self.0.arr.write(|w| w.arr().bits(arr));
    }
}

impl<'a> hal::Timer for Timer<'a, TIM1> {
    type Time = ::apb2::Ticks;

    fn get_timeout(&self) -> ::apb2::Ticks {
        ::apb2::Ticks(
            u32(self.0.psc.read().psc().bits() + 1) *
                u32(self.0.arr.read().arr().bits()),
        )
    }

    fn pause(&self) {
        self.0.cr1.modify(|_, w| w.cen().disabled());
    }

    fn restart(&self) {
        self.0.cnt.write(|w| w.cnt().bits(0));
    }

    fn resume(&self) {
        self.0.cr1.modify(|_, w| w.cen().enabled());
    }

    fn set_timeout<T>(&self, timeout: T)
    where
        T: Into<::apb2::Ticks>,
    {
        self._set_timeout(timeout.into())
    }

    fn wait(&self) -> nb::Result<(), !> {
        if self.0.sr.read().uif().bit_is_clear() {
            Err(Error::WouldBlock)
        } else {
            self.0.sr.modify(|_, w| w.uif().clear_bit());
            Ok(())
        }
    }
}

impl<'a, T> Timer<'a, T>
where
    T: Any + TIM,
{
    /// Initializes the timer with a periodic timeout of `frequency` Hz
    ///
    /// NOTE After initialization, the timer will be in the paused state.
    pub fn init<P>(&self, period: P, rcc: &RCC)
    where
        P: Into<::apb1::Ticks>,
    {
        self.init_(period.into(), rcc)
    }

    fn init_(&self, timeout: ::apb1::Ticks, rcc: &RCC) {
        let tim2 = self.0;

        // Enable TIMx
        if tim2.get_type_id() == TypeId::of::<TIM2>() {
            rcc.apb1enr.modify(|_, w| w.tim2en().enabled());
        } else if tim2.get_type_id() == TypeId::of::<TIM3>() {
            rcc.apb1enr.modify(|_, w| w.tim3en().enabled());
        } else if tim2.get_type_id() == TypeId::of::<TIM4>() {
            rcc.apb1enr.modify(|_, w| w.tim4en().enabled());
        }

        // Configure periodic update event
        self._set_timeout(timeout);

        // Continuous mode
        tim2.cr1.write(|w| w.opm().continuous());

        // Enable the update event interrupt
        tim2.dier.modify(|_, w| w.uie().set_bit());
    }

    fn _set_timeout(&self, timeout: ::apb1::Ticks) {
        let period = timeout.0;

        let psc = u16((period - 1) / (1 << 16)).unwrap();
        self.0.psc.write(|w| w.psc().bits(psc));

        let arr = u16(period / u32(psc + 1)).unwrap();
        self.0.arr.write(|w| w.arr().bits(arr));
    }
}

impl<'a, T> hal::Timer for Timer<'a, T>
where
    T: Any + TIM,
{
    type Time = ::apb1::Ticks;

    fn get_timeout(&self) -> ::apb1::Ticks {
        ::apb1::Ticks(
            u32(self.0.psc.read().psc().bits() + 1) *
                u32(self.0.arr.read().arr().bits()),
        )
    }

    fn pause(&self) {
        self.0.cr1.modify(|_, w| w.cen().disabled());
    }

    fn restart(&self) {
        self.0.cnt.write(|w| w.cnt().bits(0));
    }

    fn resume(&self) {
        self.0.cr1.modify(|_, w| w.cen().enabled());
    }

    fn set_timeout<TO>(&self, timeout: TO)
    where
        TO: Into<::apb1::Ticks>,
    {
        self._set_timeout(timeout.into())
    }

    fn wait(&self) -> nb::Result<(), !> {
        if self.0.sr.read().uif().bit_is_clear() {
            Err(Error::WouldBlock)
        } else {
            self.0.sr.modify(|_, w| w.uif().clear_bit());
            Ok(())
        }
    }
}

//! Periodic timer

use core::any::{Any, TypeId};
use core::ops::Deref;

use either::Either;
use cast::{u16, u32};
use stm32f103xx::{GPIOA, GPIOB, RCC, TIM1, TIM2, TIM3, TIM4, gpioa, tim1, tim2};

use frequency;

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

/// TIM instance that can be used with the `Capture` / `Pwm` abstraction
///
/// IMPLEMENTATION DETAIL. Do not implement this trait
pub unsafe trait Tim {
    /// GPIO block associated to this TIM instance
    type GPIO: Deref<Target = gpioa::RegisterBlock>;

    /// Returns the register block of this TIM instance
    fn register_block<'s>
        (
        &'s self,
    ) -> Either<&'s tim1::RegisterBlock, &'s tim2::RegisterBlock>;
}

unsafe impl Tim for TIM1 {
    type GPIO = GPIOA;

    fn register_block(
        &self,
    ) -> Either<&tim1::RegisterBlock, &tim2::RegisterBlock> {
        Either::Left(&**self)
    }
}

unsafe impl Tim for TIM2 {
    type GPIO = GPIOA;

    fn register_block(
        &self,
    ) -> Either<&tim1::RegisterBlock, &tim2::RegisterBlock> {
        Either::Right(&**self)
    }
}

unsafe impl Tim for TIM3 {
    // FIXME should be GPIOA *and* GPIOB
    type GPIO = GPIOA;

    fn register_block(
        &self,
    ) -> Either<&tim1::RegisterBlock, &tim2::RegisterBlock> {
        Either::Right(&**self)
    }
}

unsafe impl Tim for TIM4 {
    type GPIO = GPIOB;

    fn register_block(
        &self,
    ) -> Either<&tim1::RegisterBlock, &tim2::RegisterBlock> {
        Either::Right(&**self)
    }
}

/// Specialized `Result` type
pub type Result<T> = ::core::result::Result<T, Error>;

/// An error
pub struct Error {
    _0: (),
}

/// Periodic timer
///
/// # Interrupts
///
/// - `Tim1UpTim10` - update event
#[derive(Clone, Copy)]
pub struct Timer<'a, T>(pub &'a T)
where
    T: Any + Tim;

impl<'a, T> Timer<'a, T>
where
    T: Any + Tim,
{
    /// Initializes the timer with a periodic timeout of `frequency` Hz
    ///
    /// NOTE After initialization, the timer will be in the paused state.
    pub fn init(&self, frequency: u32, rcc: &RCC) {
        let tim = self.0;

        match self.0.register_block() {
            Either::Left(tim1) => {
                rcc.apb2enr.modify(|_, w| w.tim1en().enabled());

                // Configure periodic update event
                let ratio = frequency::APB2 / frequency;
                let psc = u16((ratio - 1) / (1 << 16)).unwrap();
                tim1.psc.write(|w| w.psc().bits(psc));
                let arr = u16(ratio / u32(psc + 1)).unwrap();
                tim1.arr.write(|w| w.arr().bits(arr));

                // Continuous mode
                tim1.cr1.write(|w| w.opm().continuous());

                // Enable update event interrupt
                tim1.dier.modify(|_, w| w.uie().set());
            }
            Either::Right(tim2) => {
                // Power on TIMx
                if tim.get_type_id() == TypeId::of::<TIM2>() {
                    rcc.apb1enr.modify(|_, w| w.tim2en().enabled());
                } else if tim.get_type_id() == TypeId::of::<TIM3>() {
                    rcc.apb1enr.modify(|_, w| w.tim3en().enabled());
                } else if tim.get_type_id() == TypeId::of::<TIM4>() {
                    rcc.apb1enr.modify(|_, w| w.tim4en().enabled());
                }

                // Configure periodic update event
                let ratio = frequency::APB1 / frequency;
                let psc = u16((ratio - 1) / (1 << 16)).unwrap();
                tim2.psc.write(|w| w.psc().bits(psc));
                let arr = u16(ratio / u32(psc + 1)).unwrap();
                tim2.arr.write(|w| w.arr().bits(arr));

                // Continuous mode
                tim2.cr1.write(|w| w.opm().continuous());

                // Enable update event interrupt
                tim2.dier.modify(|_, w| w.uie().set());
            }
        }
    }

    /// Clears the update event flag
    ///
    /// Returns `Err` if no update event has occurred
    pub fn clear_update_flag(&self) -> Result<()> {
        match self.0.register_block() {
            Either::Left(tim1) => {
                if tim1.sr.read().uif().is_clear() {
                    Err(Error { _0: () })
                } else {
                    tim1.sr.modify(|_, w| w.uif().clear());
                    Ok(())
                }
            }
            Either::Right(tim2) => {
                if tim2.sr.read().uif().is_clear() {
                    Err(Error { _0: () })
                } else {
                    tim2.sr.modify(|_, w| w.uif().clear());
                    Ok(())
                }
            }
        }
    }

    /// Pauses the timer
    pub fn pause(&self) {
        match self.0.register_block() {
            Either::Left(tim1) => {
                tim1.cr1.modify(|_, w| w.cen().disabled());
            }
            Either::Right(tim2) => {
                tim2.cr1.modify(|_, w| w.cen().disabled());
            }
        }
    }

    /// Resumes the timer count
    pub fn resume(&self) {
        match self.0.register_block() {
            Either::Left(tim1) => {
                tim1.cr1.modify(|_, w| w.cen().enabled());
            }
            Either::Right(tim2) => {
                tim2.cr1.modify(|_, w| w.cen().enabled());
            }
        }
    }
}

//! Periodic timer

use core::ops::Deref;

use either::Either;
use cast::{u16, u32};
use stm32f103xx::{GPIOA, GPIOB, RCC, TIM1, TIM2, TIM4, gpioa, tim1, tim2};

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

/// TIM instance that can be used with the `Pwm` abstraction
///
/// IMPLEMENTATION DETAIL. Do not implement this trait
pub unsafe trait Tim {
    /// GPIO block associated to this TIM instance
    type Gpio: Deref<Target = gpioa::RegisterBlock>;

    /// Returns the register block of this TIM instance
    fn register_block<'s>
        (
        &'s self,
    ) -> Either<&'s tim1::RegisterBlock, &'s tim2::RegisterBlock>;
}

unsafe impl Tim for TIM1 {
    type Gpio = GPIOA;

    fn register_block(
        &self,
    ) -> Either<&tim1::RegisterBlock, &tim2::RegisterBlock> {
        Either::Left(&**self)
    }
}

unsafe impl Tim for TIM2 {
    type Gpio = GPIOA;

    fn register_block(
        &self,
    ) -> Either<&tim1::RegisterBlock, &tim2::RegisterBlock> {
        Either::Right(&**self)
    }
}

// TODO
// impl Tim for TIM3 {
//     type Gpio = GPIOA; // *and* GPIOB
// }

unsafe impl Tim for TIM4 {
    type Gpio = GPIOB;

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
pub struct Timer<'a>(pub &'a TIM1);

impl<'a> Timer<'a> {
    /// Initializes the timer with a periodic timeout of `frequency` Hz
    ///
    /// NOTE After initialization, the timer will be in the paused state.
    pub fn init(&self, frequency: u32, rcc: &RCC) {
        let tim1 = self.0;

        /// Power on TIM1
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

    /// Clears the update event flag
    ///
    /// Returns `Err` if no update event has occurred
    pub fn clear_update_flag(&self) -> Result<()> {
        let tim1 = self.0;

        if tim1.sr.read().uif().is_clear() {
            Err(Error { _0: () })
        } else {
            self.0.sr.modify(|_, w| w.uif().clear());
            Ok(())
        }
    }

    /// Pauses the timer
    pub fn pause(&self) {
        self.0.cr1.modify(|_, w| w.cen().disabled());
    }

    /// Resumes the timer count
    pub fn resume(&self) {
        self.0.cr1.modify(|_, w| w.cen().enabled());
    }
}

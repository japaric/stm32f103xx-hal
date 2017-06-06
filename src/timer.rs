//! Periodic timer

use cast::{u16, u32};
use stm32f103xx::{RCC, TIM1};

use frequency;

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

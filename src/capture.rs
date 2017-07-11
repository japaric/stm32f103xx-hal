//! Input capture interface
//!
//! You can use the `Capture` interface with these TIM instances:
//!
//! # TIM1
//!
//! - CH1 = PA8 (5V tolerant)
//! - CH2 = PA9 (5V tolerant)
//! - CH3 = PA10 (5V tolerant)
//! - CH4 = PA11 (5V tolerant)
//!
//! # TIM2
//!
//! - CH1 = PA0
//! - CH2 = PA1
//! - CH3 = PA2
//! - CH4 = PA3
//!
//! # TIM3
//!
//! - CH1 = PA6
//! - CH2 = PA7
// - CH3 = PB0
// - CH4 = PB1
//!
//! **WARNING** Do not use channels 3 and 4 with the `Capture.capture` API or
//! you'll get junk values.
//!
//! # TIM4
//!
//! - CH1 = PB6 (5V tolerant)
//! - CH2 = PB7 (5V tolerant)
//! - CH3 = PB8 (5V tolerant)
//! - CH4 = PB9 (5V tolerant)

use core::any::{Any, TypeId};
use core::u16;

use cast::{u16, u32};
use hal;
use nb;
use stm32f103xx::{AFIO, GPIOA, RCC, TIM1, TIM2, TIM3, TIM4};

use timer::{Channel, TIM};

/// Input / capture error
#[derive(Debug)]
pub enum Error {
    /// Previous capture value was overwritten
    Overcapture,
    #[doc(hidden)]
    _Extensible,
}

/// Interrupt event
pub enum Event {
    /// Capture on channel 1
    Capture1,
    /// Capture on channel 2
    Capture2,
    /// Capture on channel 3
    Capture3,
    /// Capture on channel 4
    Capture4,
}

/// Input capture interface
pub struct Capture<'a, T>(pub &'a T)
where
    T: 'a;

impl<'a, T> Clone for Capture<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Capture<'a, T> {}

impl<'a> Capture<'a, TIM1> {
    /// Initializes the input capture interface
    ///
    /// `resolution` is the resolution of the capture timer
    pub fn init<R>(&self, resolution: R, afio: &AFIO, gpioa: &GPIOA, rcc: &RCC)
    where
        R: Into<::apb2::Ticks>,
    {
        self._init(resolution.into(), afio, gpioa, rcc)
    }

    fn _init(
        &self,
        resolution: ::apb2::Ticks,
        afio: &AFIO,
        gpioa: &GPIOA,
        rcc: &RCC,
    ) {
        let tim1 = self.0;

        // enable AFIO, TIM1 and GPIOA
        rcc.apb2enr.modify(|_, w| {
            w.tim1en().enabled().afioen().enabled().iopaen().enabled()
        });

        // don't remap TIM1 pins
        afio.mapr
            .modify(|_, w| unsafe { w.tim1_remap().bits(0b00) });

        // CH1 = PA8 = floating input
        // CH2 = PA9 = floating input
        // CH3 = PA10 = floating input
        // CH4 = PA11 = floating input
        gpioa.crh.modify(|_, w| {
            w.mode8()
                .input()
                .cnf8()
                .bits(0b01)
                .mode9()
                .input()
                .cnf9()
                .bits(0b01)
                .mode10()
                .input()
                .cnf10()
                .bits(0b01)
                .mode11()
                .input()
                .cnf11()
                .bits(0b01)
        });

        // configure CC{1,2,3,4} as input and wire it to TI{1,2,3,4}
        // apply the heaviest filter
        tim1.ccmr1_output.write(|w| unsafe {
            w.bits((0b1111 << 12) | (0b01 << 8) | (0b1111 << 4) | (0b01 << 0))
        });
        tim1.ccmr2_output.write(|w| unsafe {
            w.bits((0b1111 << 12) | (0b01 << 8) | (0b1111 << 4) | (0b01) << 0)
        });

        // enable capture on rising edge
        tim1.ccer.modify(|_, w| {
            w.cc1p()
                .clear_bit()
                .cc2p()
                .clear_bit()
                .cc3p()
                .clear_bit()
                .cc4p()
                .clear_bit()
        });

        self._set_resolution(resolution);

        tim1.arr.write(|w| w.arr().bits(u16::MAX));

        // configure timer as a continuous upcounter and start
        tim1.cr1
            .write(|w| w.dir().up().opm().continuous().cen().enabled());
    }

    /// Starts listening for an interrupt `event`
    pub fn listen(&self, event: Event) {
        let tim1 = self.0;

        match event {
            Event::Capture1 => tim1.dier.modify(|_, w| w.cc1ie().set_bit()),
            Event::Capture2 => tim1.dier.modify(|_, w| w.cc2ie().set_bit()),
            Event::Capture3 => tim1.dier.modify(|_, w| w.cc3ie().set_bit()),
            Event::Capture4 => tim1.dier.modify(|_, w| w.cc4ie().set_bit()),
        }
    }

    /// Stops listening for an interrupt `event`
    pub fn unlisten(&self, event: Event) {
        let tim1 = self.0;

        match event {
            Event::Capture1 => tim1.dier.modify(|_, w| w.cc1ie().clear_bit()),
            Event::Capture2 => tim1.dier.modify(|_, w| w.cc2ie().clear_bit()),
            Event::Capture3 => tim1.dier.modify(|_, w| w.cc3ie().clear_bit()),
            Event::Capture4 => tim1.dier.modify(|_, w| w.cc4ie().clear_bit()),
        }
    }

    fn _set_resolution(&self, resolution: ::apb2::Ticks) {
        let psc = u16(
            resolution.0.checked_sub(1).expect("impossible resolution"),
        ).unwrap();

        self.0.psc.write(|w| w.psc().bits(psc));
    }
}

impl<'a> hal::Capture for Capture<'a, TIM1> {
    type Capture = u16;
    type Channel = Channel;
    type Error = Error;
    type Time = ::apb2::Ticks;

    fn capture(&self, channel: Channel) -> nb::Result<u16, Error> {
        let tim1 = self.0;
        let sr = tim1.sr.read();

        match channel {
            Channel::_1 => {
                if sr.cc1of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc1if().bit_is_set() {
                    Ok(tim1.ccr1.read().ccr1().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
            Channel::_2 => {
                if sr.cc2of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc2if().bit_is_set() {
                    Ok(tim1.ccr2.read().ccr2().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
            Channel::_3 => {
                if sr.cc3of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc3if().bit_is_set() {
                    Ok(tim1.ccr3.read().ccr3().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
            Channel::_4 => {
                if sr.cc4of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc4if().bit_is_set() {
                    Ok(tim1.ccr4.read().ccr4().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
        }
    }

    fn disable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().clear_bit()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().clear_bit()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().clear_bit()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().clear_bit()),
        }
    }

    fn enable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().set_bit()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().set_bit()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().set_bit()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().set_bit()),
        }
    }

    fn get_resolution(&self) -> ::apb2::Ticks {
        ::apb2::Ticks(u32(self.0.psc.read().psc().bits()))
    }

    fn set_resolution<R>(&self, resolution: R)
    where
        R: Into<::apb2::Ticks>,
    {
        self._set_resolution(resolution.into())
    }
}

impl<'a, T> Capture<'a, T>
where
    T: Any + TIM,
{
    /// Initializes the input capture interface
    ///
    /// `resolution` is the resolution of the capture timer
    pub fn init<R>(&self, resolution: R, afio: &AFIO, gpio: &T::GPIO, rcc: &RCC)
    where
        R: Into<::apb1::Ticks>,
    {
        self._init(resolution.into(), afio, gpio, rcc)
    }

    fn _init(
        &self,
        resolution: ::apb1::Ticks,
        afio: &AFIO,
        gpio: &T::GPIO,
        rcc: &RCC,
    ) {
        let tim2 = self.0;

        // enable AFIO, GPIOx and TIMx
        if tim2.get_type_id() == TypeId::of::<TIM2>() {
            rcc.apb1enr.modify(|_, w| w.tim2en().enabled());
        } else if tim2.get_type_id() == TypeId::of::<TIM3>() {
            rcc.apb1enr.modify(|_, w| w.tim3en().enabled());
        } else if tim2.get_type_id() == TypeId::of::<TIM4>() {
            rcc.apb1enr.modify(|_, w| w.tim4en().enabled());
        }

        // enable AFIO, GPIOx and TIMx
        rcc.apb2enr.modify(|_, w| {
            if tim2.get_type_id() == TypeId::of::<TIM2>() {
                w.iopaen().enabled()
            } else if tim2.get_type_id() == TypeId::of::<TIM3>() {
                w.iopaen().enabled()
            // TODO
            // .iopben().enabled()
            } else if tim2.get_type_id() == TypeId::of::<TIM4>() {
                w.iopben().enabled()
            } else {
                unreachable!()
            }.afioen()
                .enabled()
        });

        // don't remap TIM pins
        if tim2.get_type_id() == TypeId::of::<TIM2>() {
            afio.mapr
                .modify(|_, w| unsafe { w.tim2_remap().bits(0b00) });

            // CH1 = PA0 = floating input
            // CH2 = PA1 = floating input
            // CH3 = PA2 = floating input
            // CH4 = PA3 = floating input
            gpio.crl.modify(|_, w| {
                w.mode0()
                    .input()
                    .cnf0()
                    .bits(0b01)
                    .mode1()
                    .input()
                    .cnf1()
                    .bits(0b01)
                    .mode2()
                    .input()
                    .cnf2()
                    .bits(0b01)
                    .mode3()
                    .input()
                    .cnf3()
                    .bits(0b01)
            });
        } else if tim2.get_type_id() == TypeId::of::<TIM3>() {
            afio.mapr
                .modify(|_, w| unsafe { w.tim3_remap().bits(0b00) });

            // CH1 = PA6 = floating input
            // CH2 = PA7 = floating input
            // CH3 = PB0 = floating input (TODO)
            // CH4 = PB1 = floating input (TODO)
            gpio.crl.modify(|_, w| {
                w.mode6()
                    .input()
                    .cnf6()
                    .bits(0b01)
                    .mode7()
                    .input()
                    .cnf7()
                    .bits(0b01)
            });
        } else if tim2.get_type_id() == TypeId::of::<TIM4>() {
            afio.mapr.modify(|_, w| w.tim4_remap().clear_bit());

            // CH1 = PB6 = floating input
            // CH2 = PB7 = floating input
            // CH3 = PB8 = floating input
            // CH4 = PB9 = floating input
            gpio.crl.modify(|_, w| {
                w.mode6()
                    .input()
                    .cnf6()
                    .bits(0b01)
                    .mode7()
                    .input()
                    .cnf7()
                    .bits(0b01)
            });

            gpio.crh.modify(|_, w| {
                w.mode8()
                    .input()
                    .cnf8()
                    .bits(0b01)
                    .mode9()
                    .input()
                    .cnf9()
                    .bits(0b01)
            });
        }

        // configure CC{1,2,3,4} as input and wire it to TI{1,2,3,4}
        // apply the heaviest filter
        tim2.ccmr1_output.write(|w| unsafe {
            w.bits((0b1111 << 12) | (0b01 << 8) | (0b1111 << 4) | (0b01 << 0))
        });
        if tim2.get_type_id() != TypeId::of::<TIM3>() {
            tim2.ccmr2_output.write(|w| unsafe {
                w.bits(
                    (0b1111 << 12) | (0b01 << 8) | (0b1111 << 4) | (0b01) << 0,
                )
            });
        }

        // enable capture on rising edge
        // capture pins disabled by default
        if tim2.get_type_id() == TypeId::of::<TIM3>() {
            tim2.ccer.modify(|_, w| {
                w.cc1p()
                    .clear_bit()
                    .cc1e()
                    .clear_bit()
                    .cc2p()
                    .clear_bit()
                    .cc2e()
                    .clear_bit()
            });
        } else {
            tim2.ccer.modify(|_, w| {
                w.cc1p()
                    .clear_bit()
                    .cc1e()
                    .clear_bit()
                    .cc2p()
                    .clear_bit()
                    .cc2e()
                    .clear_bit()
                    .cc3p()
                    .clear_bit()
                    .cc3e()
                    .clear_bit()
                    .cc4p()
                    .clear_bit()
                    .cc4e()
                    .clear_bit()
            });
        }

        self._set_resolution(resolution);

        tim2.arr.write(|w| w.arr().bits(u16::MAX));

        // configure timer as a continuous upcounter and start
        tim2.cr1
            .write(|w| w.dir().up().opm().continuous().cen().enabled());
    }

    /// Starts listening for an interrupt `event`
    pub fn listen(&self, event: Event) {
        let tim = self.0;

        match event {
            Event::Capture1 => tim.dier.modify(|_, w| w.cc1ie().set_bit()),
            Event::Capture2 => tim.dier.modify(|_, w| w.cc2ie().set_bit()),
            Event::Capture3 => tim.dier.modify(|_, w| w.cc3ie().set_bit()),
            Event::Capture4 => tim.dier.modify(|_, w| w.cc4ie().set_bit()),
        }
    }

    /// Stops listening for an interrupt `event`
    pub fn unlisten(&self, event: Event) {
        let tim = self.0;

        match event {
            Event::Capture1 => tim.dier.modify(|_, w| w.cc1ie().clear_bit()),
            Event::Capture2 => tim.dier.modify(|_, w| w.cc2ie().clear_bit()),
            Event::Capture3 => tim.dier.modify(|_, w| w.cc3ie().clear_bit()),
            Event::Capture4 => tim.dier.modify(|_, w| w.cc4ie().clear_bit()),
        }
    }

    fn _set_resolution(&self, resolution: ::apb1::Ticks) {
        let psc = u16(
            resolution.0.checked_sub(1).expect("impossible resolution"),
        ).unwrap();

        self.0.psc.write(|w| w.psc().bits(psc));
    }
}

impl<'a, T> hal::Capture for Capture<'a, T>
where
    T: Any + TIM,
{
    type Capture = u16;
    type Channel = Channel;
    type Error = Error;
    type Time = ::apb1::Ticks;

    fn capture(&self, channel: Channel) -> nb::Result<u16, Error> {
        let tim1 = self.0;
        let sr = tim1.sr.read();

        match channel {
            Channel::_1 => {
                if sr.cc1of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc1if().bit_is_set() {
                    Ok(tim1.ccr1.read().ccr1().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
            Channel::_2 => {
                if sr.cc2of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc2if().bit_is_set() {
                    Ok(tim1.ccr2.read().ccr2().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
            Channel::_3 => {
                if sr.cc3of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc3if().bit_is_set() {
                    Ok(tim1.ccr3.read().ccr3().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
            Channel::_4 => {
                if sr.cc4of().bit_is_set() {
                    Err(nb::Error::Other(Error::Overcapture))
                } else if sr.cc4if().bit_is_set() {
                    Ok(tim1.ccr4.read().ccr4().bits())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
        }
    }

    fn disable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().clear_bit()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().clear_bit()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().clear_bit()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().clear_bit()),
        }
    }

    fn enable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().set_bit()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().set_bit()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().set_bit()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().set_bit()),
        }
    }

    fn get_resolution(&self) -> ::apb1::Ticks {
        ::apb1::Ticks(u32(self.0.psc.read().psc().bits()))
    }

    fn set_resolution<R>(&self, resolution: R)
    where
        R: Into<::apb1::Ticks>,
    {
        self._set_resolution(resolution.into())
    }
}

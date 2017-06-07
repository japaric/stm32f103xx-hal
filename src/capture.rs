//! Input capture
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

use core::u16;
use core::any::{Any, TypeId};

use cast::u16;
use either::Either;
use stm32f103xx::{AFIO, RCC, TIM2, TIM3, TIM4};

use frequency;
use timer::{Channel, Tim};

/// Input capture error
pub struct Error {
    _0: (),
}

/// Input capture interface
pub struct Capture<'a, T>(pub &'a T)
where
    T: Any + Tim;

impl<'a, T> Capture<'a, T>
where
    T: Any + Tim,
{
    /// Initializes the Capture interface
    ///
    /// - `1 / frequency` is the time each tick takes
    /// - captures on the rising edge by default
    /// - all the capture channels are disabled by default
    pub fn init(&self, frequency: u32, afio: &AFIO, gpio: &T::Gpio, rcc: &RCC) {
        let tim = self.0;

        match self.0.register_block() {
            Either::Left(tim1) => {
                // enable AFIO, TIM1 and GPIOA
                rcc.apb2enr.modify(|_, w| {
                    w.tim1en().enabled().afioen().enabled().iopaen().enabled()
                });

                // don't remap TIM1 pins
                afio.mapr.modify(
                    |_, w| unsafe { w.tim1_remap().bits(0b00) },
                );

                // CH1 = PA8 = floating input
                // CH2 = PA9 = floating input
                // CH3 = PA10 = floating input
                // CH4 = PA11 = floating input
                gpio.crh.modify(|_, w| {
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
                    w.bits((0b1111 << 12) | (0b01 << 8) |
                           (0b1111 << 4) |
                           (0b01 << 0))
                });
                tim1.ccmr2_output.write(|w| unsafe {
                    w.bits((0b1111 << 12) | (0b01 << 8) |
                           (0b1111 << 4) |
                           (0b01) << 0)
                });

                // enable capture on rising edge
                tim1.ccer.modify(|_, w| {
                    w.cc1p()
                        .clear()
                        .cc2p()
                        .clear()
                        .cc3p()
                        .clear()
                        .cc4p()
                        .clear()
                });

                let psc = u16((frequency::APB2 - 1) / frequency).unwrap();
                tim1.psc.write(|w| w.psc().bits(psc));

                // configure timer as a continuous upcounter and start
                tim1.cr1.write(
                    |w| w.dir().up().opm().continuous().cen().enabled(),
                );
            }
            Either::Right(tim2) => {
                // enable AFIO, GPIOx and TIMx
                if tim.get_type_id() == TypeId::of::<TIM2>() {
                    rcc.apb1enr.modify(|_, w| w.tim2en().enabled());
                } else if tim.get_type_id() == TypeId::of::<TIM3>() {
                    rcc.apb1enr.modify(|_, w| w.tim3en().enabled());
                } else if tim.get_type_id() == TypeId::of::<TIM4>() {
                    rcc.apb1enr.modify(|_, w| w.tim4en().enabled());
                }

                rcc.apb2enr.modify(|_, w| {
                    if tim.get_type_id() == TypeId::of::<TIM2>() {
                        w.iopaen().enabled()
                    } else if tim.get_type_id() == TypeId::of::<TIM3>() {
                        w.iopaen().enabled()
                    // TODO
                    // .iopben().enabled()
                    } else if tim.get_type_id() == TypeId::of::<TIM4>() {
                        w.iopben().enabled()
                    } else {
                        unreachable!()
                    }.afioen()
                        .enabled()
                });

                // don't remap TIM pins
                if tim.get_type_id() == TypeId::of::<TIM2>() {
                    afio.mapr.modify(
                        |_, w| unsafe { w.tim2_remap().bits(0b00) },
                    );
                } else if tim.get_type_id() == TypeId::of::<TIM3>() {
                    afio.mapr.modify(
                        |_, w| unsafe { w.tim3_remap().bits(0b00) },
                    );
                } else if tim.get_type_id() == TypeId::of::<TIM4>() {
                    afio.mapr.modify(|_, w| w.tim4_remap().clear());
                }

                if tim.get_type_id() == TypeId::of::<TIM2>() {
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
                } else if tim.get_type_id() == TypeId::of::<TIM3>() {
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
                } else if tim.get_type_id() == TypeId::of::<TIM4>() {
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
                    w.bits((0b1111 << 12) | (0b01 << 8) |
                           (0b1111 << 4) |
                           (0b01 << 0))
                });
                if tim.get_type_id() != TypeId::of::<TIM3>() {
                    tim2.ccmr2_output.write(|w| unsafe {
                        w.bits(
                            (0b1111 << 12) | (0b01 << 8) | (0b1111 << 4) |
                                (0b01) << 0,
                        )
                    });
                }

                // enable capture on rising edge
                // capture pins disabled by default
                if tim.get_type_id() == TypeId::of::<TIM3>() {
                    tim2.ccer.modify(|_, w| {
                        w.cc1p()
                            .clear()
                            .cc1e()
                            .clear()
                            .cc2p()
                            .clear()
                            .cc2e()
                            .clear()
                    });
                } else {
                    tim2.ccer.modify(|_, w| {
                        w.cc1p()
                            .clear()
                            .cc1e()
                            .clear()
                            .cc2p()
                            .clear()
                            .cc2e()
                            .clear()
                            .cc3p()
                            .clear()
                            .cc3e()
                            .clear()
                            .cc4p()
                            .clear()
                            .cc4e()
                            .clear()
                    });
                }

                let psc = u16((frequency::APB1 - 1) / frequency).unwrap();
                tim2.psc.write(|w| w.psc().bits(psc));

                tim2.arr.write(|w| w.arr().bits(u16::MAX));

                // configure timer as a continuous upcounter and start
                tim2.cr1.write(
                    |w| w.dir().up().opm().continuous().cen().enabled(),
                );
            }
        }
    }

    /// Captures input on `channel`
    pub fn capture(&self, channel: Channel) -> Result<u16, Error> {
        match self.0.register_block() {
            Either::Left(tim1) => {
                match channel {
                    Channel::_1 => {
                        if tim1.sr.read().cc1if().is_set() {
                            Ok(tim1.ccr1.read().ccr1().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                    Channel::_2 => {
                        if tim1.sr.read().cc2if().is_set() {
                            Ok(tim1.ccr2.read().ccr2().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                    Channel::_3 => {
                        if tim1.sr.read().cc3if().is_set() {
                            Ok(tim1.ccr3.read().ccr3().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                    Channel::_4 => {
                        if tim1.sr.read().cc4if().is_set() {
                            Ok(tim1.ccr4.read().ccr4().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                }
            }
            Either::Right(tim2) => {
                match channel {
                    Channel::_1 => {
                        if tim2.sr.read().cc1if().is_set() {
                            Ok(tim2.ccr1.read().ccr1().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                    Channel::_2 => {
                        if tim2.sr.read().cc2if().is_set() {
                            Ok(tim2.ccr2.read().ccr2().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                    Channel::_3 => {
                        if tim2.sr.read().cc3if().is_set() {
                            Ok(tim2.ccr3.read().ccr3().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                    Channel::_4 => {
                        if tim2.sr.read().cc4if().is_set() {
                            Ok(tim2.ccr4.read().ccr4().bits())
                        } else {
                            Err(Error { _0: () })
                        }
                    }
                }
            }
        }
    }

    /// Enables the capture `channel`
    pub fn enable(&self, channel: Channel) {
        match self.0.register_block() {
            Either::Left(tim1) => {
                match channel {
                    Channel::_1 => {
                        tim1.ccer.modify(|_, w| w.cc1e().set());
                    }
                    Channel::_2 => {
                        tim1.ccer.modify(|_, w| w.cc2e().set());
                    }
                    Channel::_3 => {
                        tim1.ccer.modify(|_, w| w.cc3e().set());
                    }
                    Channel::_4 => {
                        tim1.ccer.modify(|_, w| w.cc4e().set());
                    }
                }
            }
            Either::Right(tim2) => {
                match channel {
                    Channel::_1 => {
                        tim2.ccer.modify(|_, w| w.cc1e().set());
                    }
                    Channel::_2 => {
                        tim2.ccer.modify(|_, w| w.cc2e().set());
                    }
                    Channel::_3 => {
                        tim2.ccer.modify(|_, w| w.cc3e().set());
                    }
                    Channel::_4 => {
                        tim2.ccer.modify(|_, w| w.cc4e().set());
                    }
                }
            }
        }
    }

    /// Enables the capture `channel`
    pub fn disable(&self, channel: Channel) {
        match self.0.register_block() {
            Either::Left(tim1) => {
                match channel {
                    Channel::_1 => {
                        tim1.ccer.modify(|_, w| w.cc1e().clear());
                    }
                    Channel::_2 => {
                        tim1.ccer.modify(|_, w| w.cc2e().clear());
                    }
                    Channel::_3 => {
                        tim1.ccer.modify(|_, w| w.cc3e().clear());
                    }
                    Channel::_4 => {
                        tim1.ccer.modify(|_, w| w.cc4e().clear());
                    }
                }
            }
            Either::Right(tim2) => {
                match channel {
                    Channel::_1 => {
                        tim2.ccer.modify(|_, w| w.cc1e().clear());
                    }
                    Channel::_2 => {
                        tim2.ccer.modify(|_, w| w.cc2e().clear());
                    }
                    Channel::_3 => {
                        tim2.ccer.modify(|_, w| w.cc3e().clear());
                    }
                    Channel::_4 => {
                        tim2.ccer.modify(|_, w| w.cc4e().clear());
                    }
                }
            }
        }
    }
}

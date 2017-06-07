//! Pulse Width Modulation
//!
//! You can use the `Pwm` interface with these TIM instances
//!
//! # TIM1
//!
//! - CH1 = PA8
//! - CH2 = PA9
//! - CH3 = PA10
//! - CH4 = PA11
//!
//! # TIM2
//!
//! - CH1 = PA0
//! - CH2 = PA1
//! - CH3 = PA2
//! - CH4 = PA3
//!
// # TIM3
//
// - CH1 = PA6
// - CH2 = PA7
// - CH3 = PB0
// - CH4 = PB1
//
//! # TIM4
//!
//! - CH1 = PB6
//! - CH2 = PB7
//! - CH3 = PB8
//! - CH4 = PB9

use core::any::{Any, TypeId};
use core::cell::Cell;

use cast::{u16, u32};
use either::Either;
use stm32f103xx::{AFIO, RCC, TIM2, TIM3, TIM4};

use frequency;
use timer::{Channel, Tim};

/// PWM driver
pub struct Pwm<'a, T>
where
    T: Any + Tim,
{
    // Cached
    arr: Cell<Option<u16>>,
    tim: &'a T,
}

/// Creates a PWM driver
#[allow(non_snake_case)]
pub fn Pwm<'a, T>(tim2: &'a T) -> Pwm<'a, T>
where
    T: Any + Tim,
{
    Pwm {
        arr: Cell::new(None),
        tim: tim2,
    }
}

impl<'a, T> Pwm<'a, T>
where
    T: Any + Tim,
{
    /// Initializes the PWM module
    // FIXME simplify this once we have a &TIM1 -> &TIM2 method
    pub fn init(&self, frequency: u32, afio: &AFIO, gpio: &T::Gpio, rcc: &RCC) {
        let tim = self.tim;

        match self.tim.register_block() {
            Either::Left(tim1) => {
                // enable AFIO, TIM1 and GPIOA
                rcc.apb2enr.modify(|_, w| {
                    w.tim1en().enabled().afioen().enabled().iopaen().enabled()
                });

                // no remap of TIM1 pins
                afio.mapr.modify(
                    |_, w| unsafe { w.tim1_remap().bits(0b00) },
                );

                // CH1 = PA8 = alternate push-pull
                // CH2 = PA9 = alternate push-pull
                // CH3 = PA10 = alternate push-pull
                // CH4 = PA11 = alternate push-pull
                gpio.crh.modify(|_, w| {
                    w.mode8()
                        .output()
                        .cnf8()
                        .alt_push()
                        .mode9()
                        .output()
                        .cnf9()
                        .alt_push()
                        .mode10()
                        .output()
                        .cnf10()
                        .alt_push()
                        .mode11()
                        .output()
                        .cnf11()
                        .alt_push()
                });

                // PWM mode 1
                tim1.ccmr1_output.modify(|_, w| {
                    w.oc1pe().set().oc1m().pwm1().oc2pe().set().oc2m().pwm1()
                });
                tim1.ccmr2_output.modify(|_, w| {
                    w.oc3pe().set().oc3m().pwm1().oc4pe().set().oc4m().pwm1()
                });
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

                let ratio = frequency::APB1 / frequency;
                let psc = u16((ratio - 1) / (1 << 16)).unwrap();
                tim1.psc.write(|w| w.psc().bits(psc));
                let arr = u16(ratio / u32(psc + 1)).unwrap();
                tim1.arr.write(|w| w.arr().bits(arr));

                self.arr.set(Some(arr));

                tim1.egr.write(|w| w.ug().set());

                tim1.cr1.write(|w| unsafe {
                    w.cms()
                        .bits(0b00)
                        .dir()
                        .up()
                        .opm()
                        .continuous()
                        .cen()
                        .enabled()
                });
            }
            Either::Right(tim2) => {
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
                        w.iopaen().enabled().iopben().enabled()
                    } else if tim.get_type_id() == TypeId::of::<TIM4>() {
                        w.iopben().enabled()
                    } else {
                        unreachable!()
                    }.afioen()
                        .enabled()
                });

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
                    // CH1 = PA0 = alternate push-pull
                    // CH2 = PA1 = alternate push-pull
                    // CH3 = PA2 = alternate push-pull
                    // CH4 = PA3 = alternate push-pull
                    gpio.crl.modify(|_, w| {
                        w.mode0()
                            .output()
                            .cnf0()
                            .alt_push()
                            .mode1()
                            .output()
                            .cnf1()
                            .alt_push()
                            .mode2()
                            .output()
                            .cnf2()
                            .alt_push()
                            .mode3()
                            .output()
                            .cnf3()
                            .alt_push()
                    });
                } else if tim.get_type_id() == TypeId::of::<TIM3>() {
                    // CH1 = PA6 = alternate push-pull
                    // CH2 = PA7 = alternate push-pull
                    // CH3 = PB0 = alternate push-pull
                    // CH4 = PB1 = alternate push-pull
                    unimplemented!()
                } else if tim.get_type_id() == TypeId::of::<TIM4>() {
                    // CH1 = PB6 = alternate push-pull
                    // CH2 = PB7 = alternate push-pull
                    // CH3 = PB8 = alternate push-pull
                    // CH4 = PB9 = alternate push-pull
                    gpio.crl.modify(|_, w| {
                        w.mode6()
                            .output()
                            .cnf6()
                            .alt_push()
                            .mode7()
                            .output()
                            .cnf7()
                            .alt_push()
                    });

                    gpio.crh.modify(|_, w| {
                        w.mode8()
                            .output()
                            .cnf8()
                            .alt_push()
                            .mode9()
                            .output()
                            .cnf9()
                            .alt_push()
                    });
                }

                // PWM mode 1
                tim2.ccmr1_output.modify(|_, w| unsafe {
                    w.oc1pe()
                        .set()
                        .oc1m()
                        .bits(0b110)
                        .oc2pe()
                        .set()
                        .oc2m()
                        .bits(0b110)
                });
                tim2.ccmr2_output.modify(|_, w| unsafe {
                    w.oc3pe()
                        .set()
                        .oc3m()
                        .bits(0b110)
                        .oc4pe()
                        .set()
                        .oc4m()
                        .bits(0b110)
                });
                tim2.ccer.modify(|_, w| {
                    w.cc1p()
                        .clear()
                        .cc2p()
                        .clear()
                        .cc3p()
                        .clear()
                        .cc4p()
                        .clear()
                });

                let ratio = frequency::APB1 / frequency;
                let psc = u16((ratio - 1) / (1 << 16)).unwrap();
                tim2.psc.write(|w| w.psc().bits(psc));
                let arr = u16(ratio / u32(psc + 1)).unwrap();
                tim2.arr.write(|w| w.arr().bits(arr));

                self.arr.set(Some(arr));

                tim2.cr1.write(|w| unsafe {
                    w.cms()
                        .bits(0b00)
                        .dir()
                        .up()
                        .opm()
                        .continuous()
                        .cen()
                        .enabled()
                });
            }
        }
    }

    /// Returns the PWM period in ticks
    pub fn get_period(&self) -> u16 {
        match self.arr.get() {
            Some(arr) => arr,
            None => {
                let arr = match self.tim.register_block() {
                    Either::Left(tim1) => tim1.arr.read().arr().bits(),
                    Either::Right(tim2) => tim2.arr.read().arr().bits(),
                };
                self.arr.set(Some(arr));
                arr
            }
        }
    }

    /// Returns the duty cycle of the PWM `channel`
    pub fn get_duty(&self, channel: Channel) -> u16 {
        match self.tim.register_block() {
            Either::Left(tim1) => {
                match channel {
                    Channel::_1 => tim1.ccr1.read().ccr1().bits(),
                    Channel::_2 => tim1.ccr2.read().ccr2().bits(),
                    Channel::_3 => tim1.ccr3.read().ccr3().bits(),
                    Channel::_4 => tim1.ccr4.read().ccr4().bits(),
                }
            }
            Either::Right(tim2) => {
                match channel {
                    Channel::_1 => tim2.ccr1.read().ccr1().bits(),
                    Channel::_2 => tim2.ccr2.read().ccr2().bits(),
                    Channel::_3 => tim2.ccr3.read().ccr3().bits(),
                    Channel::_4 => tim2.ccr4.read().ccr4().bits(),
                }
            }
        }
    }

    /// Turns off the PWM `channel`
    pub fn off(&self, channel: Channel) {
        match self.tim.register_block() {
            Either::Left(tim1) => {
                match channel {
                    Channel::_1 => tim1.ccer.modify(|_, w| w.cc1e().clear()),
                    Channel::_2 => tim1.ccer.modify(|_, w| w.cc2e().clear()),
                    Channel::_3 => tim1.ccer.modify(|_, w| w.cc3e().clear()),
                    Channel::_4 => tim1.ccer.modify(|_, w| w.cc4e().clear()),
                }
            }
            Either::Right(tim2) => {
                match channel {
                    Channel::_1 => tim2.ccer.modify(|_, w| w.cc1e().clear()),
                    Channel::_2 => tim2.ccer.modify(|_, w| w.cc2e().clear()),
                    Channel::_3 => tim2.ccer.modify(|_, w| w.cc3e().clear()),
                    Channel::_4 => tim2.ccer.modify(|_, w| w.cc4e().clear()),
                }
            }
        }
    }

    /// Turns on the PWM `channel`
    pub fn on(&self, channel: Channel) {
        match self.tim.register_block() {
            Either::Left(tim1) => {
                match channel {
                    Channel::_1 => tim1.ccer.modify(|_, w| w.cc1e().set()),
                    Channel::_2 => tim1.ccer.modify(|_, w| w.cc2e().set()),
                    Channel::_3 => tim1.ccer.modify(|_, w| w.cc3e().set()),
                    Channel::_4 => tim1.ccer.modify(|_, w| w.cc4e().set()),
                }
            }
            Either::Right(tim2) => {
                match channel {
                    Channel::_1 => tim2.ccer.modify(|_, w| w.cc1e().set()),
                    Channel::_2 => tim2.ccer.modify(|_, w| w.cc2e().set()),
                    Channel::_3 => tim2.ccer.modify(|_, w| w.cc3e().set()),
                    Channel::_4 => tim2.ccer.modify(|_, w| w.cc4e().set()),
                }
            }
        }
    }

    /// Sets the duty cycle for the PWM `channel`
    pub fn set_duty(&self, channel: Channel, duty: u16) {
        match self.tim.register_block() {
            Either::Left(tim1) => {
                match channel {
                    Channel::_1 => tim1.ccr1.write(|w| w.ccr1().bits(duty)),
                    Channel::_2 => tim1.ccr2.write(|w| w.ccr2().bits(duty)),
                    Channel::_3 => tim1.ccr3.write(|w| w.ccr3().bits(duty)),
                    Channel::_4 => tim1.ccr4.write(|w| w.ccr4().bits(duty)),
                }
            }
            Either::Right(tim2) => {
                match channel {
                    Channel::_1 => tim2.ccr1.write(|w| w.ccr1().bits(duty)),
                    Channel::_2 => tim2.ccr2.write(|w| w.ccr2().bits(duty)),
                    Channel::_3 => tim2.ccr3.write(|w| w.ccr3().bits(duty)),
                    Channel::_4 => tim2.ccr4.write(|w| w.ccr4().bits(duty)),
                }
            }
        }
    }
}

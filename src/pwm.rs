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
//! # TIM3
//!
//! - CH1 = PA6
//! - CH2 = PA7
// - CH3 = PB0
// - CH4 = PB1
//!
//! # TIM4
//!
//! - CH1 = PB6
//! - CH2 = PB7
//! - CH3 = PB8
//! - CH4 = PB9

use core::any::{Any, TypeId};

use cast::{u16, u32};
use hal;
use stm32f103xx::{AFIO, GPIOA, RCC, TIM1, TIM2, TIM3, TIM4};

use timer::{Channel, TIM};

/// PWM driver
pub struct Pwm<'a, T>(pub &'a T)
where
    T: 'a;

impl<'a> Pwm<'a, TIM1> {
    /// Initializes the PWM module
    pub fn init<P>(&self, period: P, afio: &AFIO, gpioa: &GPIOA, rcc: &RCC)
    where
        P: Into<::apb2::Ticks>,
    {
        self._init(period.into(), afio, gpioa, rcc)
    }

    fn _init(
        &self,
        period: ::apb2::Ticks,
        afio: &AFIO,
        gpioa: &GPIOA,
        rcc: &RCC,
    ) {
        let tim1 = self.0;

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
        gpioa.crh.modify(|_, w| {
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

        self._set_period(period);

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

    fn _set_period(&self, period: ::apb2::Ticks) {
        let period = period.0;

        let psc = u16((period - 1) / (1 << 16)).unwrap();
        self.0.psc.write(|w| w.psc().bits(psc));

        let arr = u16(period / u32(psc + 1)).unwrap();
        self.0.arr.write(|w| w.arr().bits(arr));
    }
}

impl<'a> hal::Pwm for Pwm<'a, TIM1> {
    type Channel = Channel;
    type Time = ::apb2::Ticks;
    type Duty = u16;

    fn disable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().clear()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().clear()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().clear()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().clear()),
        }
    }

    fn enable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().set()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().set()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().set()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().set()),
        }
    }

    fn get_duty(&self, channel: Channel) -> u16 {
        match channel {
            Channel::_1 => self.0.ccr1.read().ccr1().bits(),
            Channel::_2 => self.0.ccr2.read().ccr2().bits(),
            Channel::_3 => self.0.ccr3.read().ccr3().bits(),
            Channel::_4 => self.0.ccr4.read().ccr4().bits(),
        }
    }

    fn get_max_duty(&self) -> u16 {
        self.0.arr.read().arr().bits()
    }

    fn get_period(&self) -> ::apb2::Ticks {
        ::apb2::Ticks(u32(self.0.psc.read().bits() * self.0.arr.read().bits()))
    }

    fn set_duty(&self, channel: Channel, duty: u16) {
        match channel {
            Channel::_1 => self.0.ccr1.write(|w| w.ccr1().bits(duty)),
            Channel::_2 => self.0.ccr2.write(|w| w.ccr2().bits(duty)),
            Channel::_3 => self.0.ccr3.write(|w| w.ccr3().bits(duty)),
            Channel::_4 => self.0.ccr4.write(|w| w.ccr4().bits(duty)),
        }
    }

    fn set_period<P>(&self, period: P)
    where
        P: Into<::apb2::Ticks>,
    {
        self._set_period(period.into())
    }
}

impl<'a, T> Pwm<'a, T>
where
    T: Any + TIM,
{
    /// Initializes the PWM module
    pub fn init<P>(&self, period: P, afio: &AFIO, gpio: &T::GPIO, rcc: &RCC)
    where
        P: Into<::apb1::Ticks>,
    {
        self._init(period.into(), afio, gpio, rcc)
    }

    fn _init(
        &self,
        period: ::apb1::Ticks,
        afio: &AFIO,
        gpio: &T::GPIO,
        rcc: &RCC,
    ) {
        let tim2 = self.0;

        if tim2.get_type_id() == TypeId::of::<TIM2>() {
            rcc.apb1enr.modify(|_, w| w.tim2en().enabled());
        } else if tim2.get_type_id() == TypeId::of::<TIM3>() {
            rcc.apb1enr.modify(|_, w| w.tim3en().enabled());
        } else if tim2.get_type_id() == TypeId::of::<TIM4>() {
            rcc.apb1enr.modify(|_, w| w.tim4en().enabled());
        }

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

        if tim2.get_type_id() == TypeId::of::<TIM2>() {
            afio.mapr.modify(
                |_, w| unsafe { w.tim2_remap().bits(0b00) },
            );

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
        } else if tim2.get_type_id() == TypeId::of::<TIM3>() {
            afio.mapr.modify(
                |_, w| unsafe { w.tim3_remap().bits(0b00) },
            );

            // CH1 = PA6 = alternate push-pull
            // CH2 = PA7 = alternate push-pull
            // CH3 = PB0 = alternate push-pull (TODO)
            // CH4 = PB1 = alternate push-pull (TODO)
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
        } else if tim2.get_type_id() == TypeId::of::<TIM4>() {
            afio.mapr.modify(|_, w| w.tim4_remap().clear());

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
        if tim2.get_type_id() == TypeId::of::<TIM3>() {
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

            tim2.ccer.modify(|_, w| w.cc1p().clear().cc2p().clear());
        } else {
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
        }

        self._set_period(period);

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

    fn _set_period(&self, period: ::apb1::Ticks) {
        let period = period.0;

        let psc = u16((period - 1) / (1 << 16)).unwrap();
        self.0.psc.write(|w| w.psc().bits(psc));

        let arr = u16(period / u32(psc + 1)).unwrap();
        self.0.arr.write(|w| w.arr().bits(arr));
    }
}

impl<'a, T> hal::Pwm for Pwm<'a, T>
where
    T: Any + TIM,
{
    type Channel = Channel;
    type Duty = u16;
    type Time = ::apb1::Ticks;

    fn get_duty(&self, channel: Channel) -> u16 {
        match channel {
            Channel::_1 => self.0.ccr1.read().ccr1().bits(),
            Channel::_2 => self.0.ccr2.read().ccr2().bits(),
            Channel::_3 => self.0.ccr3.read().ccr3().bits(),
            Channel::_4 => self.0.ccr4.read().ccr4().bits(),
        }
    }

    fn disable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().clear()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().clear()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().clear()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().clear()),
        }
    }

    fn enable(&self, channel: Channel) {
        match channel {
            Channel::_1 => self.0.ccer.modify(|_, w| w.cc1e().set()),
            Channel::_2 => self.0.ccer.modify(|_, w| w.cc2e().set()),
            Channel::_3 => self.0.ccer.modify(|_, w| w.cc3e().set()),
            Channel::_4 => self.0.ccer.modify(|_, w| w.cc4e().set()),
        }
    }

    fn get_max_duty(&self) -> u16 {
        self.0.arr.read().arr().bits()
    }

    fn get_period(&self) -> ::apb1::Ticks {
        ::apb1::Ticks(u32(self.0.psc.read().bits() * self.0.arr.read().bits()))
    }

    fn set_duty(&self, channel: Channel, duty: u16) {
        match channel {
            Channel::_1 => self.0.ccr1.write(|w| w.ccr1().bits(duty)),
            Channel::_2 => self.0.ccr2.write(|w| w.ccr2().bits(duty)),
            Channel::_3 => self.0.ccr3.write(|w| w.ccr3().bits(duty)),
            Channel::_4 => self.0.ccr4.write(|w| w.ccr4().bits(duty)),
        }
    }

    fn set_period<P>(&self, period: P)
    where
        P: Into<::apb1::Ticks>,
    {
        self._set_period(period.into())
    }
}

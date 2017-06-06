//! Pulse Width Modulation
//!
//! PA0 - TIM2_CH1
//! PA1 - TIM2_CH2
//! PA2 - TIM2_CH3
//! PA3 - TIM2_CH4

use core::cell::Cell;

use cast::{u16, u32};
use stm32f103xx::{AFIO, GPIOA, RCC, TIM2};

use frequency;

/// PWM channel
pub enum Channel {
    /// PA0
    _1,
    /// PA1
    _2,
    /// PA2
    _3,
    /// PA3
    _4,
}

/// PWM driver
pub struct Pwm<'a> {
    arr: Cell<Option<u16>>,
    tim2: &'a TIM2,
}

/// Creates a PWM driver
#[allow(non_snake_case)]
pub fn Pwm<'a>(tim2: &'a TIM2) -> Pwm<'a> {
    Pwm {
        arr: Cell::new(None),
        tim2: tim2,
    }
}

impl<'a> Pwm<'a> {
    /// Initializes the PWM module
    pub fn init(&self, frequency: u32, afio: &AFIO, gpioa: &GPIOA, rcc: &RCC) {
        let tim2 = self.tim2;

        rcc.apb1enr.modify(|_, w| w.tim2en().enabled());
        rcc.apb2enr.modify(
            |_, w| w.afioen().enabled().iopaen().enabled(),
        );

        // configure PA{0,1,2,3} as TIM2_CH{1,2,3,4} PWM output
        afio.mapr.modify(
            |_, w| unsafe { w.tim2_remap().bits(0b00) },
        );
        gpioa.crl.modify(|_, w| {
            w.cnf0()
                .bits(0b10)
                .mode0()
                .bits(0b10)
                .cnf1()
                .bits(0b10)
                .mode1()
                .bits(0b10)
                .cnf2()
                .bits(0b10)
                .mode2()
                .bits(0b10)
                .cnf3()
                .bits(0b10)
                .mode3()
                .bits(0b10)
        });

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

    /// Returns the PWM period in ticks
    pub fn get_period(&self) -> u16 {
        match self.arr.get() {
            Some(arr) => arr,
            None => {
                let arr = self.tim2.arr.read().arr().bits();
                self.arr.set(Some(arr));
                arr
            }
        }
    }

    /// Returns the duty cycle of the PWM `channel`
    pub fn get_duty(&self, channel: Channel) -> u16 {
        let tim2 = self.tim2;

        match channel {
            Channel::_1 => tim2.ccr1.read().ccr1().bits(),
            Channel::_2 => tim2.ccr2.read().ccr2().bits(),
            Channel::_3 => tim2.ccr3.read().ccr3().bits(),
            Channel::_4 => tim2.ccr4.read().ccr4().bits(),
        }
    }

    /// Turns off the PWM `channel`
    pub fn off(&self, channel: Channel) {
        let tim2 = self.tim2;

        match channel {
            Channel::_1 => tim2.ccer.modify(|_, w| w.cc1e().clear()),
            Channel::_2 => tim2.ccer.modify(|_, w| w.cc2e().clear()),
            Channel::_3 => tim2.ccer.modify(|_, w| w.cc3e().clear()),
            Channel::_4 => tim2.ccer.modify(|_, w| w.cc4e().clear()),
        }
    }

    /// Turns on the PWM `channel`
    pub fn on(&self, channel: Channel) {
        let tim2 = self.tim2;

        match channel {
            Channel::_1 => tim2.ccer.modify(|_, w| w.cc1e().set()),
            Channel::_2 => tim2.ccer.modify(|_, w| w.cc2e().set()),
            Channel::_3 => tim2.ccer.modify(|_, w| w.cc3e().set()),
            Channel::_4 => tim2.ccer.modify(|_, w| w.cc4e().set()),
        }
    }

    /// Sets the duty cycle for the PWM `channel`
    pub fn set_duty(&self, channel: Channel, duty: u16) {
        let tim2 = self.tim2;

        match channel {
            Channel::_1 => tim2.ccr1.write(|w| w.ccr1().bits(duty)),
            Channel::_2 => tim2.ccr2.write(|w| w.ccr2().bits(duty)),
            Channel::_3 => tim2.ccr3.write(|w| w.ccr3().bits(duty)),
            Channel::_4 => tim2.ccr4.write(|w| w.ccr4().bits(duty)),
        }
    }
}

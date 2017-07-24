//! Quadrature Encoder Interface (QEI)
//!
//! # TIM1
//!
//! - CH1 = PA8
//! - CH2 = PA9
//!
//! # TIM2
//!
//! - CH1 = PA0
//! - CH2 = PA1
//!
//! # TIM3
//!
//! - CH1 = PA6
//! - CH2 = PA7
//!
//! # TIM4
//!
//! - CH1 = PB6
//! - CH2 = PB7

use core::any::{Any, TypeId};
use core::u16;

use hal;
use stm32f103xx::{TIM1, TIM2, TIM3, TIM4, AFIO, GPIOA, RCC};

use timer::TIM;

/// Quadrature Encoder Interface
pub struct Qei<'a, T>(pub &'a T)
where
    T: 'a;

impl<'a, T> Clone for Qei<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Qei<'a, T> {}

impl<'a> Qei<'a, TIM1> {
    /// Initializes the QEI
    pub fn init(&self, afio: &AFIO, gpioa: &GPIOA, rcc: &RCC) {
        let tim1 = self.0;

        // enable TIM1
        rcc.apb2enr.modify(|_, w| w.tim1en().enabled());

        // enable GPIO and AFIO
        rcc.apb2enr
            .modify(|_, w| w.iopaen().enabled().afioen().enabled());

        // no remap of TIM1 pins
        afio.mapr
            .modify(|_, w| unsafe { w.tim1_remap().bits(0b00) });

        // CH1 = PA8 = floating input
        // CH2 = PA9 = floating input
        gpioa.crh.modify(|_, w| {
            w.cnf8()
                .bits(0b01)
                .mode8()
                .bits(0b00)
                .cnf9()
                .bits(0b01)
                .mode9()
                .bits(0b00)
        });

        // Configure TxC1 and TxC2 as captures
        tim1.ccmr1_output
            .write(|w| unsafe { w.bits({ (0b01 << 0) | (0b01 << 8) }) });

        // enable and configure to capture on rising edge
        tim1.ccer.write(|w| {
            w.cc1e()
                .set_bit()
                .cc1p()
                .clear_bit()
                .cc2e()
                .set_bit()
                .cc2p()
                .clear_bit()
        });

        // configure as quadrature encoder
        tim1.smcr.modify(|_, w| w.sms().encoder_ti1ti2());

        tim1.arr.write(|w| w.arr().bits(u16::MAX));
        tim1.cr1.write(|w| w.cen().enabled());
    }
}

impl<'a> hal::Qei for Qei<'a, TIM1> {
    type Count = u16;

    fn count(&self) -> u16 {
        self.0.cnt.read().cnt().bits()
    }

    fn direction(&self) -> hal::Direction {
        if self.0.cr1.read().dir().bit_is_clear() {
            hal::Direction::Upcounting
        } else {
            hal::Direction::Downcounting
        }
    }
}

impl<'a, T> Qei<'a, T>
where
    T: Any + TIM,
{
    /// Initializes the QEI
    pub fn init(&self, afio: &AFIO, gpio: &T::GPIO, rcc: &RCC) {
        let tim2 = self.0;

        // enable AFIO, GPIOx and TIMx
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
            gpio.crl.modify(|_, w| {
                w.mode0()
                    .input()
                    .cnf0()
                    .bits(0b01)
                    .mode1()
                    .input()
                    .cnf1()
                    .bits(0b01)
            });
        } else if tim2.get_type_id() == TypeId::of::<TIM3>() {
            afio.mapr
                .modify(|_, w| unsafe { w.tim3_remap().bits(0b00) });

            // CH1 = PA6 = floating input
            // CH2 = PA7 = floating input
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
        }

        // Configure TxC1 and TxC2 as captures
        tim2.ccmr1_output
            .write(|w| unsafe { w.bits({ (0b01 << 0) | (0b01 << 8) }) });

        // enable and configure to capture on rising edge
        tim2.ccer.write(|w| {
            w.cc1e()
                .set_bit()
                .cc1p()
                .clear_bit()
                .cc2e()
                .set_bit()
                .cc2p()
                .clear_bit()
        });

        // configure as quadrature encoder
        tim2.smcr.modify(|_, w| w.sms().encoder_ti1ti2());

        tim2.arr.write(|w| w.arr().bits(u16::MAX));
        tim2.cr1.write(|w| w.cen().enabled());
    }
}

impl<'a, T> hal::Qei for Qei<'a, T>
where
    T: Any + TIM,
{
    type Count = u16;

    fn count(&self) -> u16 {
        self.0.cnt.read().cnt().bits()
    }

    fn direction(&self) -> hal::Direction {
        if self.0.cr1.read().dir().bit_is_clear() {
            hal::Direction::Upcounting
        } else {
            hal::Direction::Downcounting
        }
    }
}

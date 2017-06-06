//! Quadrature Encoder Interface
//!
//! # TIM3
//!
//! - PA6 - T3C1
//! - PA7 - T3C1
//!
//! # TIM4
//!
//! - PB6 - T4C1
//! - PB7 - T4C1

use core::any::{Any, TypeId};
use core::ops::Deref;
use core::u16;

use stm32f103xx::{AFIO, GPIOA, GPIOB, RCC, TIM3, TIM4, gpioa, tim2};

/// A timer that can be used with the QEI
pub trait Timer: Any + Deref<Target = tim2::RegisterBlock> {
    /// I/O block associated to the QEI
    type Gpio: Any + Deref<Target = gpioa::RegisterBlock>;
}

impl Timer for TIM3 {
    type Gpio = GPIOA;
}

impl Timer for TIM4 {
    type Gpio = GPIOB;
}

/// Quadrature Encoder Interface
pub struct Qei<'a, T>(pub &'a T)
where
    T: Any + Timer;

impl<'a, T> Qei<'a, T>
where
    T: Any + Timer,
{
    /// Initializes the QEI
    pub fn init(&self, afio: &AFIO, gpio: &T::Gpio, rcc: &RCC) {
        let tim = self.0;

        // enable TIMer
        if tim.get_type_id() == TypeId::of::<TIM3>() {
            rcc.apb1enr.modify(|_, w| w.tim3en().enabled());
        } else if tim.get_type_id() == TypeId::of::<TIM4>() {
            rcc.apb1enr.modify(|_, w| w.tim4en().enabled());
        } else {
            unreachable!()
        }

        // enable GPIO and AFIO
        rcc.apb2enr.modify(|_, w| {
            if gpio.get_type_id() == TypeId::of::<GPIOA>() {
                w.iopaen().enabled()
            } else if gpio.get_type_id() == TypeId::of::<GPIOB>() {
                w.iopben().enabled()
            } else {
                unreachable!()
            }.afioen()
                .enabled()
        });

        // configure P{A,B}{6,7} as TIM{3,4}_CH{1,2} capture inputs
        if tim.get_type_id() == TypeId::of::<TIM3>() {
            afio.mapr.modify(
                |_, w| unsafe { w.tim3_remap().bits(0b00) },
            );
        } else if tim.get_type_id() == TypeId::of::<TIM4>() {
            afio.mapr.modify(|_, w| w.tim4_remap().clear());
        } else {
            unreachable!()
        }

        gpio.crl.modify(|_, w| {
            w.cnf6()
                .bits(0b01)
                .mode6()
                .bits(0b00)
                .cnf7()
                .bits(0b01)
                .mode7()
                .bits(0b00)
        });

        // Configure TxC1 and TxC2 as captures
        tim.ccmr1_output.write(|w| unsafe {
            w.bits({
                (0b01 << 0) | (0b01 << 8)
            })
        });

        // enable and configure to capture on rising edge
        tim.ccer.write(|w| {
            w.cc1e().set().cc1p().clear().cc2e().set().cc2p().clear()
        });

        // configure as quadrature encoder
        tim.smcr.modify(|_, w| w.sms().encoder_ti1ti2());

        tim.arr.write(|w| w.arr().bits(u16::MAX));
        tim.cr1.write(|w| w.cen().enabled());
    }

    /// Returns the count of the encoder
    pub fn count(&self) -> u16 {
        self.0.cnt.read().cnt().bits()
    }

    /// Returns the direction the encoder is counting
    pub fn direction(&self) -> Direction {
        if self.0.cr1.read().dir().is_clear() {
            Direction::Upcounting
        } else {
            Direction::Downcounting
        }
    }
}

/// Encoder direction
#[derive(Debug, Eq, PartialEq)]
pub enum Direction {
    /// Counting up
    Upcounting,
    /// Counting down
    Downcounting,
}

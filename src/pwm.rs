use core::marker::PhantomData;

use cast::{u16, u32};
use stm32f103xx::{TIM2, TIM3};

use afio::MAPR;
use bb;
use gpio::{AltPush, PA1, PA6};
use rcc::{Clocks, ENR};
use time::Hertz;

pub struct T2C2 {
    _0: (),
}

pub struct T3C1 {
    _0: (),
}

pub struct Pwm<CHANNEL, PIN> {
    _chan: PhantomData<CHANNEL>,
    pin: PIN,
}

impl<PIN> Pwm<T2C2, PIN> {
    pub fn disable(&mut self) {
        unsafe {
            bb::clear(&(*TIM2::ptr()).ccer, 4);
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            bb::set(&(*TIM2::ptr()).ccer, 4);
        }
    }

    pub fn get_duty(&self) -> u16 {
        unsafe { (*TIM3::ptr()).ccr2.read().ccr2().bits() }
    }

    pub fn get_max_duty(&self) -> u16 {
        unsafe { (*TIM3::ptr()).arr.read().arr().bits() }
    }

    pub fn set_duty(&mut self, duty: u16) {
        unsafe { (*TIM3::ptr()).ccr2.write(|w| w.ccr2().bits(duty)) }
    }

    pub fn unwrap(mut self) -> PIN {
        self.disable();
        self.pin
    }
}

impl<PIN> Pwm<T3C1, PIN> {
    pub fn disable(&mut self) {
        unsafe {
            bb::clear(&(*TIM3::ptr()).ccer, 0);
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            bb::set(&(*TIM3::ptr()).ccer, 0);
        }
    }

    pub fn get_duty(&self) -> u16 {
        unsafe { (*TIM3::ptr()).ccr1.read().ccr1().bits() }
    }

    pub fn get_max_duty(&self) -> u16 {
        unsafe { (*TIM3::ptr()).arr.read().arr().bits() }
    }

    pub fn set_duty(&mut self, duty: u16) {
        unsafe { (*TIM3::ptr()).ccr1.write(|w| w.ccr1().bits(duty)) }
    }

    pub fn unwrap(mut self) -> PIN {
        self.disable();
        self.pin
    }
}

pub trait PwmExt<Pins> {
    type Channels;
    type Time;

    fn pwm<F>(
        self,
        pins: Pins,
        frequency: F,
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Self::Channels
    where
        F: Into<Self::Time>;
}

impl PwmExt<PA1<AltPush>> for TIM2 {
    type Channels = Pwm<T2C2, PA1<AltPush>>;
    type Time = Hertz;

    fn pwm<F>(
        self,
        pin: PA1<AltPush>,
        freq: F,
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Pwm<T2C2, PA1<AltPush>>
    where
        F: Into<Self::Time>,
    {
        enr.apb1().modify(|_, w| w.tim2en().enabled());

        mapr.mapr()
            .modify(|_, w| unsafe { w.tim2_remap().bits(0b00) });

        self.ccmr1_output
            .modify(|_, w| w.oc2pe().set_bit().oc2m().pwm1());

        let clk = clocks.pclk1().0 * if clocks.ppre1() == 1 { 1 } else { 2 };
        let freq = freq.into().0;
        let ticks = clk / freq;
        let psc = u16(ticks / (1 << 16)).unwrap();
        self.psc.write(|w| w.psc().bits(psc));
        let arr = u16(ticks / u32(psc + 1)).unwrap();
        self.arr.write(|w| w.arr().bits(arr));

        self.cr1.write(|w| unsafe {
            w.cms()
                .bits(0b00)
                .dir()
                .up()
                .opm()
                .continuous()
                .cen()
                .enabled()
        });

        Pwm {
            _chan: PhantomData,
            pin,
        }
    }
}
impl PwmExt<PA6<AltPush>> for TIM3 {
    type Channels = Pwm<T3C1, PA6<AltPush>>;
    type Time = Hertz;

    fn pwm<F>(
        self,
        pin: PA6<AltPush>,
        freq: F,
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Pwm<T3C1, PA6<AltPush>>
    where
        F: Into<Self::Time>,
    {
        enr.apb1().modify(|_, w| w.tim3en().enabled());

        mapr.mapr()
            .modify(|_, w| unsafe { w.tim3_remap().bits(0b00) });

        self.ccmr1_output
            .modify(|_, w| w.oc1pe().set_bit().oc1m().pwm1());

        let clk = clocks.pclk1().0 * if clocks.ppre1() == 1 { 1 } else { 2 };
        let freq = freq.into().0;
        let ticks = clk / freq;
        let psc = u16(ticks / (1 << 16)).unwrap();
        self.psc.write(|w| w.psc().bits(psc));
        let arr = u16(ticks / u32(psc + 1)).unwrap();
        self.arr.write(|w| w.arr().bits(arr));

        self.cr1.write(|w| unsafe {
            w.cms()
                .bits(0b00)
                .dir()
                .up()
                .opm()
                .continuous()
                .cen()
                .enabled()
        });

        Pwm {
            _chan: PhantomData,
            pin,
        }
    }
}

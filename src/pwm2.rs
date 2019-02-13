use core::marker::PhantomData;

use cast::{u16, u32};
use crate::device::{TIM1, TIM2, TIM3, TIM4, tim2, tim1};
use crate::gpio::gpioa::{PA0, PA1, PA2, PA3, PA6, PA7, PA8, PA9, PA10, PA11, PA15};
use crate::gpio::gpiob::{PB0, PB1, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11};
use crate::afio::MAPR;
use crate::bb;
use crate::rcc::{APB1, APB2, Clocks};
use crate::gpio::Alternate;
use crate::time::{U32Ext, Hertz};

pub trait PwmTrait<TIM, BUS, REGISTERBLOCK, MAPPING>: Sized
    where MAPPING: TimChannelsMapping<TIM>,
          REGISTERBLOCK: TimRegisterType {
    fn create_pwm(&self, clocks: Clocks, _pins: MAPPING, apb: &mut BUS, mapr: &mut MAPR)
                  -> PwmTimer<TIM, BUS, REGISTERBLOCK, MAPPING> {
        self.enable_and_reset_tim(apb);
        self.configure_pin_mapping(mapr);
        <Self as PwmTrait<TIM, BUS, REGISTERBLOCK, MAPPING>>::get_register_block().configure();
        PwmTimer {
            _tim: PhantomData,
            _bus: PhantomData,
            _mapping: PhantomData,
            _register: PhantomData,
            clock_freq: Self::get_tim_clock_freq(clocks),
        }
    }
    fn enable_and_reset_tim(&self, apb: &mut BUS);
    fn configure_pin_mapping(&self, mapr: &mut MAPR);
    fn get_tim_clock_freq(clocks: Clocks) -> Hertz {
        // by default we return the apb1 clocks because it's the bus with most timers
        clocks.pclk1_tim()
    }
    fn write_ccer(ccer_bit: u8, value: bool) {
        <Self as PwmTrait<TIM, BUS, REGISTERBLOCK, MAPPING>>::get_register_block().write_ccer(ccer_bit, value)
    }
    fn get_period(clock_freq: u32) -> u32 {
        clock_freq / <Self as PwmTrait<TIM, BUS, REGISTERBLOCK, MAPPING>>::get_register_block().get_clock_division()
    }
    fn get_register_block<'a>() -> &'a REGISTERBLOCK;
}

pub trait TimerMapping<TIM> {
    type Channels;
}

pub struct PwmTimer<TIM, BUS, REGISTERBLOCK, MAPPING> {
    _tim: PhantomData<TIM>,
    _bus: PhantomData<BUS>,
    _mapping: PhantomData<MAPPING>,
    _register: PhantomData<REGISTERBLOCK>,
    clock_freq: Hertz,
}

pub enum Channel { _1, _2, _3, _4 }

impl<TIM, BUS, REGISTERBLOCK, MAPPING> PwmTimer<TIM, BUS, REGISTERBLOCK, MAPPING>
    where TIM: PwmTrait<TIM, BUS, REGISTERBLOCK, MAPPING>,
          MAPPING: TimChannelsMapping<TIM>,
          REGISTERBLOCK: TimRegisterType {
    fn channel_to_ccer_bit(channel: Channel) -> u8 {
        match channel {
            Channel::_1 => 0u8,
            Channel::_2 => 4u8,
            Channel::_3 => 8u8,
            Channel::_4 => 12u8,
        }
    }
}

impl<TIM, BUS, REGISTERBLOCK, MAPPING> crate::hal::Pwm for PwmTimer<TIM, BUS, REGISTERBLOCK, MAPPING>
    where TIM: PwmTrait<TIM, BUS, REGISTERBLOCK, MAPPING>,
          MAPPING: TimChannelsMapping<TIM>,
          REGISTERBLOCK: TimRegisterType {
    type Channel = Channel;
    type Time = Hertz;
    type Duty = u16;

    fn disable(&mut self, channel: Self::Channel) {
        TIM::write_ccer(Self::channel_to_ccer_bit(channel), false);
    }

    fn enable(&mut self, channel: Self::Channel) {
        TIM::write_ccer(Self::channel_to_ccer_bit(channel), true);
    }

    fn get_period(&self) -> Self::Time {
        TIM::get_period(self.clock_freq.0).hz()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        TIM::get_register_block().get_channel_duty(channel)
    }

    fn get_max_duty(&self) -> Self::Duty {
        TIM::get_register_block().get_counter_autoreload()
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        TIM::get_register_block().set_channel_duty(channel, duty)
    }

    fn set_period<P>(&mut self, period: P) where
        P: Into<Self::Time> {
        let ticks = self.clock_freq.0 / period.into().0;
        let psc = u16(ticks / (1 << 16)).unwrap();
        TIM::get_register_block().write_prescaler(psc);
        let arr = u16(ticks / u32(psc + 1)).unwrap();
        TIM::get_register_block().write_counter_autoreload(arr);
    }
}

pub trait TimChannelsMapping<TIM>: Sized {
    const MAPPING: u8;
}

// this trait unifies Advanced Control timers and General-purpose timers
pub trait TimRegisterType {
    fn get_counter_autoreload(&self) -> u16;
    fn get_clock_division(&self) -> u32;
    fn write_ccer(&self, ccer_bits: u8, value: bool);
    fn write_prescaler(&self, value: u16);
    fn write_counter_autoreload(&self, value: u16);
    fn configure(&self);
    fn set_channel_duty(&self, channel: Channel, value: u16);
    fn get_channel_duty(&self, channel: Channel) -> u16;
}

impl TimRegisterType for tim2::RegisterBlock {
    fn get_counter_autoreload(&self) -> u16 {
        self.arr.read().arr().bits()
    }
    fn get_clock_division(&self) -> u32 {
        (u32(self.psc.read().psc().bits()) + 1)
            * (u32(self.get_counter_autoreload()) + 1)
    }
    fn write_ccer(&self, ccer_bits: u8, value: bool) {
        bb::write(&self.ccer, ccer_bits, value)
    }
    fn write_prescaler(&self, value: u16) {
        self.psc.write(|w| w.psc().bits(value));
    }
    fn write_counter_autoreload(&self, value: u16) {
        self.arr.write(|w| w.arr().bits(value));
    }
    fn configure(&self) {
        self.ccmr1_output.modify(|_, w| w.oc1pe().set_bit().oc1m().pwm1());
        self.ccmr1_output.modify(|_, w| w.oc2pe().set_bit().oc2m().pwm1());
        self.ccmr2_output.modify(|_, w| w.oc3pe().set_bit().oc3m().pwm1());
        self.ccmr2_output.modify(|_, w| w.oc4pe().set_bit().oc4m().pwm1());
        self.cr1.write(|w| unsafe {
            w.cms().bits(0b00).dir().up().opm().continuous().cen().enabled()
        });
    }
    fn set_channel_duty(&self, channel: Channel, value: u16) {
        match channel {
            Channel::_1 => { self.ccr1.write(|w| w.ccr1().bits(value)) }
            Channel::_2 => { self.ccr2.write(|w| w.ccr2().bits(value)) }
            Channel::_3 => { self.ccr3.write(|w| w.ccr3().bits(value)) }
            Channel::_4 => { self.ccr4.write(|w| w.ccr4().bits(value)) }
        }
    }
    fn get_channel_duty(&self, channel: Channel) -> u16 {
        match channel {
            Channel::_1 => self.ccr1.read().ccr1().bits(),
            Channel::_2 => self.ccr2.read().ccr2().bits(),
            Channel::_3 => self.ccr3.read().ccr3().bits(),
            Channel::_4 => self.ccr4.read().ccr4().bits()
        }
    }
}

impl TimRegisterType for tim1::RegisterBlock {
    fn get_counter_autoreload(&self) -> u16 {
        self.arr.read().arr().bits()
    }
    fn get_clock_division(&self) -> u32 {
        (u32(self.psc.read().psc().bits()) + 1)
            * (u32(self.get_counter_autoreload()) + 1)
    }
    fn write_ccer(&self, ccer_bits: u8, value: bool) {
        bb::write(&self.ccer, ccer_bits, value)
    }
    fn write_prescaler(&self, value: u16) {
        self.psc.write(|w| w.psc().bits(value));
    }
    fn write_counter_autoreload(&self, value: u16) {
        self.arr.write(|w| w.arr().bits(value));
    }
    fn configure(&self) {
        self.ccmr1_output.modify(|_, w| w.oc1pe().set_bit().oc1m().pwm1());
        self.ccmr1_output.modify(|_, w| w.oc2pe().set_bit().oc2m().pwm1());
        self.ccmr2_output.modify(|_, w| w.oc3pe().set_bit().oc3m().pwm1());
        self.ccmr2_output.modify(|_, w| w.oc4pe().set_bit().oc4m().pwm1());
        self.cr1.write(|w| unsafe {
            w.cms().bits(0b00).dir().up().opm().continuous().cen().enabled()
        });
    }
    fn set_channel_duty(&self, channel: Channel, value: u16) {
        match channel {
            Channel::_1 => { self.ccr1.write(|w| w.ccr1().bits(value)) }
            Channel::_2 => { self.ccr2.write(|w| w.ccr2().bits(value)) }
            Channel::_3 => { self.ccr3.write(|w| w.ccr3().bits(value)) }
            Channel::_4 => { self.ccr4.write(|w| w.ccr4().bits(value)) }
        }
    }
    fn get_channel_duty(&self, channel: Channel) -> u16 {
        match channel {
            Channel::_1 => self.ccr1.read().ccr1().bits(),
            Channel::_2 => self.ccr2.read().ccr2().bits(),
            Channel::_3 => self.ccr3.read().ccr3().bits(),
            Channel::_4 => self.ccr4.read().ccr4().bits()
        }
    }
}

// TIM1 ////////////////////////
pub type Pwm1Mapping0<M1, M2, M3, M4> = (Option<PA8<Alternate<M1>>>,
                                         Option<PA9<Alternate<M2>>>,
                                         Option<PA10<Alternate<M3>>>,
                                         Option<PA11<Alternate<M4>>>);
pub type PWM1<Mapping> = PwmTimer<TIM1, APB2, tim1::RegisterBlock, Mapping>;

impl<M1, M2, M3, M4> TimChannelsMapping<TIM1> for Pwm1Mapping0<M1, M2, M3, M4>
{ const MAPPING: u8 = 0; }

impl<T> PwmTrait<TIM1, APB2, tim1::RegisterBlock, T> for TIM1
    where T: TimChannelsMapping<TIM1> {
    fn enable_and_reset_tim(&self, apb: &mut APB2) {
        apb.enr().modify(|_, w| w.tim1en().enabled());
        apb.rstr().modify(|_, w| w.tim1rst().set_bit());
        apb.rstr().modify(|_, w| w.tim1rst().clear_bit());
    }

    fn configure_pin_mapping(&self, mapr: &mut MAPR) {
        mapr.mapr().modify(|_, w| unsafe { w.tim1_remap().bits(T::MAPPING) });
    }
    fn get_tim_clock_freq(clocks: Clocks) -> Hertz {
        clocks.pclk2_tim()
    }
    fn get_register_block<'a>() -> &'a tim1::RegisterBlock {
        unsafe { &(*TIM1::ptr()) }
    }
}

// TIM2 ////////////////////////
pub type Pwm2Mapping0<M1, M2, M3, M4> = (Option<PA0<Alternate<M1>>>,
                                         Option<PA1<Alternate<M2>>>,
                                         Option<PA2<Alternate<M3>>>,
                                         Option<PA3<Alternate<M4>>>);
pub type Pwm2Mapping01<M1, M2, M3, M4> = (Option<PA15<Alternate<M1>>>,
                                          Option<PB3<Alternate<M2>>>,
                                          Option<PA2<Alternate<M3>>>,
                                          Option<PA3<Alternate<M4>>>);
pub type Pwm2Mapping10<M1, M2, M3, M4> = (Option<PA0<Alternate<M1>>>,
                                          Option<PA1<Alternate<M2>>>,
                                          Option<PB10<Alternate<M3>>>,
                                          Option<PB11<Alternate<M4>>>);
pub type Pwm2Mapping11<M1, M2, M3, M4> = (Option<PA15<Alternate<M1>>>,
                                          Option<PB3<Alternate<M2>>>,
                                          Option<PB10<Alternate<M3>>>,
                                          Option<PB11<Alternate<M4>>>);

pub type PWM2<Mapping> = PwmTimer<TIM2, APB1, tim2::RegisterBlock, Mapping>;

impl<M1, M2, M3, M4> TimChannelsMapping<TIM2> for Pwm2Mapping0<M1, M2, M3, M4>
{ const MAPPING: u8 = 0; }

impl<M1, M2, M3, M4> TimChannelsMapping<TIM2> for Pwm2Mapping01<M1, M2, M3, M4>
{ const MAPPING: u8 = 0b01; }

impl<M1, M2, M3, M4> TimChannelsMapping<TIM2> for Pwm2Mapping10<M1, M2, M3, M4>
{ const MAPPING: u8 = 0b10; }

impl<M1, M2, M3, M4> TimChannelsMapping<TIM2> for Pwm2Mapping11<M1, M2, M3, M4>
{ const MAPPING: u8 = 0b11; }

impl<T> PwmTrait<TIM2, APB1, tim2::RegisterBlock, T> for TIM2
    where T: TimChannelsMapping<TIM2> {
    fn enable_and_reset_tim(&self, apb: &mut APB1) {
        apb.enr().modify(|_, w| w.tim2en().enabled());
        apb.rstr().modify(|_, w| w.tim2rst().set_bit());
        apb.rstr().modify(|_, w| w.tim2rst().clear_bit());
    }
    fn configure_pin_mapping(&self, mapr: &mut MAPR) {
        mapr.mapr().modify(|_, w| unsafe { w.tim2_remap().bits(T::MAPPING) });
    }
    fn get_register_block<'a>() -> &'a tim2::RegisterBlock {
        unsafe { &(*TIM2::ptr()) }
    }
}

// TIM3 ////////////////////////
pub type Pwm3Mapping0<M1, M2, M3, M4> = (Option<PA6<Alternate<M1>>>,
                                         Option<PA7<Alternate<M2>>>,
                                         Option<PB0<Alternate<M3>>>,
                                         Option<PB1<Alternate<M4>>>);

pub type Pwm3Mapping10<M1, M2, M3, M4> = (Option<PB4<Alternate<M1>>>,
                                          Option<PB5<Alternate<M2>>>,
                                          Option<PB0<Alternate<M3>>>,
                                          Option<PB1<Alternate<M4>>>);
pub type PWM3<Mapping> = PwmTimer<TIM3, APB1, tim2::RegisterBlock, Mapping>;

impl<M1, M2, M3, M4> TimChannelsMapping<TIM3> for Pwm3Mapping0<M1, M2, M3, M4>
{ const MAPPING: u8 = 0; }

impl<M1, M2, M3, M4> TimChannelsMapping<TIM3> for Pwm3Mapping10<M1, M2, M3, M4>
{ const MAPPING: u8 = 0b10; }

impl<T> PwmTrait<TIM3, APB1, tim2::RegisterBlock, T> for TIM3
    where T: TimChannelsMapping<TIM3> {
    fn enable_and_reset_tim(&self, apb: &mut APB1) {
        apb.enr().modify(|_, w| w.tim3en().enabled());
        apb.rstr().modify(|_, w| w.tim3rst().set_bit());
        apb.rstr().modify(|_, w| w.tim3rst().clear_bit());
    }
    fn configure_pin_mapping(&self, mapr: &mut MAPR) {
        mapr.mapr().modify(|_, w| unsafe { w.tim3_remap().bits(T::MAPPING) });
    }
    fn get_register_block<'a>() -> &'a tim2::RegisterBlock {
        unsafe { &(*TIM3::ptr()) }
    }
}

// TIM4 ////////////////////////
pub type Pwm4Mapping0<M1, M2, M3, M4> = (Option<PB6<Alternate<M1>>>,
                                         Option<PB7<Alternate<M2>>>,
                                         Option<PB8<Alternate<M3>>>,
                                         Option<PB9<Alternate<M4>>>);
pub type PWM4<Mapping> = PwmTimer<TIM4, APB1, tim2::RegisterBlock, Mapping>;

impl<M1, M2, M3, M4> TimChannelsMapping<TIM4> for Pwm4Mapping0<M1, M2, M3, M4>
{ const MAPPING: u8 = 0; }

impl<T> PwmTrait<TIM4, APB1, tim2::RegisterBlock, T> for TIM4
    where T: TimChannelsMapping<TIM4> {
    fn enable_and_reset_tim(&self, apb: &mut APB1) {
        apb.enr().modify(|_, w| w.tim4en().enabled());
        apb.rstr().modify(|_, w| w.tim4rst().set_bit());
        apb.rstr().modify(|_, w| w.tim4rst().clear_bit());
    }
    fn configure_pin_mapping(&self, mapr: &mut MAPR) {
        mapr.mapr().modify(|_, w| w.tim4_remap().bit(T::MAPPING != 0));
    }

    fn get_register_block<'a>() -> &'a tim2::RegisterBlock {
        unsafe { &(*TIM4::ptr()) }
    }
}
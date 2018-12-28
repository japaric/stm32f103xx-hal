use stm32f103xx::{ADC1, ADC2, ADC3};
// TODO: Clear TODO comments


use crate::gpio::Analog;
use crate::gpio::gpioa::*;
use crate::gpio::gpiob::*;

use crate::rcc::APB2;

pub struct Adc<ADC> {
    adc: ADC
}

pub trait AnalogPin<ADC> {
    fn analog_read(&self, adc: &mut ADC) -> u16;
}

// trait AdcPin <ADC, T> {
//     fn analog_read()
// }


// Enabling ADC
// Turn ADC on by setting ADON in ADC_CR2
// Wait for t_STAB
// Set ADON again

//XXX: Maybe ensure that PCLK2 is on. Do we need the clock to
//run the ADC in single conversion mode?

macro_rules! hal {
    ($(
        $ADC:ident: (
            $init:ident,
            $adcxen:ident,
            $adcxrst:ident
        ),
    )+) => {
        $(
            impl Adc<$ADC> {
                /**
                  Powers up $ADC and blocks until it's ready
                */
                pub fn $init(adc: $ADC, apb2: &mut APB2) -> Self {
                    // Reset and enable the ADC peripheral
                    apb2.rstr().modify(|_, w| w.$adcxrst().set_bit());
                    apb2.rstr().modify(|_, w| w.$adcxrst().clear_bit());
                    apb2.enr().modify(|_, w| w.$adcxen().set_bit());


                    adc.cr2.modify(|_, w| { w.cont().clear_bit()});

                    adc.cr2.modify(|_, w| { w.adon().set_bit()});

                    // Wait for the ADC to be ready
                    while adc.cr2.read().adon().bit_is_set() == false
                        {}

                    // Set the sequence length to 1
                    // Amount of conversions n-1

                    unsafe{adc.sqr1.modify(|_, w| w.l().bits(0))}
                    Self {
                        adc
                    }
                }

                /**
                  Make a single reading of the specified channel
                */
                fn read(&mut self, channel: u8) -> u16 {
                    // Select the channel to be converted
                    // NOTE: Unsafe write of u8 to 4 bit register. Will this cause issues?
                    unsafe{self.adc.sqr3.modify(|_, w| w.sq1().bits(channel))};
                    // Set ADON
                    // self.adc.cr2.modify(|_, w| w.swstart().set_bit());
                    self.adc.cr2.modify(|_, w| { w.adon().set_bit()});
                    // Wait for end of conversion
                    while self.adc.sr.read().eoc().bit() == false
                        {}
                    // Read the data in the ADC_DR reg
                    self.adc.dr.read().data().bits()
                }
            }
        )+
    }
}


macro_rules! analog_pin_impls {
    ($($adc:ty: ($($pin:ident: $channel:expr),+)),+) =>
    {
        $(
            $(
                impl AnalogPin<$adc> for $pin<Analog> {
                    fn analog_read(&self, adc: &mut $adc) -> u16 {
                        adc.read($channel)
                    }
                }
            )+
        )+
    }
}


hal! {
    ADC1: (
        adc1,
        adc1en,
        adc1rst
    ),
    ADC2: (
        adc2,
        adc2en,
        adc2rst
    ),
    ADC3: (
        adc3,
        adc3en,
        adc3rst
    ),
}

analog_pin_impls!{
    Adc<ADC1>: (
        PA0: 0,
        PA1: 1,
        PA2: 2,
        PA3: 3,
        PA4: 4,
        PA5: 5,
        PA6: 6,
        PA7: 7,
        PB0: 8,
        PB1: 9
    ),
    Adc<ADC2>: (
        PA0: 0,
        PA1: 1,
        PA2: 2,
        PA3: 3,
        PA4: 4,
        PA5: 5,
        PA6: 6,
        PA7: 7,
        PB0: 8,
        PB1: 9
    )
}

use stm32f103xx::{ADC1, ADC2, ADC3};
// TODO: Clear TODO comments

use rcc::APB2;

pub struct Adc<ADC> {
    adc: ADC
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
                    
                    let status = adc.cr2.read().bits();

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
                pub fn read(&mut self, channel: u8) -> u16 {
                    // Select the channel to be converted
                    // NOTE: Unsafe write of u8 to 4 bit register. Will this cause issues?
                    unsafe{self.adc.sqr3.modify(|_, w| w.sq1().bits(channel))};
                    // Set ADON
                    // self.adc.cr2.modify(|_, w| w.swstart().set_bit());
                    self.adc.cr2.modify(|_, w| { w.adon().set_bit()});
                    // Wait for end of conversion
                    while self.adc.sr.read().eoc().bit() == false
                        {}

                    // TODO: Check if we need to clear the EOC bit

                    // Read the data in the ADC_DR reg
                    self.adc.dr.read().data().bits()
                }
            }
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

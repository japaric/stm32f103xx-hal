use stm32f103xx::{ADC1, ADC2, ADC3};

pub struct Adc<ADC> {
    adc: ADC
}
// Enabling ADC
// Turn ADC on by setting ADON in ADC_CR2
// Wait for t_STAB
// Set ADON again

macro_rules! hal {
    ($(
        $ADC:ident: (
            $init:ident
        ),
    )+) => {
        $(
            impl Adc<$ADC> {
                /**
                  Powers up $ADC and blocks until it's ready
                */
                pub fn $init(adc: $ADC) -> Self {
                    adc.cr2.modify(|_, w| w.adon().set_bit());

                    // Wait for the ADC to be ready
                    while adc.cr2.read().adon().bit() == true {}

                    Self {
                        adc
                    }
                }
            }


        )+
    }

}



hal! {
    ADC1: (
        adc1
    ),
    ADC2: (
        adc2
    ),
    ADC3: (
        adc3
    ),
}

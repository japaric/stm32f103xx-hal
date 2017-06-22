//! Analog to Digital Converter

use core::marker::Unsize;

use cast::u16;
use hal::prelude::*;
use static_ref::Ref;

use dma::{self, CircBuffer, Dma1Channel1};
use stm32f103xx::{ADC1, DMA1, GPIOA, RCC, TIM2};
use {Channel, Pwm};

/// ADC Channel 1 (PA1)
pub struct Adc1<'a>(pub &'a ADC1);

impl<'a> Adc1<'a> {
    /// Initializes the ADC
    ///
    /// NOTE `Pwm<TIM2>.init` must be called before this method because both
    /// methods configure the PA1 pin (one as input and the other as output :-/)
    pub fn init(&self, dma1: &DMA1, gpioa: &GPIOA, rcc: &RCC) {
        let adc1 = self.0;

        // enable ADC1, DMA1, GPIOA, TIM2
        rcc.ahbenr.modify(|_, w| w.dma1en().enabled());
        rcc.apb1enr.modify(|_, w| w.tim2en().enabled());
        rcc.apb2enr
            .modify(|_, w| w.adc1en().enabled().iopaen().enabled());

        // Set PA1 as analog input
        gpioa.crl.modify(|_, w| w.cnf1().bits(0b00).mode1().input());

        // Sample only the channel 1
        adc1.sqr1.modify(|_, w| unsafe { w.l().bits(1) });
        adc1.sqr3.modify(|_, w| unsafe { w.sq1().bits(1) });

        // Sample time: 55.5 + 12.5 = 68 cycles
        adc1.smpr2.modify(|_, w| unsafe { w.smp1().bits(0b101) });

        // ADC1
        // mem2mem: Memory to memory mode disabled
        // pl: Medium priority
        // msize: Memory size = 16 bits
        // psize: Peripheral size = 16 bits
        // minc: Memory increment mode enabled
        // pinc: Peripheral increment mode disabled
        // circ: Circular mode enabled
        // dir: Transfer from peripheral to memory
        // htie: Half transfer interrupt enabled
        // tceie: Transfer complete interrupt enabled
        // en: Disabled
        dma1.ccr1.write(|w| unsafe {
            w.mem2mem()
                .clear()
                .pl()
                .bits(0b01)
                .msize()
                .bits(0b01)
                .psize()
                .bits(0b01)
                .minc()
                .set()
                .pinc()
                .clear()
                .circ()
                .set()
                .dir()
                .clear()
                .htie()
                .set()
                .tcie()
                .set()
                .en()
                .clear()
        });

        // exttrig: Conversion on external event enabled
        // extsel: Timer 2 CC2 event
        // align: Right alignment
        // dma: DMA mode enabled
        // cont: Single conversion mode
        // adon: Disable ADC conversion
        adc1.cr2.write(|w| unsafe {
            w.exttrig()
                .set()
                .extsel()
                .bits(0b011) // T2C2
                // .bits(0b111) // swstart
                .align()
                .clear()
                .dma()
                .set()
                .cont()
                .clear()
                .adon()
                .clear()
        });
    }

    /// Disables the ADC
    pub fn disable(&self) {
        self.0.cr2.modify(|_, w| w.adon().clear());
    }

    /// Enables the ADC
    pub fn enable(&self) {
        self.0.cr2.modify(|_, w| w.adon().set());
    }

    /// Starts an analog to digital conversion that will be periodically
    /// triggered by the channel 2 of TIM2
    ///
    /// The conversions will be stored in the circular `buffer`
    pub fn start<B>(
        &self,
        buffer: Ref<CircBuffer<u16, B, Dma1Channel1>>,
        dma1: &DMA1,
        pwm: Pwm<TIM2>,
    ) -> Result<(), dma::Error>
    where
        B: Unsize<[u16]>,
    {
        let adc1 = self.0;


        if dma1.ccr1.read().en().is_set() {
            return Err(dma::Error::InUse);
        }

        pwm.disable(Channel::_2);
        pwm.set_duty(Channel::_2, 1);

        let buffer: &[u16] = &buffer.lock()[0];

        dma1.cndtr1
            .write(|w| unsafe { w.ndt().bits(u16(buffer.len() * 2).unwrap()) });

        dma1.cpar1
            .write(|w| unsafe { w.bits(&adc1.dr as *const _ as u32) });

        dma1.cmar1
            .write(|w| unsafe { w.bits(buffer.as_ptr() as u32) });

        dma1.ccr1.modify(|_, w| w.en().set());
        pwm.enable(Channel::_2);

        Ok(())
    }
}

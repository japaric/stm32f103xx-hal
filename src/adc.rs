use core::marker::Unsize;

use cast::u16;
use hal::PwmPin;
use stm32f103xx::{ADC1, DMA1};

use dma::{CircTransfer, D1C1, Static};
use gpio::gpioa::CRL;
use gpio::{AltPush, PA1};
use pwm::{Pwm, T2C2};
use rcc::ENR;

pub fn start<B>(
    mut pwm: Pwm<T2C2, PA1<AltPush>>,
    buffer: Static<[B; 2]>,
    adc: ADC1,
    enr: &mut ENR,
    crl: &mut CRL,
    chan: D1C1,
) -> CircTransfer<B, D1C1>
where
    B: Unsize<[u16]>,
{
    // enable the ADC
    enr.apb2().modify(|_, w| w.adc1en().enabled());

    // set PA1 as analog input
    crl.crl().modify(|_, w| w.cnf1().bits(0b00).mode1().input());

    // Sample only the channel 1
    adc.sqr1.modify(|_, w| unsafe { w.l().bits(1) });
    adc.sqr3.modify(|_, w| unsafe { w.sq1().bits(1) });

    // Sample time: 55.5 + 12.5 = 68 cycles
    adc.smpr2.modify(|_, w| unsafe { w.smp1().bits(0b101) });

    let dma = unsafe { &*DMA1::ptr() };

    {
        let slice: &[u16] = &buffer[0];
        dma.cndtr1
            .write(|w| unsafe { w.ndt().bits(u16(slice.len() * 2).unwrap()) });

        dma.cpar1
            .write(|w| unsafe { w.bits(&adc.dr as *const _ as u32) });

        dma.cmar1
            .write(|w| unsafe { w.bits(slice.as_ptr() as u32) });
    }

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
    dma.ccr1.write(|w| unsafe {
        w.mem2mem()
            .clear_bit()
            .msize()
            .bits(0b01)
            .psize()
            .bits(0b01)
            .minc()
            .set_bit()
            .pinc()
            .clear_bit()
            .circ()
            .set_bit()
            .dir()
            .clear_bit()
            .htie()
            .set_bit()
            .tcie()
            .set_bit()
            .en()
            .set_bit()
    });

    // exttrig: Conversion on external event enabled
    // extsel: Timer 2 CC2 event
    // align: Right alignment
    // dma: DMA mode enabled
    // cont: Single conversion mode
    // adon: Disable ADC conversion
    adc.cr2.write(|w| unsafe {
        w.exttrig()
            .set_bit()
            .extsel()
            .bits(0b011) // T2C2
            .align()
            .clear_bit()
            .dma()
            .set_bit()
            .cont()
            .clear_bit()
            .adon()
            .set_bit()
    });

    pwm.set_duty(1);
    pwm.enable();

    unsafe { CircTransfer::new(buffer, chan) }
}

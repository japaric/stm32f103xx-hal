use core::marker::PhantomData;
use core::u16;

use cast::u16;
use nb;
use stm32f103xx::TIM4;

use afio::MAPR;
use bb;
use gpio::{Input, PB6};
use rcc::{Clocks, ENR};
use time::MicroSeconds;

#[derive(Debug)]
pub enum Error {
    /// Previous capture value was overwritten
    Overcapture,
    #[doc(hidden)] _Extensible,
}

pub struct T4C1 {
    _0: (),
}

pub struct Capture<PIN> {
    _pin: PhantomData<PIN>,
}

impl Capture<T4C1> {
    pub fn enable(&mut self) {
        unsafe {
            bb::set(&(*TIM4::ptr()).ccer, 0);
        }
    }

    pub fn disable(&mut self) {
        unsafe {
            bb::clear(&(*TIM4::ptr()).ccer, 0);
        }
    }

    pub fn capture(&mut self) -> nb::Result<u16, Error> {
        let tim = unsafe { &*TIM4::ptr() };
        let sr = tim.sr.read();

        if sr.cc1of().bit_is_set() {
            Err(nb::Error::Other(Error::Overcapture))
        } else if sr.cc1if().bit_is_set() {
            Ok(tim.ccr1.read().ccr1().bits())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

pub trait CaptureExt<Pins> {
    type Channels;
    type Time;

    fn capture<R>(
        self,
        pins: Pins,
        res: R,
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Self::Channels
    where
        R: Into<Self::Time>;
}

impl<I> CaptureExt<PB6<I>> for TIM4
where
    I: Input,
{
    type Channels = Capture<T4C1>;
    type Time = MicroSeconds;

    fn capture<R>(
        self,
        _: PB6<I>,
        res: R,
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Capture<T4C1>
    where
        R: Into<MicroSeconds>,
    {
        enr.apb1().modify(|_, w| w.tim4en().enabled());

        mapr.mapr().modify(|_, w| w.tim4_remap().clear_bit());

        let clk = clocks.pclk1().0 * if clocks.ppre1() == 1 { 1 } else { 2 };
        let res = res.into().0;
        let psc = u16(((clk - 1) / 1_000_000) * res ).unwrap();
        self.psc.write(|w| w.psc().bits(psc));
        self.arr.write(|w| w.arr().bits(u16::MAX));

        // configure CC1 as input and wire it to TI1
        // apply the heaviest filter
        self.ccmr1_output.write(|w| unsafe {
            w.bits((0b1111 << 12) | (0b01 << 8) | (0b1111 << 4) | (0b01 << 0))
        });

        // enable capture on rising edge
        self.ccer.write(|w| w.cc1p().clear_bit());

        // configure timer as a continuous upcounter and start
        self.cr1
            .write(|w| w.dir().up().opm().continuous().cen().enabled());

        Capture { _pin: PhantomData }
    }
}

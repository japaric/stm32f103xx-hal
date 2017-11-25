use stm32f103xx::{GPIOA, GPIOB, GPIOC};

use rcc::ENR;

pub trait GpioExt {
    type Parts;

    fn split(self, enr: &mut ENR) -> Self::Parts;
}

// States
pub struct AltPush;
pub struct InputFloating;
pub struct InputPullDown;
pub struct InputPullUp;
pub struct Output;

pub trait Input {}

impl Input for InputFloating {}
impl Input for InputPullUp {}
impl Input for InputPullDown {}

macro_rules! gpio {
    ($GPIO:ident, $gpio:ident, $iopen:ident, [
        $($PIN:ident: ($pin:ident, $n:expr, $CR:ident, $cr:ident),)+
    ]) => {
        pub mod $gpio {
            use stm32f103xx::{gpioa, $GPIO};
            use super::InputFloating;

            pub struct Parts {
                pub crl: CRL,
                pub crh: CRH,
                $(pub $pin: super::$PIN<InputFloating>,)+
            }

            // Registers
            pub struct CRL {
                _0: (),
            }

            impl CRL {
                pub(crate) unsafe fn new() -> Self {
                    CRL { _0: () }
                }

                // FIXME GPIOC has no CRL
                #[allow(dead_code)]
                pub(crate) fn crl(&mut self) -> &gpioa::CRL {
                    unsafe { &(*$GPIO::ptr()).crl }
                }
            }

            pub struct CRH {
                _0: (),
            }

            impl CRH {
                pub(crate) unsafe fn new() -> Self {
                    CRH { _0: () }
                }

                pub(crate) fn crh(&mut self) -> &gpioa::CRH {
                    unsafe { &(*$GPIO::ptr()).crh }
                }
            }
        }

        impl GpioExt for $GPIO {
            type Parts = $gpio::Parts;

            fn split(self, enr: &mut ENR) -> $gpio::Parts {
                enr.apb2().modify(|_, w| w.$iopen().enabled());

                $gpio::Parts {
                    crl: unsafe { $gpio::CRL::new() },
                    crh: unsafe { $gpio::CRH::new() },
                    $($pin: $PIN { _state: InputFloating },)+
                }
            }
        }

        $(
            pub struct $PIN<STATE> {
                _state: STATE,
            }

            impl<STATE> $PIN<STATE> {
                pub fn as_alt_push(self, cr: &mut $gpio::$CR) -> $PIN<AltPush> {
                    // MODE = 0b01 = 10 MHz output
                    // CNF = 0b10 = Alternate function output push-pull
                    cr.$cr()
                        .modify(|r, w| unsafe {
                            w.bits(r.bits() & !(0b1111 << (4*$n % 32)) | 0b1001 << (4*$n % 32))
                        });

                    $PIN { _state: AltPush }
                }

                pub fn as_floating_input(self, cr: &mut $gpio::$CR) -> $PIN<InputFloating> {
                    // MODE = 0b00 = floating input
                    // CNF = 0b01 = input mode
                    cr.$cr()
                        .modify(|r, w| unsafe {
                            w.bits(r.bits() & !(0b1111 << (4*$n % 32)) | 0b0100 << (4*$n % 32))
                        });

                    $PIN { _state: InputFloating }
                }

                pub fn as_pull_down_input(self, cr: &mut $gpio::$CR) -> $PIN<InputPullDown> {
                    // MODE = 0b10 = input pull-X
                    // CNF = 0b01 = input mode
                    cr.$cr()
                        .modify(|r, w| unsafe {
                            w.bits(r.bits() & !(0b1111 << (4*$n % 32)) | 0b1000 << (4*$n % 32))
                        });

                    // NOTE atomic write to stateless register
                    unsafe {
                        (*$GPIO::ptr()).bsrr.write(|w| w.bits(1 << (16 + $n)))
                    }

                    $PIN { _state: InputPullDown }
                }

                pub fn as_pull_up_input(self, cr: &mut $gpio::$CR) -> $PIN<InputPullDown> {
                    // MODE = 0b10 = input pull-X
                    // CNF = 0b01 = input mode
                    cr.$cr()
                        .modify(|r, w| unsafe {
                            w.bits(r.bits() & !(0b1111 << (4*$n % 32)) | 0b1000 << (4*$n % 32))
                        });

                    // NOTE atomic write to stateless register
                    unsafe {
                        (*$GPIO::ptr()).bsrr.write(|w| w.bits(1 << $n))
                    }

                    $PIN { _state: InputPullDown }
                }

                pub fn as_output(self, cr: &mut $gpio::$CR) -> $PIN<Output> {
                    // MODE = 0b01 = 10 MHz output
                    // CNF = 0b00 = General purpose output push-pull
                    cr.$cr()
                        .modify(|r, w| unsafe {
                            w.bits(r.bits() & !(0b1111 << (4*$n % 32)) | 0b0001 << (4*$n % 32))
                        });

                    $PIN { _state: Output }
                }
            }

            impl<I> $PIN<I> where I: Input {
                pub fn is_high(&self) -> bool {
                    !self.is_low()
                }

                pub fn is_low(&self) -> bool {
                    // NOTE atomic read with not side effects
                    unsafe { (*$GPIO::ptr()).idr.read().bits() & (1 << $n) == 0 }
                }
            }

            impl $PIN<Output> {
                pub fn is_high(&self) -> bool {
                    !self.is_low()
                }

                pub fn is_low(&self) -> bool {
                    // NOTE atomic read with not side effects
                    unsafe { (*$GPIO::ptr()).odr.read().bits() & (1 << $n) == 0 }
                }

                pub fn set_high(&mut self) {
                    // NOTE atomic write to a stateless register
                    unsafe { (*$GPIO::ptr()).bsrr.write(|w| w.bits(1 << $n)) }
                }

                pub fn set_low(&mut self) {
                    // NOTE atomic write to a stateless register
                    unsafe { (*$GPIO::ptr()).bsrr.write(|w| w.bits(1 << (16 + $n))) }
                }
            }
        )+
    }
}

gpio!(GPIOA, gpioa, iopaen, [
    PA0: (pa0, 0, CRL, crl),
    PA1: (pa1, 1, CRL, crl),
    PA2: (pa2, 2, CRL, crl),
    PA3: (pa3, 3, CRL, crl),
    PA4: (pa4, 4, CRL, crl),
    PA5: (pa5, 5, CRL, crl),
    PA6: (pa6, 6, CRL, crl),
    PA7: (pa7, 7, CRL, crl),
    PA8: (pa8, 8, CRH, crh),
    PA9: (pa9, 9, CRH, crh),
    PA10: (pa10, 10, CRH, crh),
    PA11: (pa11, 11, CRH, crh),
    PA12: (pa12, 12, CRH, crh),
    PA13: (pa13, 13, CRH, crh),
    PA14: (pa14, 14, CRH, crh),
    PA15: (pa15, 15, CRH, crh),
]);

gpio!(GPIOB, gpiob, iopben, [
    PB0: (pb0, 0, CRL, crl),
    PB1: (pb1, 1, CRL, crl),
    PB2: (pb2, 2, CRL, crl),
    PB3: (pb3, 3, CRL, crl),
    PB4: (pb4, 4, CRL, crl),
    PB5: (pb5, 5, CRL, crl),
    PB6: (pb6, 6, CRL, crl),
    PB7: (pb7, 7, CRL, crl),
    PB8: (pb8, 8, CRH, crh),
    PB9: (pb9, 9, CRH, crh),
    PB10: (pb10, 10, CRH, crh),
    PB11: (pb11, 11, CRH, crh),
    PB12: (pb12, 12, CRH, crh),
    PB13: (pb13, 13, CRH, crh),
    PB14: (pb14, 14, CRH, crh),
    PB15: (pb15, 15, CRH, crh),
]);

gpio!(GPIOC, gpioc, iopcen, [
    PC13: (pc13, 13, CRH, crh),
    PC14: (pc14, 14, CRH, crh),
    PC15: (pc15, 15, CRH, crh),
]);

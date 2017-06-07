//! Serial Peripheral Interface
//!
//! You can use the `Spi` interface with these SPI instances
//!
//! # SPI1
//!
//! - nCS = PA4
//! - SCK = PA5
//! - MISO = PA6
//! - MOSI = PA7
//!
//! # SPI2
//!
//! - nCS = PB12
//! - SCK = PB13
//! - MISO = PB14
//! - MOSI = PB15

use core::any::{Any, TypeId};
use core::ops::Deref;
use core::ptr;

use stm32f103xx::{AFIO, GPIOA, GPIOB, RCC, SPI1, SPI2, gpioa, spi1};

/// SPI instance that can be used with the `Spi` abstraction
pub unsafe trait Bus: Deref<Target = spi1::RegisterBlock> {
    /// GPIO block associated to this SPI instance
    type GPIO: Deref<Target = gpioa::RegisterBlock>;
}

unsafe impl Bus for SPI1 {
    type GPIO = GPIOA;
}

unsafe impl Bus for SPI2 {
    type GPIO = GPIOB;
}

/// SPI error
pub struct Error {
    _0: (),
}

/// SPI result
pub type Result<T> = ::core::result::Result<T, Error>;

/// Serial Peripheral Interface
pub struct Spi<'a, S>(pub &'a S)
where
    S: Any + Bus;

impl<'a, S> Spi<'a, S>
where
    S: Any + Bus,
{
    /// Initializes the SPI
    pub fn init(&self, afio: &AFIO, gpio: &S::GPIO, rcc: &RCC) {
        let spi = self.0;

        if spi.get_type_id() == TypeId::of::<SPI1>() {
            // enable AFIO, SPI1, GPIOA
            rcc.apb2enr.modify(|_, w| {
                w.afioen().enabled().spi1en().enabled().iopaen().enabled()
            });

            // do not remap the SPI1 pins
            afio.mapr.modify(|_, w| w.spi1_remap().clear());

            // NSS = PA4 = Alternate function push pull
            // SCK = PA5 = Alternate function push pull
            // MISO = PA6 = Floating input
            // MOSI = PA7 = Alternate function push pull
            gpio.crl.modify(|_, w| {
                w.cnf4()
                    .bits(0b10)
                    .mode4()
                    .bits(0b10)
                    .cnf5()
                    .bits(0b10)
                    .mode5()
                    .bits(0b10)
                    .cnf6()
                    .bits(0b01)
                    .mode6()
                    .bits(0b00)
                    .cnf7()
                    .bits(0b10)
                    .mode7()
                    .bits(0b10)
            });
        } else if spi.get_type_id() == TypeId::of::<SPI2>() {
            // enable AFIO, SPI1, GPIOA
            rcc.apb1enr.modify(|_, w| w.spi2en().enabled());
            rcc.apb2enr.modify(
                |_, w| w.afioen().enabled().iopben().enabled(),
            );

            // NSS = PB12 = Alternate function push pull
            // SCK = PB13 = Alternate function push pull
            // MISO = PB14 = Floating input
            // MOSI = PB15 = Alternate function push pull
            gpio.crh.modify(|_, w| {
                w.cnf12()
                    .bits(0b10)
                    .mode12()
                    .bits(0b10)
                    .cnf13()
                    .bits(0b10)
                    .mode13()
                    .bits(0b10)
                    .cnf14()
                    .bits(0b01)
                    .mode14()
                    .bits(0b00)
                    .cnf15()
                    .bits(0b10)
                    .mode15()
                    .bits(0b10)
            });
        }

        // enable SS output
        spi.cr2.write(|w| w.ssoe().set());

        // cpha: second clock transition is the first data capture
        // cpol: CK to 1 when idle
        // mstr: master configuration
        // br: 1 MHz frequency
        // lsbfirst: MSB first
        // ssm: disable software slave management
        // dff: 8 bit frames
        // bidimode: 2-line unidirectional
        spi.cr1.write(|w| unsafe {
            w.cpha()
                .set()
                .cpol()
                .set()
                .mstr()
                .set()
                .br()
                .bits(0b10)
                .lsbfirst()
                .clear()
                .ssm()
                .clear()
                .rxonly()
                .clear()
                .dff()
                .clear()
                .bidimode()
                .clear()
        });
    }

    /// Disables the SPI bus
    pub fn disable(&self) {
        self.0.cr1.modify(|_, w| w.spe().clear())
    }

    /// Enables the SPI bus
    pub fn enable(&self) {
        self.0.cr1.modify(|_, w| w.spe().set())
    }

    /// Receives a byte
    ///
    /// NOTE You *must* send a byte before you can receive one
    pub fn receive(&self) -> Result<u8> {
        let spi1 = self.0;

        if spi1.sr.read().rxne().is_set() {
            unsafe { Ok(ptr::read_volatile(&spi1.dr as *const _ as *const u8)) }
        } else {
            Err(Error { _0: () })
        }
    }

    /// Sends a byte
    pub fn send(&self, byte: u8) -> Result<()> {
        let spi1 = self.0;

        if spi1.sr.read().txe().is_set() {
            // NOTE(write_volatile) see note above
            unsafe {
                ptr::write_volatile(&spi1.dr as *const _ as *mut u8, byte)
            }
            Ok(())
        } else {
            Err(Error { _0: () })
        }
    }
}

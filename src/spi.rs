//! Serial Peripheral Interface
//!
//! You can use the `Spi` interface with these SPI instances
//!
//! # SPI1
//!
//! - NSS = PA4
//! - SCK = PA5
//! - MISO = PA6
//! - MOSI = PA7
//!
//! # SPI2
//!
//! - NSS = PB12
//! - SCK = PB13
//! - MISO = PB14
//! - MOSI = PB15

use core::any::{Any, TypeId};
use core::ops::Deref;
use core::ptr;

use hal;
use nb;
use stm32f103xx::{AFIO, GPIOA, GPIOB, RCC, SPI1, SPI2, gpioa, spi1};

/// SPI instance that can be used with the `Spi` abstraction
pub unsafe trait SPI: Deref<Target = spi1::RegisterBlock> {
    /// GPIO block associated to this SPI instance
    type GPIO: Deref<Target = gpioa::RegisterBlock>;
}

unsafe impl SPI for SPI1 {
    type GPIO = GPIOA;
}

unsafe impl SPI for SPI2 {
    type GPIO = GPIOB;
}

/// SPI result
pub type Result<T> = ::core::result::Result<T, nb::Error<Error>>;

/// SPI error
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)]
    _Extensible,
}

/// Serial Peripheral Interface
pub struct Spi<'a, S>(pub &'a S)
where
    S: Any + SPI;

impl<'a, S> Spi<'a, S>
where
    S: Any + SPI,
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
                w.mode4()
                    .output()
                    .cnf4()
                    .alt_push()
                    .mode5()
                    .output()
                    .cnf5()
                    .alt_push()
                    .mode6()
                    .input()
                    .cnf6()
                    .bits(0b01)
                    .mode7()
                    .output()
                    .cnf7()
                    .alt_push()
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
                w.mode12()
                    .output()
                    .cnf12()
                    .alt_push()
                    .mode13()
                    .output()
                    .cnf13()
                    .alt_push()
                    .mode14()
                    .input()
                    .cnf14()
                    .bits(0b01)
                    .mode15()
                    .output()
                    .cnf15()
                    .alt_push()
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
    ///
    /// **NOTE** This drives the NSS pin high
    pub fn disable(&self) {
        self.0.cr1.modify(|_, w| w.spe().clear())
    }

    /// Enables the SPI bus
    ///
    /// **NOTE** This drives the NSS pin low
    pub fn enable(&self) {
        self.0.cr1.modify(|_, w| w.spe().set())
    }
}

impl<'a, S> hal::Spi<u8> for Spi<'a, S>
where
    S: Any + SPI,
{
    type Error = Error;

    fn read(&self) -> Result<u8> {
        let spi1 = self.0;
        let sr = spi1.sr.read();

        if sr.ovr().is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.modf().is_set() {
            Err(nb::Error::Other(Error::ModeFault))
        } else if sr.crcerr().is_set() {
            Err(nb::Error::Other(Error::Crc))
        } else if sr.rxne().is_set() {
            Ok(unsafe {
                ptr::read_volatile(&spi1.dr as *const _ as *const u8)
            })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn send(&self, byte: u8) -> Result<()> {
        let spi1 = self.0;
        let sr = spi1.sr.read();

        if sr.ovr().is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.modf().is_set() {
            Err(nb::Error::Other(Error::ModeFault))
        } else if sr.crcerr().is_set() {
            Err(nb::Error::Other(Error::Crc))
        } else if sr.txe().is_set() {
            // NOTE(write_volatile) see note above
            unsafe {
                ptr::write_volatile(&spi1.dr as *const _ as *mut u8, byte)
            }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

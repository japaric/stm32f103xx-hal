use core::ptr;

use hal;
use nb;
use stm32f103xx::SPI1;

use afio::MAPR;
use gpio::{AltPush, InputFloating, PA4, PA5, PA6, PA7};
use rcc::{Clocks, ENR};

/// SPI error
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)] _Extensible,
}

pub struct Spi {
    spi: SPI1,
}

impl Spi {
    /// - PA4 - NSS1
    /// - PA5 - SCK1
    /// - PA6 - MISO1
    /// - PA7 - MOSI1
    pub fn new(
        spi: SPI1,
        _: (PA4<AltPush>, PA5<AltPush>, PA6<InputFloating>, PA7<AltPush>),
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Self {
        // enable the SPI peripheral
        enr.apb2().modify(|_, w| w.spi1en().enabled());

        mapr.mapr().modify(|_, w| w.spi1_remap().clear_bit());

        // enable SS output
        spi.cr2.write(|w| w.ssoe().set_bit());

        let br = match clocks.pclk2().0 / 1_000_000 {
            0 => unreachable!(),
            1...2 => 0b000,
            3...5 => 0b001,
            6...11 => 0b010,
            12...23 => 0b011,
            24...47 => 0b100,
            48...95 => 0b101,
            96...191 => 0b110,
            _ => 0b111,
        };

        // cpha: second clock transition is the first data capture
        // cpol: CK to 1 when idle
        // mstr: master configuration
        // br: 1 MHz frequency
        // lsbfirst: MSB first
        // ssm: disable software slave management
        // dff: 8 bit frames
        // bidimode: 2-line unidirectional
        spi.cr1.write(|w| {
            w.cpha()
                .set_bit()
                .cpol()
                .set_bit()
                .mstr()
                .set_bit()
                .br()
                .bits(br)
                .lsbfirst()
                .clear_bit()
                .ssm()
                .clear_bit()
                .rxonly()
                .clear_bit()
                .dff()
                .clear_bit()
                .bidimode()
                .clear_bit()
        });

        Spi { spi }
    }

    /// Disables the SPI bus
    ///
    /// **NOTE** This drives the NSS pin high
    pub fn disable(&mut self) {
        self.spi.cr1.modify(|_, w| w.spe().clear_bit());
    }

    /// Enables the SPI bus
    ///
    /// **NOTE** This drives the NSS pin low
    pub fn enable(&mut self) {
        self.spi.cr1.modify(|_, w| w.spe().set_bit());
    }
}

impl hal::Spi<u8> for Spi {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        let sr = self.spi.sr.read();

        if sr.ovr().bit_is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.modf().bit_is_set() {
            Err(nb::Error::Other(Error::ModeFault))
        } else if sr.crcerr().bit_is_set() {
            Err(nb::Error::Other(Error::Crc))
        } else if sr.rxne().bit_is_set() {
            Ok(unsafe {
                ptr::read_volatile(&self.spi.dr as *const _ as *const u8)
            })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
        let sr = self.spi.sr.read();

        if sr.ovr().bit_is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.modf().bit_is_set() {
            Err(nb::Error::Other(Error::ModeFault))
        } else if sr.crcerr().bit_is_set() {
            Err(nb::Error::Other(Error::Crc))
        } else if sr.txe().bit_is_set() {
            // NOTE(write_volatile) see note above
            unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

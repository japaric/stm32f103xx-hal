use core::ptr;

use hal::spi::{self, Mode, Phase, Polarity};
use nb;
use stm32f103xx::SPI1;

use afio::MAPR;
use gpio::{AltPush, InputFloating, PA5, PA6, PA7};
use rcc::{Clocks, ENR};
use time::Hertz;

/// SPI error
#[derive(Debug, PartialEq)]
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
    /// MSB format
    ///
    /// - PA5 - SCK1
    /// - PA6 - MISO1
    /// - PA7 - MOSI1
    pub fn new<F>(
        spi: SPI1,
        (_sck, _miso, _mosi): (PA5<AltPush>, PA6<InputFloating>, PA7<AltPush>),
        mode: Mode,
        freq: F,
        clocks: Clocks,
        enr: &mut ENR,
        mapr: &mut MAPR,
    ) -> Self
    where
        F: Into<Hertz>,
    {
        // enable the SPI peripheral
        enr.apb2().modify(|_, w| w.spi1en().enabled());

        mapr.mapr().modify(|_, w| w.spi1_remap().clear_bit());

        // disable SS output
        spi.cr2.write(|w| w.ssoe().clear_bit());

        let br = match clocks.pclk2().0 / freq.into().0 {
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

        let cpol = mode.polarity == Polarity::IdleHigh;
        let cpha = mode.phase == Phase::CaptureOnSecondTransition;

        // mstr: master configuration
        // lsbfirst: MSB first
        // ssm: enable software slave management (NSS pin free for other uses)
        // ssi: set nss high = master mode
        // dff: 8 bit frames
        // bidimode: 2-line unidirectional
        // spe: enable the SPI bus
        spi.cr1.write(|w| {
            w.cpha()
                .bit(cpha)
                .cpol()
                .bit(cpol)
                .mstr()
                .set_bit()
                .br()
                .bits(br)
                .lsbfirst()
                .clear_bit()
                .ssm()
                .set_bit()
                .ssi()
                .set_bit()
                .rxonly()
                .clear_bit()
                .dff()
                .clear_bit()
                .bidimode()
                .clear_bit()
                .spe()
                .set_bit()
        });

        Spi { spi }
    }
}

impl spi::FullDuplex<u8> for Spi {
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

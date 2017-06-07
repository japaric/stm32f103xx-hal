//! Serial Peripheral Interface
//!
//! - PA4 - nCS
//! - PA5 - SCK
//! - PA6 - MISO
//! - PA7 - MOSI

use core::ptr;

use stm32f103xx::{AFIO, GPIOA, RCC, SPI1};

/// SPI error
pub struct Error {
    _0: (),
}

/// SPI result
pub type Result<T> = ::core::result::Result<T, Error>;

/// Serial Peripheral Interface
pub struct Spi<'a>(pub &'a SPI1);

impl<'a> Spi<'a> {
    /// Initializes the SPI
    pub fn init(&self, afio: &AFIO, gpioa: &GPIOA, rcc: &RCC) {
        let spi1 = self.0;

        // Enable AFIO, SPI1, GPIOB
        rcc.apb2enr.modify(|_, w| {
            w.afioen()
                .enabled()
                .spi1en()
                .enabled()
                .iopaen()
                .enabled()
                .iopben()
                .enabled()
        });

        afio.mapr.modify(|_, w| w.spi1_remap().clear());

        gpioa.crh.modify(|_, w| w);

        // NSS: Alternate function push pull
        // SCK: Alternate function push pull
        // MISO: Floating input
        // MOSI: Alternate function push pull
        gpioa.crl.modify(|_, w| {
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

        // enable SS output
        spi1.cr2.write(|w| w.ssoe().set());

        // cpha: second clock transition is the first data capture
        // cpol: CK to 1 when idle
        // mstr: master configuration
        // br: 1 MHz frequency
        // lsbfirst: MSB first
        // ssm: disable software slave management
        // dff: 8 bit frames
        // bidimode: 2-line unidirectional
        spi1.cr1.write(|w| unsafe {
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

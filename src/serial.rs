//! Serial interface

use core::ptr;

use stm32f103xx::{AFIO, GPIOA, RCC, USART1};

use frequency;

/// Specialized `Result` type
pub type Result<T> = ::core::result::Result<T, Error>;

/// An error
pub struct Error {
    _0: (),
}

/// Serial interface
///
/// # Interrupts
///
/// - `Usart1` - RXNE
#[derive(Clone, Copy)]
pub struct Serial<'a>(pub &'a USART1);

impl<'a> Serial<'a> {
    /// Initializes the serial interface with a baud rate of `baut_rate` bits
    /// per second
    pub fn init(&self, baud_rate: u32, afio: &AFIO, gpioa: &GPIOA, rcc: &RCC) {
        let usart1 = self.0;

        // power up peripherals
        rcc.apb2enr.modify(|_, w| {
            w.afioen().enabled().iopaen().enabled().usart1en().enabled()
        });

        // wire the PA9 and PA10 pins to USART1
        afio.mapr.modify(|_, w| w.usart1_remap().clear());
        gpioa.crh.modify(|_, w| {
            w.mode9()
                .output()
                .cnf9()
                .alt_push()
                .mode10()
                .input()
                .cnf10()
                .bits(0b01)
        });

        // 8N1
        usart1.cr2.write(|w| unsafe { w.stop().bits(0b00) });

        // baud rate
        usart1.brr.write(
            |w| unsafe { w.bits(frequency::APB2 / baud_rate) },
        );

        // disable hardware flow control
        usart1.cr3.write(|w| w.rtse().clear().ctse().clear());

        // enable TX, RX; enable RXNE; disable parity checking
        usart1.cr1.write(|w| {
            w.ue()
                .set()
                .re()
                .set()
                .te()
                .set()
                .m()
                .clear()
                .pce()
                .clear()
                .rxneie()
                .set()
        });
    }

    /// Reads a byte from the RX buffer
    ///
    /// Returns `Err` if the RX buffer is empty
    pub fn read(&self) -> Result<u8> {
        let usart1 = self.0;

        if usart1.sr.read().rxne().is_set() {
            // NOTE(read_volatile) the register is 9 bits big but we'll only
            // work with the first 8 bits
            Ok(unsafe {
                ptr::read_volatile(&usart1.dr as *const _ as *const u8)
            })
        } else {
            Err(Error { _0: () })
        }
    }

    /// Writes byte into the TX buffer
    ///
    /// Returns `Err` if the TX buffer is already full
    pub fn write(&self, byte: u8) -> Result<()> {
        let usart1 = self.0;

        if usart1.sr.read().txe().is_set() {
            // NOTE(write_volatile) see NOTE in the `read` method
            unsafe {
                ptr::write_volatile(&usart1.dr as *const _ as *mut u8, byte)
            }
            Ok(())
        } else {
            Err(Error { _0: () })
        }
    }
}

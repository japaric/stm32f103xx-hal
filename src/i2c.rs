//! Inter-Integrated Circuit (I2C) bus

use afio::MAPR;
use gpio::{Alternate, OpenDrain};
use gpio::gpiob::{PB10, PB11, PB6, PB7, PB8, PB9};
use hal::blocking::i2c::{Read, Write, WriteRead};
use rcc::{APB1, Clocks};
use stm32f103xx::{I2C1, I2C2};

/// I2C error
#[derive(Debug)]
pub enum Error {
    /// Timed out busy waiting
    Timeout,
    /// Bus error
    Bus,
    /// Arbitration loss
    Arbitration,
    /// No ack received
    Acknowledge,
    /// Overrun/underrun
    Overrun,
    // Pec, // SMBUS mode only
    // Timeout, // SMBUS mode only
    // Alert, // SMBUS mode only
    #[doc(hidden)] _Extensible,
}

#[derive(Debug)]
pub enum DutyCycle {
    Ratio1to1,
    Ratio16to9,
}

#[derive(Debug)]
pub enum Mode {
    Standard { frequency: u32 },
    Fast { frequency: u32, duty_cycle: DutyCycle },
}

impl Mode {
    pub fn get_frequency(&self) -> u32 {
        match self {
            &Mode::Standard { frequency } => frequency,
            &Mode::Fast { frequency, .. } => frequency,
        }
    }
}


pub trait Pins<I2C> {
    const REMAP: bool;
}

impl Pins<I2C1>
for (
    PB6<Alternate<OpenDrain>>,
    PB7<Alternate<OpenDrain>>,
) {
    const REMAP: bool = false;
}

impl Pins<I2C1>
for (
    PB8<Alternate<OpenDrain>>,
    PB9<Alternate<OpenDrain>>,
) {
    const REMAP: bool = true;
}

impl Pins<I2C2>
for (
    PB10<Alternate<OpenDrain>>,
    PB11<Alternate<OpenDrain>>,
) {
    const REMAP: bool = false;
}

/// I2C peripheral operating in master mode
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
    bus_timeout: u32,
    mode: Mode,
    pclk1: u32,
}

impl<PINS> I2c<I2C1, PINS> {
    pub fn i2c1(
        i2c: I2C1,
        pins: PINS,
        mapr: &mut MAPR,
        mode: Mode,
        clocks: Clocks,
        apb: &mut APB1,
    ) -> Self
        where
            PINS: Pins<I2C1>,
    {
        mapr.mapr().modify(|_, w| w.i2c1_remap().bit(PINS::REMAP));
        I2c::_i2c1(i2c, pins, mode, clocks, apb)
    }
}

impl<PINS> I2c<I2C2, PINS> {
    pub fn i2c2(
        i2c: I2C2,
        pins: PINS,
        mode: Mode,
        clocks: Clocks,
        apb: &mut APB1,
    ) -> Self
        where
            PINS: Pins<I2C2>,
    {
        I2c::_i2c2(i2c, pins, mode, clocks, apb)
    }
}


macro_rules! busy_wait {
    ($cycles:expr, $i2c:expr, $flag:ident) => {
        {
            let mut cycles_left = $cycles;
            loop {
                if cycles_left == 0 {
                    break Err(Error::Timeout)
                } else {
                    cycles_left -= 1;
                }

                let sr1 = $i2c.sr1.read();

                if sr1.berr().bit_is_set() {
                    break Err(Error::Bus);
                } else if sr1.arlo().bit_is_set() {
                    break Err(Error::Arbitration);
                } else if sr1.af().bit_is_set() {
                    break Err(Error::Acknowledge);
                } else if sr1.ovr().bit_is_set() {
                    break Err(Error::Overrun);
                } else if sr1.$flag().bit_is_set() {
                    break Ok(());
                } else {
                    // try again
                }
            }
        }
    }
}

macro_rules! hal {
    ($($I2CX:ident: ($i2cX:ident, $i2cXen:ident, $i2cXrst:ident),)+) => {
        $(
            impl<PINS> I2c<$I2CX, PINS> {
                /// Configures the I2C peripheral to work in master mode
                pub fn $i2cX(
                    i2c: $I2CX,
                    pins: PINS,
                    mode: Mode,
                    clocks: Clocks,
                    apb: &mut APB1,
                ) -> Self {
                    apb.enr().modify(|_, w| w.$i2cXen().enabled());
                    apb.rstr().modify(|_, w| w.$i2cXrst().set_bit());
                    apb.rstr().modify(|_, w| w.$i2cXrst().clear_bit());

                    let bus_timeout = (clocks.sysclk().0 / 400).max(10);
                    let pclk1 = clocks.pclk1().0;

                    assert!(mode.get_frequency() <= 400_000);

                    let mut i2c = I2c { i2c, pins, bus_timeout, mode, pclk1 };
                    i2c.init();
                    i2c
                }

                fn init(&mut self) {
                    let freq = self.mode.get_frequency();
                    let pclk1_mhz = (self.pclk1 / 1000000) as u16;

                    // 25 ms bus timeout @ ~10 clks per busy_wait cycle

                    self.i2c.cr2.write(|w| unsafe {
                        w.freq().bits(pclk1_mhz as u8)
                    });
                    self.i2c.cr1.write(|w| w.pe().clear_bit());

                    match self.mode {
                        Mode::Standard { .. } => {
                            self.i2c.trise.write(|w| unsafe {
                                w.trise().bits((pclk1_mhz + 1) as u8)
                            });
                            self.i2c.ccr.write(|w| unsafe {
                                w.ccr().bits(((self.pclk1 / (freq * 2)) as u16).max(4))
                            });
                        },
                        Mode::Fast { ref duty_cycle, .. } => {
                            self.i2c.trise.write(|w| unsafe {
                                w.trise().bits((pclk1_mhz * 300 / 1000 + 1) as u8)
                            });

                            self.i2c.ccr.write(|w| {
                                let (freq, duty) = match duty_cycle {
                                    &DutyCycle::Ratio1to1 => (((self.pclk1 / (freq * 3)) as u16).max(1), false),
                                    &DutyCycle::Ratio16to9 => (((self.pclk1 / (freq * 25)) as u16).max(1), true)
                                };

                                unsafe {
                                    w.ccr().bits(freq).duty().bit(duty).f_s().set_bit()
                                }
                            });
                        }
                    };

                    self.i2c.cr1.modify(|_, w| w.pe().set_bit());
                }

                fn reset(&mut self) {
                    self.i2c.cr1.write(|w| w.pe().set_bit().swrst().set_bit());
                    self.i2c.cr1.reset();
                    self.init();
                }

                fn send_start_and_wait(&mut self) -> Result<(), Error> {
                    // According to http://www.st.com/content/ccc/resource/technical/document/errata_sheet/f5/50/c9/46/56/db/4a/f6/CD00197763.pdf/files/CD00197763.pdf/jcr:content/translations/en.CD00197763.pdf
                    // 2.14.4 Wrong behavior of I2C peripheral in master mode after a misplaced Stop
                    let mut retries_left = 3;
                    let mut last_ret: Result<(), Error> = Err(Error::Timeout);
                    while retries_left > 0 {
                        self.i2c.cr1.modify(|_, w| w.start().set_bit());
                        last_ret = busy_wait!(self.bus_timeout, self.i2c, sb);
                        if let Err(_) = last_ret {
                            self.reset();
                        } else {
                            break;
                        }
                        retries_left -= 1;
                    }
                    last_ret
                }

                fn send_addr(&self, addr: u8, read: bool) {
                    self.i2c.dr.write(|w| unsafe { w.dr().bits(addr << 1 | (if read {1} else {0})) });
                }

                fn send_addr_and_wait(&self, addr: u8, read: bool) -> Result<(), Error> {
                    self.send_addr(addr, read);
                    busy_wait!(self.bus_timeout, self.i2c, addr)?;
                    self.i2c.sr2.read();

                    Ok(())
                }

                fn send_stop(&self) -> Result<(), Error> {
                    self.i2c.cr1.modify(|_, w| w.stop().set_bit());

                    Ok(())
                }

                /// Releases the I2C peripheral and associated pins
                pub fn free(self) -> ($I2CX, PINS) {
                    (self.i2c, self.pins)
                }

                fn write_without_stop(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
                    self.send_start_and_wait()?;
                    self.send_addr_and_wait(addr, false)?;

                    for byte in bytes {
                        busy_wait!(self.bus_timeout, self.i2c, tx_e)?;
                        self.i2c.dr.write(|w| unsafe { w.dr().bits(*byte) });
                    }
                    busy_wait!(self.bus_timeout, self.i2c, tx_e)?;

                    Ok(())
                }
            }

            impl<PINS> Write for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
                    self.write_without_stop(addr, bytes)?;
                    self.send_stop()?;

                    Ok(())
                }
            }

            impl<PINS> Read for I2c<$I2CX, PINS> {
                type Error = Error;

                fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
                    self.send_start_and_wait()?;

                    match buffer.len() {
                        1 => {
                            self.send_addr(addr, true);
                            busy_wait!(self.bus_timeout, self.i2c, addr)?;
                            self.i2c.cr1.modify(|_, w| w.ack().clear_bit());
                            let _ = self.i2c.sr2.read();
                            self.send_stop()?;

                            busy_wait!(self.bus_timeout, self.i2c, rx_ne)?;
                            buffer[0] = self.i2c.dr.read().dr().bits();
                        },
                        2 => {
                            self.i2c.cr1.modify(|_, w| w.pos().set_bit().ack().set_bit());
                            self.send_addr_and_wait(addr, true)?;
                            self.i2c.cr1.modify(|_, w| w.pos().clear_bit().ack().clear_bit());

                            busy_wait!(self.bus_timeout, self.i2c, btf)?;
                            self.send_stop()?;
                            buffer[0] = self.i2c.dr.read().dr().bits();
                            buffer[1] = self.i2c.dr.read().dr().bits();
                        },
                        buffer_len => {
                            self.i2c.cr1.modify(|_, w| w.ack().set_bit());
                            self.send_addr_and_wait(addr, true)?;

                            let (mut first_bytes, mut last_two_bytes) = buffer.split_at_mut(buffer_len - 3);
                            for mut byte in first_bytes {
                                self.i2c.cr1.modify(|_, w| w.ack().set_bit());
                                busy_wait!(self.bus_timeout, self.i2c, rx_ne)?;
                                *byte = self.i2c.dr.read().dr().bits();
                            }

                            busy_wait!(self.bus_timeout, self.i2c, btf)?;
                            self.i2c.cr1.modify(|_, w| w.ack().clear_bit());
                            last_two_bytes[0] = self.i2c.dr.read().dr().bits();
                            self.send_stop()?;
                            last_two_bytes[1] = self.i2c.dr.read().dr().bits();
                            busy_wait!(self.bus_timeout, self.i2c, rx_ne)?;
                            last_two_bytes[2] = self.i2c.dr.read().dr().bits();
                        }
                    }

                    Ok(())
                }
            }

            impl<PINS> WriteRead for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write_read(
                    &mut self,
                    addr: u8,
                    bytes: &[u8],
                    buffer: &mut [u8],
                ) -> Result<(), Error> {
                    assert!(buffer.len() > 0);

                    if bytes.len() != 0 {
                        self.write_without_stop(addr, bytes)?;
                    }

                    self.read(addr, buffer)?;

                    Ok(())
                }
            }
        )+
    }
}

hal! {
    I2C1: (_i2c1, i2c1en, i2c1rst),
    I2C2: (_i2c2, i2c2en, i2c2rst),
}

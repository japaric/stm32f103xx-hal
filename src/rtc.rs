//! Real Time Clock

use stm32f103xx::{RCC, PWR, RTC};

/// RTC clock source
#[derive(Clone, Copy, Debug)]
pub enum RtcClkSource {
    /// No clock
    NoClock = 0b00,
    /// LSE oscillator clock (32.768 kHz), 
    LSE = 0b01, 
    /// LSI oscillator clock (~ 40kHz)
    LSI = 0b10,
    /// HSE oscillator clock (4-16 MHz) divided by 128 
    HSE = 0b11,
}

/// RTC interrupt event
pub enum RtcEvent {
    /// "Second" Tick Interrupt
    Second,
    /// Alarm
    Alarm,
    /// Overflow
    Overflow,
}

/// RTC
pub struct Rtc<'a>(pub &'a RTC);

impl<'a> Rtc<'a> {
    /// Initializes RTC
    pub fn init(&self, clk_source: RtcClkSource, prescalar: u32, rcc: &RCC, pwr: &PWR) {
        let rtc = self.0;

        // Enable backup and power interface clocks
        rcc.apb1enr.modify(|_, w| w.bkpen().enabled().pwren().enabled());

        // unprotect backup and RTC registers
        pwr.cr.modify(|_, w| w.dbp().set_bit());

        // Reset backup domain to allow changing clock source
        rcc.bdcr.modify(|_, w| w.bdrst().set_bit());
        rcc.bdcr.modify(|_, w| w.bdrst().clear_bit());

        match clk_source {
            // Turn on clock source and wait for it to be ready
            RtcClkSource::LSE => {
                rcc.bdcr.modify(|_, w| w.lseon().set_bit());
                while rcc.bdcr.read().lserdy().bit_is_clear() { };
            },
            RtcClkSource::LSI => {
                rcc.csr.modify(|_, w| w.lsion().set_bit());
                while rcc.csr.read().lsirdy().bit_is_clear() { };
            },
            RtcClkSource::HSE => {
                rcc.cr.modify(|_, w| w.hseon().set_bit());
                while rcc.cr.read().hserdy().bit_is_clear() { };
            },
            RtcClkSource::NoClock => { }
        }

        // NOTE once set, the clock source cannot be changed until the backup domain is reset
        rcc.bdcr.modify(|_, w| unsafe { w.rtcsel().bits(clk_source as u8) });

        // enable RTC clock
        rcc.bdcr.modify(|_, w| w.rtcen().set_bit());

        // wait for RTC register sync
        while rtc.crl.read().rsf().bit_is_clear() { };

        while rtc.crl.read().rtoff().bit_is_clear() { };
        // enter config mode
        rtc.crl.modify(|_, w| w.cnf().set_bit());

        rtc.prlh.write(|w| unsafe { w.bits(prescalar >> 16) });
        rtc.prll.write(|w| unsafe { w.bits(prescalar) });

        // reset counter value
        rtc.cnth.write(|w| unsafe { w.bits(0 >> 16) });
        rtc.cntl.write(|w| unsafe { w.bits(0) });

        // leave config mode
        rtc.crl.modify(|_, w| w.cnf().clear_bit());
        while rtc.crl.read().rtoff().bit_is_clear() { };
    }

    /// Starts listening for an interrupt `event`
    pub fn listen(&self, event: RtcEvent) {
        let rtc = self.0;

        // cannot write to RTC_CRH when RTOFF = 0
        while rtc.crl.read().rtoff().bit_is_clear() { };
        match event {
            RtcEvent::Second => rtc.crh.modify(|_, w| w.secie().set_bit()),
            RtcEvent::Alarm => rtc.crh.modify(|_, w| w.alrie().set_bit()),
            RtcEvent::Overflow => rtc.crh.modify(|_, w| w.owie().set_bit()),
        }
    }

    /// Stops listening for an interrupt `event`
    pub fn unlisten(&self, event: RtcEvent) {
        let rtc = self.0;

        // cannot write to RTC_CRH when RTOFF = 0
        while rtc.crl.read().rtoff().bit_is_clear() { };
        match event {
            RtcEvent::Second => rtc.crh.modify(|_, w| w.secie().clear_bit()),
            RtcEvent::Alarm => rtc.crh.modify(|_, w| w.alrie().clear_bit()),
            RtcEvent::Overflow => rtc.crh.modify(|_, w| w.owie().clear_bit()),
        }
    }

    /// Clear interrupt `event` flag
    pub fn clear_flag(&self, event: RtcEvent) {
        let rtc = self.0;

        match event {
            RtcEvent::Second => rtc.crl.modify(|_, w| w.secf().clear_bit()),
            RtcEvent::Alarm => rtc.crl.modify(|_, w| w.alrf().clear_bit()),
            RtcEvent::Overflow => rtc.crl.modify(|_, w| w.owf().clear_bit()),
        }
    }
}

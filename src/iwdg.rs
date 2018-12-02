//! Independent Watchdog

use stm32f103xx::IWDG;
use hal::watchdog::{Watchdog, WatchdogEnable};

use rcc::CSR;


pub struct IndependentWatchdog {
    iwdg: IWDG,
}

enum Key {
    Feed = 0xAAAA,
    Unlock = 0x5555,
    Lock = 0x0,
    Start = 0xCCCC,
}

impl IndependentWatchdog {
    /// Wrap the watchdog peripheral into a struct that implements the
    /// embedded_hal `Watchdog` and `WatchdogEnable` traits.
    ///
    /// Pass a `rcc.csr` to initialize the LSI clock.
    pub fn new(iwdg: IWDG, csr: &mut CSR) -> Self {
        csr.enable_lsi();

        IndependentWatchdog { iwdg }
    }

    fn write_key(&self, key: Key) {
        self.iwdg.kr.write(|w| unsafe { w.key().bits(key as u16) });
    }

    fn unlocked<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut IWDG) -> R,
    {
        self.write_key(Key::Unlock);
        let r = f(&mut self.iwdg);
        self.write_key(Key::Lock);
        r
    }

    fn configure(&mut self, prescaler: u8, reload: u16) {
        self.unlocked(|iwdg| {
            iwdg.pr.write(|w| unsafe {
                w.pr().bits(prescaler)
            });
            iwdg.rlr.write(|w| unsafe {
                w.rl().bits(reload)
            });
        });
    }

    /// Maximum watchdog timeout is 26.214 seconds
    pub const MAX_TIMEOUT_MS: u32 = 26214;

    const MAX_RELOAD: u16 = 0xFFF;
    const MAX_PRESCALE: u8 = 6;
}

impl WatchdogEnable for IndependentWatchdog {
    type Time = u32;

    fn start<T>(&mut self, period: T) where T: Into<Self::Time> {
        let timeout_ms = period.into();

        let mut max_timeout_ms = Self::MAX_TIMEOUT_MS;
        assert!(timeout_ms <= max_timeout_ms);

        let mut prescaler = Self::MAX_PRESCALE;
        while prescaler > 0 && timeout_ms <= max_timeout_ms / 2 {
            prescaler -= 1;
            max_timeout_ms /= 2;
        }

        let reload = ((Self::MAX_RELOAD as u32) * timeout_ms / max_timeout_ms) as u16;
        self.configure(prescaler, reload);
        self.write_key(Key::Start);
    }
}

impl Watchdog for IndependentWatchdog {
    fn feed(&mut self) {
        self.write_key(Key::Feed);
    }
}

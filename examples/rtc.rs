
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

extern crate panic_halt;

extern crate cortex_m_semihosting as sh;
use stm32f103xx_hal::prelude::*;
use stm32f103xx_hal::rtc::Rtc;

use core::fmt::Write;

use sh::hio;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let mut hstdout = hio::hstdout().unwrap();

    let mut p = stm32f103xx::Peripherals::take().unwrap();

    let mut pwr = p.PWR;
    // Enable the clocks in the backup domain
    let bd_token = p.RCC.enable_backup_domain(&mut pwr);
    let rcc = p.RCC.constrain();

    let rtc = Rtc::rtc(p.RTC, &rcc, &bd_token);

    loop {
        writeln!(hstdout, "time: {}", rtc.read()).unwrap();
    }
}

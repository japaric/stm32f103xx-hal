//! "Blinky" using delays instead of a timer

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use stm32f103xx_hal::{
    prelude::*,
    device,
    delay::Delay,
};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let dp = device::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        led.set_high();
        delay.delay_ms(1_000_u16);
        led.set_low();
        delay.delay_ms(1_000_u16);
    }
}

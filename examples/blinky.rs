//! Blinks an LED

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m;
extern crate panic_abort;
extern crate stm32f103xx_hal as hal;
#[macro_use(block)]
extern crate nb;

use hal::prelude::*;
use hal::stm32f103xx;
use hal::timer::Timer;

fn main() {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f103xx::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Try a different clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let clocks = rcc.cfgr
    //     .sysclk(64.mhz())
    //     .pclk1(32.mhz())
    //     .freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Try a different timer (even SYST)
    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);
    loop {
        block!(timer.wait()).unwrap();
        led.set_high();
        block!(timer.wait()).unwrap();
        led.set_low();
    }
}

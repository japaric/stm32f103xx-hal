//! Turns the user LED on

#![no_std]

extern crate blue_pill;
extern crate cortex_m;
extern crate cortex_m_semihosting as semihosting;

use core::fmt::Write;

use blue_pill::hal::prelude::*;
use blue_pill::hal::delay::Delay;
use blue_pill::hal::qei::Qei;
use semihosting::hio;

fn main() {
    let dp = blue_pill::hal::stm32f103xx::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // let gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // TIM2
    // let c1 = gpioa.pa0;
    // let c2 = gpioa.pa1;

    // TIM3
    // let c1 = gpioa.pa6;
    // let c2 = gpioa.pa7;

    // TIM4
    let c1 = gpiob.pb6;
    let c2 = gpiob.pb7;

    let qei = Qei::tim4(dp.TIM4, (c1, c2), &mut afio.mapr, &mut rcc.apb1);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut hstdout = hio::hstdout().unwrap();
    loop {
        let before = qei.count();
        delay.delay_ms(1_000_u16);
        let after = qei.count();

        let elapsed = after.wrapping_sub(before) as i16;

        writeln!(hstdout, "{}", elapsed).unwrap();
    }
}

//! Turns the user LED on

#![no_std]

extern crate blue_pill;

use blue_pill::hal::prelude::*;

fn main() {
    let p = blue_pill::hal::stm32f103xx::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let mut gpioc = p.GPIOC.split(&mut rcc.apb2);

    gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
}

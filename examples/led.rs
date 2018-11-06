//! Turns the user LED on

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;
extern crate cortex_m_rt as rt;

use stm32f103xx_hal::{
    prelude::*,
    device,
};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let p = device::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let mut gpioc = p.GPIOC.split(&mut rcc.apb2);

    gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    loop {}
}

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate stm32f103xx_hal;

fn main() {
    let p = cortex_m::Peripherals::take().unwrap();
    let mut itm = p.ITM;

    iprintln!(&mut itm.stim[0], "Hello, world!");
}

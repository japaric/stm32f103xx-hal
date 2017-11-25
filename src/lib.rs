#![feature(never_type)]
#![feature(unsize)]
#![no_std]

extern crate cast;
extern crate nb;
pub extern crate embedded_hal as hal;
pub extern crate stm32f103xx;

mod bb;
pub mod adc;
pub mod afio;
pub mod capture;
pub mod dma;
pub mod flash;
pub mod gpio;
pub mod prelude;
pub mod pwm;
pub mod rcc;
pub mod serial;
pub mod time;
pub mod timer;

pub use timer::Timer;
pub use serial::Serial;

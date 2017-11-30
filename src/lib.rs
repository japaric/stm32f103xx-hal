#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(never_type)]
#![feature(unsize)]
#![no_std]

extern crate cast;
#[macro_use]
extern crate cortex_m;
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
pub mod spi;
pub mod time;
pub mod timer;

use cortex_m::itm;

pub use serial::Serial;
pub use spi::Spi;
pub use timer::Timer;

// TODO remove
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    args: ::core::fmt::Arguments,
    file: &'static str,
    line: u32,
    col: u32,
) -> ! {
    let itm = &*cortex_m::peripheral::ITM::ptr();

    itm::write_str(&itm.stim[0], "panicked at '");
    itm::write_fmt(&itm.stim[0], args);
    iprintln!(&itm.stim[0], "', {}:{}:{}", file, line, col);

    ::core::intrinsics::abort()
}

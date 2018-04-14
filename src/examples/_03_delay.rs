//! "Blinky" using delays instead of a timer
//!
//! ```
//! 
//! #![deny(unsafe_code)]
//! #![deny(warnings)]
//! #![no_std]
//! 
//! extern crate cortex_m;
//! extern crate panic_abort;
//! extern crate stm32f103xx_hal as hal;
//! 
//! use hal::delay::Delay;
//! use hal::prelude::*;
//! use hal::stm32f103xx;
//! 
//! fn main() {
//!     let dp = stm32f103xx::Peripherals::take().unwrap();
//!     let cp = cortex_m::Peripherals::take().unwrap();
//! 
//!     let mut flash = dp.FLASH.constrain();
//!     let mut rcc = dp.RCC.constrain();
//! 
//!     let clocks = rcc.cfgr.freeze(&mut flash.acr);
//! 
//!     let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
//! 
//!     let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
//!     let mut delay = Delay::new(cp.SYST, clocks);
//! 
//!     loop {
//!         led.set_high();
//!         delay.delay_ms(1_000_u16);
//!         led.set_low();
//!         delay.delay_ms(1_000_u16);
//!     }
//! }
//! ```
// Auto-generated. Do not modify.

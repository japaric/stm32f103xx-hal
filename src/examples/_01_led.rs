//! Turns the user LED on
//!
//! ```
//! 
//! #![deny(unsafe_code)]
//! #![deny(warnings)]
//! #![no_std]
//! 
//! extern crate panic_abort;
//! extern crate stm32f103xx_hal as hal;
//! 
//! use hal::prelude::*;
//! use hal::stm32f103xx;
//! 
//! fn main() {
//!     let p = stm32f103xx::Peripherals::take().unwrap();
//! 
//!     let mut rcc = p.RCC.constrain();
//!     let mut gpioc = p.GPIOC.split(&mut rcc.apb2);
//! 
//!     gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
//! }
//! ```
// Auto-generated. Do not modify.

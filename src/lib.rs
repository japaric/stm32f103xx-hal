//! Board Support Crate for the [Blue Pill]
//!
//! [Blue Pill]: http://wiki.stm32duino.com/index.php?title=Blue_Pill
//!
//! # Usage
//!
//! Follow `cortex-m-quickstart` [instructions][i] but remove the `memory.x`
//! linker script and the `build.rs` build script file as part of the
//! configuration of the quickstart crate.
//!
//! [i]: https://docs.rs/cortex-m-quickstart/0.1.8/cortex_m_quickstart/

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(get_type_id)]
#![feature(never_type)]
#![no_std]

extern crate cast;
extern crate either;
extern crate nb;

pub extern crate stm32f103xx;

mod frequency;

pub mod capture;
pub mod gpio;
pub mod led;
pub mod pwm;
pub mod qei;
pub mod serial;
pub mod spi;
pub mod timer;

pub use capture::Capture;
pub use pwm::Pwm;
pub use qei::Qei;
pub use serial::Serial;
pub use spi::Spi;
pub use timer::{Channel, Timer};

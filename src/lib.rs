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
//! [i]: https://docs.rs/cortex-m-quickstart/0.1.2/cortex_m_quickstart/

#![deny(missing_docs)]
#![no_std]

extern crate cast;

pub extern crate stm32f103xx;

mod frequency;

pub mod led;
pub mod serial;
pub mod timer;

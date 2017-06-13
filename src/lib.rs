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
#![feature(const_fn)]
#![feature(get_type_id)]
#![feature(never_type)]
#![no_std]

extern crate cast;
extern crate either;
extern crate embedded_hal as hal;
extern crate nb;
extern crate static_ref;

pub extern crate stm32f103xx;

pub mod capture;
pub mod dma;
pub mod gpio;
pub mod led;
pub mod pwm;
pub mod qei;
pub mod serial;
pub mod spi;
pub mod time;
pub mod timer;

pub use capture::Capture;
pub use pwm::Pwm;
pub use qei::Qei;
pub use serial::Serial;
pub use spi::Spi;
pub use timer::{Channel, Timer};

macro_rules! frequency {
    ($FREQUENCY:expr) => {
        use time::*;

        /// Frequency
        pub const FREQUENCY: u32 = $FREQUENCY;

        /// Unit of time
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct Ticks(pub u32);

        impl Ticks {
            /// Applies the function `f` to the inner value
            pub fn map<F>(self, f: F) -> Self
                where F: FnOnce(u32) -> u32,
            {
                Ticks(f(self.0))
            }
        }

        impl From<Ticks> for Microseconds {
            fn from(ticks: Ticks) -> Self {
                Microseconds(ticks.0 / (FREQUENCY / 1_000_000))
            }
        }

        impl From<Ticks> for Milliseconds {
            fn from(ticks: Ticks) -> Self {
                Milliseconds(ticks.0 / (FREQUENCY / 1_000))
            }
        }

        impl From<Ticks> for Seconds {
            fn from(ticks: Ticks) -> Self {
                Seconds(ticks.0 / FREQUENCY)
            }
        }

        impl From<IHertz> for Ticks {
            fn from(ihz: IHertz) -> Ticks {
                Ticks(FREQUENCY / ihz.0)
            }
        }

        impl From<Microseconds> for Ticks {
            fn from(us: Microseconds) -> Ticks {
                Ticks(us.0 * (FREQUENCY / 1_000_000))
            }
        }

        impl From<Milliseconds> for Ticks {
            fn from(ms: Milliseconds) -> Ticks {
                Ticks(ms.0 * (FREQUENCY / 1_000))
            }
        }

        impl From<Seconds> for Ticks {
            fn from(s: Seconds) -> Ticks {
                Ticks(s.0 * FREQUENCY)
            }
        }

        impl Into<u32> for Ticks {
            fn into(self) -> u32 {
                self.0
            }
        }
    }
}

/// Advance High-performance Bus (AHB)
pub mod ahb {
    frequency!(8_000_000);
}

/// Advance Peripheral Bus 1 (APB1)
pub mod apb1 {
    frequency!(8_000_000);
}

/// Advance Peripheral Bus 2 (APB2)
pub mod apb2 {
    frequency!(8_000_000);
}

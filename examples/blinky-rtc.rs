//! Blinks the user LED using RTC
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::Rtc;
use blue_pill::rtc::{RtcClkSource, RtcEvent};
use blue_pill::led::{self, Green};
use blue_pill::stm32f103xx::Interrupt;
use rtfm::{app, Threshold};

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static ON: bool = false;
    },

    tasks: {
        RTC: {
            path: toggle,
            resources: [ON, RTC],
        },
    },
}

fn init(p: init::Peripherals, _r: init::Resources) {
    led::init(p.GPIOC, p.RCC);

    let rtc = Rtc(p.RTC);

    // ~40kHz clock
    rtc.init(RtcClkSource::LSI, 40000, p.RCC, p.PWR);
    rtc.listen(RtcEvent::Second);
    p.NVIC.enable(Interrupt::RTC);
}

fn idle() -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
fn toggle(_t: &mut Threshold, r: RTC::Resources) {
    let rtc = Rtc(&**r.RTC);
    rtc.clear_flag(RtcEvent::Second);

    **r.ON = !**r.ON;

    if **r.ON {
        Green.on();
    } else {
        Green.off();
    }
}

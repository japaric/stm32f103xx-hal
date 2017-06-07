//! Output a PWM with a duty cycle of 6% on all the channels

#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate blue_pill;

use blue_pill::{Channel, Pwm, stm32f103xx};
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const FREQUENCY: u32 = 1_000; // Hz

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOB: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM4: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpiob = &GPIOB.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim4 = TIM4.access(prio, thr);

    let pwm = Pwm(&*tim4);

    pwm.init(FREQUENCY, afio, gpiob, rcc);
    let duty = pwm.get_period() / 16;

    pwm.set_duty(Channel::_1, duty);
    pwm.set_duty(Channel::_2, duty);
    pwm.set_duty(Channel::_3, duty);
    pwm.set_duty(Channel::_4, duty);

    rtfm::bkpt();

    pwm.on(Channel::_1);
    pwm.on(Channel::_2);
    pwm.on(Channel::_3);
    pwm.on(Channel::_4);
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {});

//! Output a PWM on pin PA0 and control its duty cycle via a serial interface
//!
//! - '*' increase duty by a factor of 2
//! - '+' increase duty by 1
//! - '-' decrease duty by 1
//! - '/' decrease duty by a factor of 2

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

use core::u16;

use blue_pill::pwm::Pwm;
use blue_pill::serial::Serial;
use blue_pill::stm32f103xx;
use blue_pill::timer::Channel;
use rtfm::{P0, P1, T0, T1, TMax};
use stm32f103xx::interrupt::USART1;

// CONFIGURATION
const BAUD_RATE: u32 = 115_200; // bps
const FREQUENCY: u32 = 1_000; // Hz

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOA: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM2: Peripheral {
        ceiling: C1,
    },
    USART1: Peripheral {
        ceiling: C1,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim2 = TIM2.access(prio, thr);
    let usart1 = USART1.access(prio, thr);

    let pwm = Pwm(&*tim2);
    let serial = Serial(&*usart1);

    serial.init(BAUD_RATE, afio, gpioa, rcc);

    pwm.init(FREQUENCY, afio, gpioa, rcc);
    pwm.set_duty(Channel::_1, 0);

    pwm.on(Channel::_1);
}

// IDLE LOOP
fn idle(_prio: P0, _thr: T0) -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
tasks!(stm32f103xx, {
    rx: Task {
        interrupt: USART1,
        priority: P1,
        enabled: true,
    },
});

fn rx(_task: USART1, ref prio: P1, ref thr: T1) {
    let tim2 = TIM2.access(prio, thr);
    let usart1 = USART1.access(prio, thr);

    let pwm = Pwm(&*tim2);
    let serial = Serial(&*usart1);

    if let Ok(byte) = serial.read() {
        // Echo back to signal we are alive
        if serial.write(byte).is_err() {
            // NOTE Impossible to override the TX buffer
            unreachable!()
        }

        match byte {
            b'+' | b'-' | b'*' | b'/' => {
                let duty = pwm.get_duty(Channel::_1);

                match byte {
                    b'+' => {
                        let period = pwm.get_period();
                        pwm.set_duty(
                            Channel::_1,
                            if duty < period { duty + 1 } else { period },
                        );
                    }
                    b'-' => {
                        pwm.set_duty(
                            Channel::_1,
                            duty.checked_sub(1).unwrap_or(0),
                        );
                    }
                    b'*' => {
                        let new_duty = duty.checked_mul(2).unwrap_or(u16::MAX);
                        let period = pwm.get_period();

                        if new_duty < period {
                            pwm.set_duty(Channel::_1, new_duty)
                        }
                    }
                    b'/' => pwm.set_duty(Channel::_1, duty / 2),
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
    } else {
        // NOTE Can only be reached via `rtfm::request(rx)`
        unreachable!()
    }
}

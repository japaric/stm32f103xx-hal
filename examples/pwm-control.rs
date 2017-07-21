//! Output a PWM on pin PA0 and control its duty cycle via a serial interface
//!
//! - '*' increase duty by a factor of 2
//! - '+' increase duty by 1
//! - '-' decrease duty by 1
//! - '/' decrease duty by a factor of 2

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;

use core::u16;

use blue_pill::prelude::*;
use blue_pill::time::Hertz;
use blue_pill::{Channel, Pwm, Serial};
use rtfm::{app, Threshold};

const BAUD_RATE: Hertz = Hertz(115_200);
const FREQUENCY: Hertz = Hertz(1_000);

app! {
    device: blue_pill::stm32f103xx,

    tasks: {
        USART1: {
            enabled: true,
            priority: 1,
            resources: [TIM2, USART1],
        },
    },
}

fn init(p: init::Peripherals) {
    let pwm = Pwm(p.TIM2);
    let serial = Serial(p.USART1);

    serial.init(BAUD_RATE.invert(), p.AFIO, None, p.GPIOA, p.RCC);

    pwm.init(FREQUENCY.invert(), p.AFIO, None, p.GPIOA, p.RCC);
    pwm.set_duty(Channel::_1, 0);

    pwm.enable(Channel::_1);
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

task!(USART1, rx);

fn rx(_t: &mut Threshold, r: USART1::Resources) {
    let pwm = Pwm(&**r.TIM2);
    let serial = Serial(&**r.USART1);

    let byte = serial.read().unwrap();
    // Echo back to signal we are alive
    serial.write(byte).unwrap();

    match byte {
        b'+' | b'-' | b'*' | b'/' => {
            let duty = pwm.get_duty(Channel::_1);

            match byte {
                b'+' => {
                    let max = pwm.get_max_duty();
                    pwm.set_duty(
                        Channel::_1,
                        if duty < max { duty + 1 } else { max },
                    );
                }
                b'-' => {
                    pwm.set_duty(Channel::_1, duty.checked_sub(1).unwrap_or(0));
                }
                b'*' => {
                    let new_duty = duty.checked_mul(2).unwrap_or(u16::MAX);
                    let max_duty = pwm.get_max_duty();

                    if new_duty < max_duty {
                        pwm.set_duty(Channel::_1, new_duty)
                    }
                }
                b'/' => pwm.set_duty(Channel::_1, duty / 2),
                _ => { /* unreachable */ }
            }
        }
        _ => {}
    }
}

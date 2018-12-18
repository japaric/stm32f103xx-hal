//! Open loop motor control

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;
use cortex_m_semihosting::hprintln;

use nb::block;

use stm32f103xx_hal::{
    prelude::*,
    device,
    serial::Serial,
};
use motor_driver::Motor;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let p = device::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);

    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let serial = Serial::usart1(
        p.USART1,
        (tx, rx),
        &mut afio.mapr,
        115_200.bps(),
        clocks,
        &mut rcc.apb2,
    );

    let mut rx = serial.split().1;

    let pwm = p.TIM2.pwm(
        gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl),
        &mut afio.mapr,
        1.khz(),
        clocks,
        &mut rcc.apb1,
    );

    let max_duty = pwm.get_max_duty() as i16;
    let mut motor = Motor::tb6612fng(
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa2.into_push_pull_output(&mut gpioa.crl),
        pwm,
    );

    let mut duty = max_duty;
    let mut brake = true;

    motor.duty(duty as u16);

    hprintln!("{} {}", max_duty, brake).unwrap();
    loop {
        match block!(rx.read()).unwrap() {
            b'*' => duty *= 2,
            b'+' => duty += 1,
            b'-' => duty -= 1,
            b'/' => duty /= 2,
            b'r' => duty *= -1,
            b's' => brake = !brake,
            _ => continue,
        }

        if duty > max_duty {
            duty = max_duty;
        } else if duty < -max_duty {
            duty = -max_duty;
        }

        if brake {
            motor.brake();
        } else if duty > 0 {
            motor.cw();
        } else {
            motor.ccw();
        }

        motor.duty(duty.abs() as u16);

        hprintln!("{} {}", duty, brake).unwrap();
    }
}

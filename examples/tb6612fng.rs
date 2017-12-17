//! Drive an LED, connected to PA6, using PWM

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate tb6612fng;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use tb6612fng::Motor;
use rtfm::app;

app! {
    device: stm32f103xx,
}

fn init(p: init::Peripherals) {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa4 = gpioa.pa4.as_output(&mut gpioa.crl);
    let pa5 = gpioa.pa5.as_output(&mut gpioa.crl);
    let pa6 = gpioa.pa6.as_alt_push(&mut gpioa.crl);
    let t3c1 = p.device
        .TIM3
        .pwm(pa6, 10.khz(), clocks, &mut rcc.enr, &mut afio.mapr);

    let max_duty = t3c1.get_max_duty();

    let mut motor = Motor::new(pa4, pa5, t3c1);

    motor.speed(max_duty);

    motor.cw();

    rtfm::bkpt();

    motor.ccw();

    rtfm::bkpt();

    motor.brake();

    rtfm::bkpt();

    motor.ccw();

    rtfm::bkpt();

    motor.coast();

    rtfm::bkpt();

    motor.ccw();

    rtfm::bkpt();

    for i in (0..max_duty).into_iter().rev() {
        motor.speed(i);

        for _ in 0..1_000 {
            cortex_m::asm::nop();
        }
    }

    rtfm::bkpt();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

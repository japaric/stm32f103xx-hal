//! Drive an LED, connected to PA6, using PWM

#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use blue_pill::prelude::*;
use blue_pill::stm32f103xx;
use rtfm::app;

app! {
    device: stm32f103xx,
}

fn init(p: init::Peripherals) {
    let mut rcc = p.device.RCC.split();
    let mut afio = p.device.AFIO.split(&mut rcc.enr);
    let mut gpioa = p.device.GPIOA.split(&mut rcc.enr);
    let mut flash = p.device.FLASH.split();

    // try commenting out this line
    rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz());

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let pa6 = gpioa.pa6.as_alt_push(&mut gpioa.crl);
    let mut t3c1 = p.device
        .TIM3
        .pwm(pa6, 1.hz(), clocks, &mut rcc.enr, &mut afio.mapr);

    let max_duty = t3c1.get_max_duty();

    t3c1.enable();

    t3c1.set_duty(max_duty / 4);

    rtfm::bkpt();

    t3c1.set_duty(max_duty / 2);

    rtfm::bkpt();

    t3c1.set_duty((3 * max_duty as u32 / 4) as u16);

    rtfm::bkpt();

    t3c1.set_duty(max_duty);
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

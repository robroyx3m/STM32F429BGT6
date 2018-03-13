//! PWM test
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt6;

use stm32_f429_bgt6::prelude::*;
use stm32_f429_bgt6::time::Hertz;
use stm32_f429_bgt6::{Channel, Pwm};
use rtfm::app;

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

const FREQUENCY: Hertz = Hertz(1000);

app! {
    device: stm32f429x,
}

fn init(p: init::Peripherals) {
    let pwm = Pwm(p.TIM2);

    const CHANNELS: [Channel; 3] = [Channel::_1, Channel::_2, Channel::_3];

    for c in &CHANNELS {
        pwm.init(
            FREQUENCY.invert(),
            *c,
            None,
            p.GPIOA,
            p.GPIOB,
            p.GPIOC,
            p.RCC,
        );
        pwm.set_duty(*c, pwm.get_max_duty() / 16);
        pwm.enable(*c);
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

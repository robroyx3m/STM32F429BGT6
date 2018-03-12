//! Blocking version of blinky

#![allow(unreachable_code)] // for the `block!` macro
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt6 as f4;

#[macro_use(block)]
extern crate nb;

use f4::Timer;
use f4::led::{self, LED};
use f4::prelude::*;
use f4::time::Hertz;
use f4::clock;
use rtfm::{app, Threshold};

const FREQUENCY: Hertz = Hertz(1);

app! {
    device: f4::stm32f40x,

    idle: {
        resources: [TIM11],
    }
}

fn init(p: init::Peripherals) {
    // Set system clock in order to test that it works
    clock::set_84_mhz(&p.RCC, &p.FLASH);

    led::init(p.GPIOA, p.RCC);
    let timer = Timer(&*p.TIM11);

    timer.init(FREQUENCY.invert(), p.RCC);
}

fn idle(_t: &mut Threshold, r: idle::Resources) -> ! {
    let timer = Timer(&*r.TIM11);

    timer.resume();
    let mut state = false;
    loop {
        block!(timer.wait()).unwrap(); // NOTE(unwrap) E = !

        state = !state;

        if state {
            LED.on();
        } else {
            LED.off();
        }
    }
}

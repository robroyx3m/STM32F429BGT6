//! Turns all the user LEDs on
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt6;

use stm32_f429_bgt6::led::{self, LED};
use rtfm::app;

use stm32_f429_bgt6::stm32f40x as stm32f429x;

// TASKS & RESOURCES
app! {
    device: stm32f429x,
}

// INITIALIZATION PHASE
fn init(p: init::Peripherals) {
    led::init(&p.GPIOA, &p.RCC);
}

// IDLE LOOP
fn idle() -> ! {
    // Sleep
    loop {
        LED.on();
        rtfm::wfi();
        LED.off();
    }
}

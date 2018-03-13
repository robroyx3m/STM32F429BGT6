//! Prints "Hello, World" in the OpenOCD console
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
#[macro_use]
extern crate stm32_f429_bgt6;

use rtfm::app;

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

app! {
    device: stm32f429x,
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    let f = 1.0;
    println!("Hello, world! {}", f);

    rtfm::bkpt();
    loop {
        rtfm::wfi();
    }
}

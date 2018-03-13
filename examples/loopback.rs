//! Serial interface loopback
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt6;

use stm32_f429_bgt6::prelude::*;
use stm32_f429_bgt6::Serial;
use stm32_f429_bgt6::serial::Event;
use stm32_f429_bgt6::time::Hertz;
use rtfm::{app, Threshold};

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

// CONFIGURATION
const BAUD_RATE: Hertz = Hertz(115_200);

// TASKS & RESOURCES
app! {
    device: stm32f429x,

    tasks: {
        USART2: {
            path: loopback,
            resources: [USART2],
        },
    }
}

// INITIALIZATION PHASE
fn init(p: init::Peripherals) {
    let serial = Serial(p.USART2);

    serial.init(BAUD_RATE.invert(), Some(p.DMA1), p.GPIOA, p.RCC);
    serial.listen(Event::Rxne);
}

// IDLE LOOP
fn idle() -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
// Send back the received byte
fn loopback(_t: &mut Threshold, r: USART2::Resources) {
    let serial = Serial(&**r.USART2);

    let byte = serial.read().unwrap();
    serial.write(byte).unwrap();
}

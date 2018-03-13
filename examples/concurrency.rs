//! LED roulette and serial loopback running concurrently
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cast;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt;

use stm32_f429_bgt6::Serial;
use stm32_f429_bgt6::led::{self, LED};
use stm32_f429_bgt6::prelude::*;
use stm32_f429_bgt6::serial::Event;
use stm32_f429_bgt6::time::Hertz;
use cortex_m::peripheral::SystClkSource;
use rtfm::{app, Threshold};

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

// CONFIGURATION
const BAUD_RATE: Hertz = Hertz(115_200);
const DIVISOR: u32 = 4;

// TASKS & RESOURCES
app! {
    device: stm32f429x,

    resources: {
        static ON: bool = false;
    },

    tasks: {
        SYS_TICK: {
            path: roulette,
            resources: [ON],
        },

        USART2: {
            path: loopback,
            resources: [USART2],
        },
    }
}

// INITIALIZATION PHASE
fn init(p: init::Peripherals, _r: init::Resources) {
    led::init(p.GPIOA, p.RCC);

    let serial = Serial(p.USART2);
    serial.init(BAUD_RATE.invert(), Some(p.DMA1), p.GPIOA, p.RCC);
    serial.listen(Event::Rxne);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(16_000_000 / DIVISOR);
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

// IDLE LOOP
fn idle() -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

// TASKS
fn loopback(_t: &mut Threshold, r: USART2::Resources) {
    let serial = Serial(&**r.USART2);

    if let Ok(byte) = serial.read() {
        if serial.write(byte).is_err() {
            // As we are echoing the bytes as soon as they arrive, it should
            // be impossible to have a TX buffer overrun
            #[cfg(debug_assertions)]
            unreachable!()
        }
    } else {
        // Only reachable through `rtfm::request(loopback)`
        #[cfg(debug_assertions)]
        unreachable!()
    }
}

fn roulette(_t: &mut Threshold, r: SYS_TICK::Resources) {
    **r.ON = !**r.ON;

    if **r.ON {
        LED.on();
    } else {
        LED.off();
    }
}

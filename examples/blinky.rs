//! Blinks an LED
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt6;

use cortex_m::peripheral::SystClkSource;
use stm32_f429_bgt6::led::{self, LED};
use rtfm::{app, Threshold};

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

// CONFIGURATION
const FREQUENCY: u32 = 1; // Hz

// TASKS & RESOURCES
app! {
    device: stm32f429x,

    resources: {
        static ON: bool = false;
    },

    tasks: {
        SYS_TICK: {
            path: toggle,
            resources: [ON],
        },
    },
}

// INITIALIZATION PHASE
fn init(p: init::Peripherals, _r: init::Resources) {
    led::init(p.GPIOA, p.RCC);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(16_000_000 / FREQUENCY);
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
// Toggle the state of the LED
fn toggle(_t: &mut Threshold, r: SYS_TICK::Resources) {
    **r.ON = !**r.ON;

    if **r.ON {
        LED.on();
    } else {
        LED.off();
    }
}

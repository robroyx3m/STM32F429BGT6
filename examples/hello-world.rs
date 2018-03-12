//! Prints "Hello" and then "World" in the OpenOCD console
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting as semihosting;
extern crate stm32_f429_bgt6 as f4;

use core::fmt::Write;

use rtfm::{app, Threshold};
use semihosting::hio::{self, HStdout};

app! {
    device: f4::stm32f40x,

    resources: {
        static HSTDOUT: Option<HStdout> = None;
    },

    idle: {
        resources: [HSTDOUT],
    },
}

fn init(_p: init::Peripherals, r: init::Resources) {
    let mut hstdout = hio::hstdout().unwrap();

    writeln!(hstdout, "Hello").unwrap();

    **r.HSTDOUT = Some(hstdout);
}

fn idle(_t: &mut Threshold, r: idle::Resources) -> ! {
    writeln!(r.HSTDOUT.as_mut().unwrap(), "World").unwrap();

    loop {
        rtfm::wfi();
    }
}

//! Sends "Hello" and then "World" through the ITM port 0
//!
//! You'll need to run these lines in your GDB session
//!
//! ``` text
//! > monitor tpiu config internal /tmp/itm.log uart off 16000000 2000000
//! > monitor itm port 0 on
//! ```
//!
//! And connect the SWO (PB3) pin to an UART adapter, or read it by some other
//! means.
//!
//! Finally you should see the output if you monitor the UART adapter device
//! file.
//!
//! ``` console
//! $ itmdump /tmp/itm.log
//! Hello
//! World
//! ```
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

#[macro_use(iprint, iprintln)]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt6;

use rtfm::{app, Threshold};

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

app! {
    device: stm32f429x,

    idle: {
        resources: [ITM],
    },
}

fn init(p: init::Peripherals) {
    iprintln!(&p.ITM.stim[0], "Hello");
}

fn idle(_t: &mut Threshold, r: idle::Resources) -> ! {
    iprintln!(&r.ITM.stim[0], "World");

    // Sleep
    rtfm::bkpt();
    loop {
        rtfm::wfi();
    }
}

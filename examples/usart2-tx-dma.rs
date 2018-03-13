//! Test sending serial data using the DMA
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32_f429_bgt6;

use stm32_f429_bgt6::Serial;
use stm32_f429_bgt6::dma::{Buffer, Dma1Stream6};
use stm32_f429_bgt6::time::Hertz;
use rtfm::{app, Threshold};

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

const BAUD_RATE: Hertz = Hertz(115_200);

app! {
    device: stm32f429x,

    resources: {
        static BUFFER: Buffer<[u8; 15], Dma1Stream6> = Buffer::new([0; 15]);
    },

    tasks: {
        DMA1_STREAM6: {
            path: transfer_done,
            resources: [BUFFER, DMA1],
        },
    },
}

fn init(p: init::Peripherals, r: init::Resources) {
    let serial = Serial(p.USART2);

    serial.init(BAUD_RATE.invert(), Some(p.DMA1), p.GPIOA, p.RCC);
    r.BUFFER.borrow_mut().clone_from_slice(b"Hello, world!\r\n");

    serial.write_all(p.DMA1, r.BUFFER).unwrap();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn transfer_done(_t: &mut Threshold, r: DMA1_STREAM6::Resources) {
    r.BUFFER.release(r.DMA1).unwrap();

    rtfm::bkpt();
}

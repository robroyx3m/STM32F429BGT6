//! Sharing memory using a `Resource`
//!
//! This builds on top of the `concurrent` example. The `loopback` task now
//! additionally parses the received data as a command. Three commands are
//! available:
//!
//! - `reverse` - reverses the spin direction of the LED roulette
//! - `bounce` - puts the roulette in *bounce* mode where it reverses its spin
//!   direction every time it completes one turn.
//! - `continuous` - puts the roulette in *continuous* mode where it keeps
//!   spinning in the same direction.
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cast;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate heapless;
extern crate stm32_f429_bgt6;

use cast::{usize, u8};
use cortex_m::peripheral::SystClkSource;
use stm32_f429_bgt6::Serial;
use stm32_f429_bgt6::leds::LEDS;
use stm32_f429_bgt6::prelude::*;
use stm32_f429_bgt6::serial::Event;
use stm32_f429_bgt6::time::Hertz;
use heapless::Vec;
use rtfm::{app, Threshold};

use stm32_f429_bgt6::stm32f40x as stm32f429x; //VERY IMPORTANT! Always do this to clarify what the base device crate really is!

// CONFIGURATION
const BAUD_RATE: Hertz = Hertz(115_200);
const DIVISOR: u32 = 4;

// TASKS & RESOURCES
app! {
    device: stm32f429x,

    resources: {
        // 16 byte buffer
        static BUFFER: Vec<u8, [u8; 16]> = Vec::new();
        static SHARED: State = State::new();
        static STATE: u8 = 0;
    },

    tasks: {
        USART2: {
            path: receive,
            resources: [BUFFER, SHARED, USART2],
        },

        SYS_TICK: {
            path: roulette,
            resources: [SHARED, STATE],
        },
    }
}

// INITIALIZATION PHASE
fn init(p: init::Peripherals, _r: init::Resources) {
    f4::leds::init(&p.GPIOB, &p.RCC);

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
fn receive(_t: &mut Threshold, r: USART2::Resources) {
    let serial = Serial(&**r.USART2);

    let byte = serial.read().unwrap();

    serial.write(byte).unwrap();

    if byte == b'\r' {
        // end of command

        match &***r.BUFFER {
            b"bounce" => r.SHARED.mode = Mode::Bounce,
            b"continuous" => r.SHARED.mode = Mode::Continuous,
            b"reverse" => {
                r.SHARED.direction = r.SHARED.direction.reverse();
            }
            _ => {}
        }

        // clear the buffer to prepare for the next command
        r.BUFFER.clear();
    } else {
        // push the byte into the buffer

        if r.BUFFER.push(byte).is_err() {
            // error: buffer full
            // KISS: we just clear the buffer when it gets full
            r.BUFFER.clear();
        }
    }
}

fn roulette(_t: &mut Threshold, r: SYS_TICK::Resources) {
    let curr = **r.STATE;

    let mut direction = r.SHARED.direction;
    if curr == 0 && r.SHARED.mode == Mode::Bounce {
        direction = direction.reverse();
        r.SHARED.direction = direction;
    }

    let n = u8(LEDS.len()).unwrap();
    let next = match direction {
        Direction::Clockwise => (curr + 1) % n,
        Direction::Counterclockwise => curr.checked_sub(1).unwrap_or(n - 1),
    };

    LEDS[usize(curr)].off();
    LEDS[usize(next)].on();

    **r.STATE = next;
}

// SUPPORT CODE
pub struct State {
    direction: Direction,
    mode: Mode,
}

impl State {
    const fn new() -> Self {
        State {
            direction: Direction::Clockwise,
            mode: Mode::Continuous,
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Clockwise,
    Counterclockwise,
}

impl Direction {
    fn reverse(self) -> Self {
        match self {
            Direction::Clockwise => Direction::Counterclockwise,
            Direction::Counterclockwise => Direction::Clockwise,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Bounce,
    Continuous,
}

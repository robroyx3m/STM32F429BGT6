//! Eight LEDs connected to PORTB

use stm32f40x::{GPIOC, RCC};

/// All the LEDs
pub static LEDS: [Led; 2] = [
    Led { i: 11 },
    Led { i: 12 },
];

/// An LED
pub struct Led {
    i: u8,
}

impl Led {
    /// Turns off the LED
    pub fn off(&self) {
        // NOTE(safe) atomic write
        unsafe { (*GPIOC.get()).bsrr.write(|w| w.bits(1 << (self.i + 16))) }
    }

    /// Turns on the LED
    pub fn on(&self) {
        // NOTE(safe) atomic write
        unsafe { (*GPIOC.get()).bsrr.write(|w| w.bits(1 << self.i)) }
    }
}

/// Initializes all the user LEDs
pub fn init(gpioa: &GPIOC, rcc: &RCC) {
    // Power up peripherals
    rcc.ahb1enr.modify(|_, w| w.gpiocen().set_bit());

    // Configure pins 8-15 as outputs
    gpioa.moder.modify(|_, w| unsafe {
        w.moder11()
            .bits(1)
            .moder12()
            .bits(1)
    });
}

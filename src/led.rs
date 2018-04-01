//! User LED PA5

use stm32f40x::{GPIOC, RCC};

/// LED connected to pin PA5
pub const LED: PC12 = PC12;

/// Pin PA5. There's an LED connected to this pin
pub struct PC12;

/// Initializes the user LED
pub fn init(gpioa: &GPIOC, rcc: &RCC) {
    // power on GPIOA
    rcc.ahb1enr.modify(|_, w| w.gpiocen().set_bit());

    // configure PA5 as output
    gpioa.moder.modify(|_, w| unsafe{w.moder12().bits(1)});
}

impl PC12 {
    /// Turns the LED on
    pub fn on(&self) {
        unsafe {
            (*GPIOC.get()).odr.modify(|_, w| w.odr12().bit(true));
        }
    }

    /// Turns the LED off
    pub fn off(&self) {
        unsafe {
            (*GPIOC.get()).odr.modify(|_, w| w.odr12().bit(false));
        }
    }

    /// True if LED is ON, false otherwise.
    pub fn is_on(&self) -> bool {
        unsafe { (*GPIOC.get()).odr.read().odr12().bit_is_set() }
    }

    /// Toggles LED state.
    pub fn toggle(&self) {
        if LED.is_on() {
            LED.off();
        } else {
            LED.on();
        }
    }
}

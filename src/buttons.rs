//! User buttons PB13, PB14, PB15, PH14, PH15, PI0, PI1, PI2

use stm32f40x::{EXTI, GPIOB, GPIOD, GPIOH, GPIOI, RCC, SYSCFG};

/// Button connected to pins PB13, PB14, PB15, PH14, PH15, PI0, PI1, PI2

pub static BUTTONS: [Button; 8] = [
    Button { i: 13 },
    Button { i: 14 },
    Button { i: 15 },
    Button { i: 0 },
    Button { i: 1 },
    Button { i: 2 },
    Button { i: 8 },
    Button { i: 9 },
];

/// A button
pub struct Button {
    i: u8,
}

/// Initializes the user button with interrupt EXTI15_10, EXTI2, EXTI1, EXTI0
pub fn init(gpiob: &GPIOB, gpiod: &GPIOD, gpioh: &GPIOH, gpioi: &GPIOI, rcc: &RCC, syscfg: &SYSCFG, exti: &EXTI) {
    // Enable GPIOC
    rcc.ahb1enr.modify(|_, w| w.gpioben().set_bit());
    // Configure PC13 as input with pull-downs
    gpiob.moder.modify(|_, w| unsafe { w.moder13().bits(0) });
    gpiob.pupdr.modify(|_, w| unsafe { w.pupdr13().bits(0b10) });

    // Configure PD8 as input with pull-downs
    gpiod.moder.modify(|_, w| unsafe { w.moder8().bits(0) });
    gpiod.pupdr.modify(|_, w| unsafe { w.pupdr8().bits(0b10) });

    // Configure PD9 as input with pull-downs
    gpiod.moder.modify(|_, w| unsafe { w.moder9().bits(0) });
    gpiod.pupdr.modify(|_, w| unsafe { w.pupdr9().bits(0b10) });

    // Configure PH14 as input with pull-downs
    gpioh.moder.modify(|_, w| unsafe { w.moder14().bits(0) });
    gpioh.pupdr.modify(|_, w| unsafe { w.pupdr14().bits(0b10) });

    // Configure PH15 as input with pull-downs
    gpioh.moder.modify(|_, w| unsafe { w.moder15().bits(0) });
    gpioh.pupdr.modify(|_, w| unsafe { w.pupdr15().bits(0b10) });

    // Configure PI0 as input with pull-downs
    gpioi.moder.modify(|_, w| unsafe { w.moder0().bits(0) });
    gpioi.pupdr.modify(|_, w| unsafe { w.pupdr0().bits(0b10) });

    // Configure PI1 as input with pull-downs
    gpioi.moder.modify(|_, w| unsafe { w.moder1().bits(0) });
    gpioi.pupdr.modify(|_, w| unsafe { w.pupdr1().bits(0b10) });

    // Configure PI2 as input with pull-downs
    gpioi.moder.modify(|_, w| unsafe { w.moder2().bits(0) });
    gpioi.pupdr.modify(|_, w| unsafe { w.pupdr2().bits(0b10) });

    // System configuration controller clock enable
    rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());
    // Enable external interrupt RM0368 7.2.6
    syscfg
        .exticr4
        .modify(|_, w| unsafe { w.exti13().bits(0b0001) });
    syscfg
        .exticr3
        .modify(|_, w| unsafe { w.exti8().bits(0b0011) });
    syscfg
        .exticr3
        .modify(|_, w| unsafe { w.exti9().bits(0b0011) });
    syscfg
        .exticr4
        .modify(|_, w| unsafe { w.exti14().bits(0b0111) });
    syscfg
        .exticr4
        .modify(|_, w| unsafe { w.exti15().bits(0b0111) });
    syscfg
        .exticr1
        .modify(|_, w| unsafe { w.exti0().bits(0b1000) });
    syscfg
        .exticr1
        .modify(|_, w| unsafe { w.exti1().bits(0b1000) });
    syscfg
        .exticr1
        .modify(|_, w| unsafe { w.exti2().bits(0b1000) });
    // Interrupt request from line 13 is not masked
    exti.imr.modify(|_, w| w.mr13().set_bit());
    exti.imr.modify(|_, w| w.mr8().set_bit());
    exti.imr.modify(|_, w| w.mr9().set_bit());
    exti.imr.modify(|_, w| w.mr14().set_bit());
    exti.imr.modify(|_, w| w.mr15().set_bit());
    exti.imr.modify(|_, w| w.mr0().set_bit());
    exti.imr.modify(|_, w| w.mr1().set_bit());
    exti.imr.modify(|_, w| w.mr2().set_bit());

    // Falling edge trigger
    //rtsr if we want to activate when we press and not release the button
    exti.ftsr.modify(|_, w| w.tr13().set_bit());
    exti.ftsr.modify(|_, w| w.tr8().set_bit());
    exti.ftsr.modify(|_, w| w.tr9().set_bit());
    exti.ftsr.modify(|_, w| w.tr14().set_bit());
    exti.ftsr.modify(|_, w| w.tr15().set_bit());
    exti.ftsr.modify(|_, w| w.tr0().set_bit());
    exti.ftsr.modify(|_, w| w.tr1().set_bit());
    exti.ftsr.modify(|_, w| w.tr2().set_bit());
}

impl Button {
    /// True if button is pressed, false otherwise.

    /// Clear the pending external interrupt line used by the button, PR13
    pub fn clear_pending(&self, exti: &EXTI) {
        // RM0368 10.3.6 Pending register
        if self.i == 13 {
            exti.pr.modify(|_, w| w.pr13().set_bit());
        }
        else if self.i == 8 {
            exti.pr.modify(|_, w| w.pr8().set_bit());
        }
        else if self.i == 9 {
            exti.pr.modify(|_, w| w.pr9().set_bit());
        }
        else if self.i == 14 {
            exti.pr.modify(|_, w| w.pr14().set_bit());
        }
        else if self.i == 15 {
            exti.pr.modify(|_, w| w.pr15().set_bit());
        }
        else if self.i == 0 {
            exti.pr.modify(|_, w| w.pr0().set_bit());
        }
        else if self.i == 1 {
            exti.pr.modify(|_, w| w.pr1().set_bit());
        }
        else if self.i == 2 {
            exti.pr.modify(|_, w| w.pr2().set_bit());
        }
    }
}

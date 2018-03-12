//! Serial interface
//!
//! You can use the `Serial` interface with these USART instances
//!
//! # USART2
//!
//! - TX = PA2
//! - RX = PA3
//! - Interrupt = USART2

use core::any::{Any, TypeId};
use core::marker::Unsize;
use core::ops::Deref;
use core::ptr;

use cast::u16;
use hal;
use nb;
use static_ref::Static;
use stm32f40x::{gpioa, DMA1, USART2, usart6, GPIOA, RCC};

use dma::{self, Buffer, Dma1Stream5, Dma1Stream6};

use core::fmt;

/// Specialized `Result` type
pub type Result<T> = ::core::result::Result<T, nb::Error<Error>>;

/// IMPLEMENTATION DETAIL
pub unsafe trait Usart: Deref<Target = usart6::RegisterBlock> {
    /// IMPLEMENTATION DETAIL
    type GPIO: Deref<Target = gpioa::RegisterBlock>;
    /// IMPLEMENTATION DETAIL
    type Ticks: Into<u32>;
}

unsafe impl Usart for USART2 {
    type GPIO = GPIOA;
    type Ticks = ::apb2::Ticks;
}

/// An error
#[derive(Debug)]
pub enum Error {
    /// De-synchronization, excessive noise or a break character detected
    Framing,
    /// Noise detected in the received frame
    Noise,
    /// RX buffer overrun
    Overrun,
    #[doc(hidden)]
    _Extensible,
}

/// Interrupt event
pub enum Event {
    /// RX buffer Not Empty (new data available)
    Rxne,
    /// Transmission Complete
    Tc,
    /// TX buffer Empty (more data can be send)
    Txe,
}

/// Serial interface
///
/// # Interrupts
///
/// - RXNE
pub struct Serial<'a, U>(pub &'a U)
where
    U: Any + Usart;

impl<'a, U> Clone for Serial<'a, U>
where
    U: Any + Usart,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, U> Copy for Serial<'a, U>
where
    U: Any + Usart,
{
}

impl<'a, U> Serial<'a, U>
where
    U: Any + Usart,
{
    /// Initializes the serial interface with a baud rate of `baut_rate` bits
    /// per second
    ///
    /// The serial interface will be configured to use 8 bits of data, 1 stop
    /// bit, no hardware control and to omit parity checking
    pub fn init<B>(&self, baud_rate: B, dma1: Option<&DMA1>, gpio: &U::GPIO, rcc: &RCC)
    where
        B: Into<U::Ticks>,
    {
        self._init(baud_rate.into(), dma1, gpio, rcc)
    }

    fn _init(&self, baud_rate: U::Ticks, dma1: Option<&DMA1>, gpio: &U::GPIO, rcc: &RCC) {
        let usart = self.0;

        // power up peripherals
        if dma1.is_some() {
            rcc.ahb1enr.modify(|_, w| w.dma1en().set_bit());
        }
        // RM0368 6.3.9
        // enable clock to GPIOA, USART2
        if usart.get_type_id() == TypeId::of::<USART2>() {
            rcc.apb1enr.modify(|_, w| w.usart2en().set_bit());
            rcc.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        }
        // PA2. = TX, PA3 = RX

        // RM0368 8.4.1
        // set output mode for GPIOA
        // PA2 = TX (output mode), PA3 = RX (input mode)
        if usart.get_type_id() == TypeId::of::<USART2>() {
            // we don't care about the speed register atm
            // DM00102166
            // PA2 and PA3 is connected to USART2 TX and RX respectively
            // Alternate function AF7, Table 9
            gpio.afrl
                .modify(|_, w| unsafe { w.afrl2().bits(7).afrl3().bits(7) });
            // Highest output speed
            gpio.ospeedr
                .modify(|_, w| unsafe { w.ospeedr2().bits(0b11).ospeedr3().bits(0b11) });
            // RM0368 8.3 Table 23
            gpio.moder
                .modify(|_, w| unsafe { w.moder2().bits(2).moder3().bits(2) });
        }

        if let Some(dma1) = dma1 {
            if usart.get_type_id() == TypeId::of::<USART2>() {
                // TX DMA transfer
                // chsel: Channel 4 (RM0368 9.3.3 Table 27)
                // pl: Medium priority
                // msize: Memory size = 8 bits
                // psize: Peripheral size = 8 bits
                // minc: Memory increment mode enabled
                // pinc: Peripheral increment mode disabled
                // circ: Circular mode disabled
                // dir: Transfer from memory to peripheral
                // tcie: Transfer complete interrupt enabled
                // en: Disabled
                dma1.s6cr.write(|w| unsafe {
                    w.chsel()
                        .bits(4)
                        .pl()
                        .bits(0b01)
                        .msize()
                        .bits(0b00)
                        .psize()
                        .bits(0b00)
                        .minc()
                        .set_bit()
                        .circ()
                        .clear_bit()
                        .pinc()
                        .clear_bit()
                        .dir()
                        .bits(1)
                        .tcie()
                        .set_bit()
                        .en()
                        .clear_bit()
                });

                // RX DMA transfer
                // chsel: Channel 4 (RM0368 9.3.3 Table 27)
                // pl: Medium priority
                // msize: Memory size = 8 bits
                // psize: Peripheral size = 8 bits
                // minc: Memory increment mode enabled
                // pinc: Peripheral increment mode disabled
                // circ: Circular mode disabled
                // dir: Transfer from peripheral to memory
                // tcie: Transfer complete interrupt enabled
                // en: Disabled
                dma1.s5cr.write(|w| unsafe {
                    w.chsel()
                        .bits(4)
                        .pl()
                        .bits(0b01)
                        .msize()
                        .bits(0b00)
                        .psize()
                        .bits(0b00)
                        .minc()
                        .set_bit()
                        .circ()
                        .clear_bit()
                        .pinc()
                        .clear_bit()
                        .dir()
                        .bits(0)
                        .tcie()
                        .set_bit()
                        .en()
                        .clear_bit()
                });
            }
        }

        // 8N1, stop bit
        usart.cr2.write(|w| unsafe { w.stop().bits(0b00) });

        // Baud rate
        // Check if peripheral does not use default clock
        let ahb1_clk: u32 = ::ahb1::Ticks::from(::time::Seconds(1)).into();
        let apb1_clk: u32 = ::apb1::Ticks::from(::time::Seconds(1)).into();
        let apb1psc: u32 = ahb1_clk / apb1_clk;
        let brr = baud_rate.into() / apb1psc;
        assert!(brr >= 16, "impossible baud rate");
        usart.brr.write(|w| unsafe { w.bits(brr) });

        // disable hardware flow control
        // enable DMA TX and RX transfers
        usart.cr3.write(|w| {
            w.rtse()
                .clear_bit()
                .ctse()
                .clear_bit()
                .dmat()
                .set_bit()
                .dmar()
                .set_bit()
        });

        // enable TX, RX; disable parity checking
        usart.cr1.write(|w| {
            w.ue()
                .set_bit()
                .re()
                .set_bit()
                .te()
                .set_bit()
                .m()
                .clear_bit()
                .over8()
                .clear_bit()
                .pce()
                .clear_bit()
                .rxneie()
                .clear_bit()
        });
    }

    /// Starts listening for an interrupt `event`
    pub fn listen(&self, event: Event) {
        let usart = self.0;

        match event {
            Event::Rxne => usart.cr1.modify(|_, w| w.rxneie().set_bit()),
            Event::Tc => usart.cr1.modify(|_, w| w.tcie().set_bit()),
            Event::Txe => usart.cr1.modify(|_, w| w.txeie().set_bit()),
        }
    }

    /// Stops listening for an interrupt `event`
    pub fn unlisten(&self, event: Event) {
        let usart = self.0;

        match event {
            Event::Rxne => usart.cr1.modify(|_, w| w.rxneie().clear_bit()),
            Event::Tc => usart.cr1.modify(|_, w| w.tcie().clear_bit()),
            Event::Txe => usart.cr1.modify(|_, w| w.txeie().clear_bit()),
        }
    }
}

impl<'a, U> hal::serial::Read<u8> for Serial<'a, U>
where
    U: Any + Usart,
{
    type Error = Error;

    fn read(&self) -> Result<u8> {
        let usart2 = self.0;
        let sr = usart2.sr.read();

        if sr.ore().bit_is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.nf().bit_is_set() {
            Err(nb::Error::Other(Error::Noise))
        } else if sr.fe().bit_is_set() {
            Err(nb::Error::Other(Error::Framing))
        } else if sr.rxne().bit_is_set() {
            // NOTE(read_volatile) the register is 9 bits big but we'll only
            // work with the first 8 bits
            Ok(unsafe { ptr::read_volatile(&usart2.dr as *const _ as *const u8) })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<'a, U> hal::serial::Write<u8> for Serial<'a, U>
where
    U: Any + Usart,
{
    type Error = Error;

    fn write(&self, byte: u8) -> Result<()> {
        let usart2 = self.0;
        let sr = usart2.sr.read();

        if sr.ore().bit_is_set() {
            Err(nb::Error::Other(Error::Overrun))
        } else if sr.nf().bit_is_set() {
            Err(nb::Error::Other(Error::Noise))
        } else if sr.fe().bit_is_set() {
            Err(nb::Error::Other(Error::Framing))
        } else if sr.txe().bit_is_set() {
            // NOTE(write_volatile) see NOTE in the `read` method
            unsafe { ptr::write_volatile(&usart2.dr as *const _ as *mut u8, byte) }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<'a> Serial<'a, USART2> {
    /// Starts a DMA transfer to receive serial data into a `buffer`
    ///
    /// This will mutably lock the `buffer` preventing borrowing its contents
    /// The `buffer` can be `release`d after the DMA transfer finishes
    // TODO support circular mode + half transfer interrupt as a double
    // buffering mode
    pub fn read_exact<B>(
        &self,
        dma1: &DMA1,
        buffer: &Static<Buffer<B, Dma1Stream5>>,
    ) -> ::core::result::Result<(), dma::Error>
    where
        B: Unsize<[u8]>,
    {
        let usart2 = self.0;

        if dma1.s5cr.read().en().bit_is_set() {
            return Err(dma::Error::InUse);
        }

        let buffer: &mut [u8] = buffer.lock_mut();

        dma1.s5ndtr
            .write(|w| unsafe { w.ndt().bits(u16(buffer.len()).unwrap()) });
        dma1.s5par
            .write(|w| unsafe { w.bits(&usart2.dr as *const _ as u32) });
        dma1.s5m0ar
            .write(|w| unsafe { w.bits(buffer.as_ptr() as u32) });
        dma1.s5cr.modify(|_, w| w.en().set_bit());

        Ok(())
    }

    /// Starts a DMA transfer to send `buffer` through this serial port
    ///
    /// This will immutably lock the `buffer` preventing mutably borrowing its
    /// contents. The `buffer` can be `release`d after the DMA transfer finishes
    pub fn write_all<B>(
        &self,
        dma1: &DMA1,
        buffer: &Static<Buffer<B, Dma1Stream6>>,
    ) -> ::core::result::Result<(), dma::Error>
    where
        B: Unsize<[u8]>,
    {
        // write!(dma1, "hi {}", 1);
        let usart2 = self.0;

        if dma1.s6cr.read().en().bit_is_set() {
            return Err(dma::Error::InUse);
        }

        let buffer: &[u8] = buffer.lock();

        dma1.s6ndtr
            .write(|w| unsafe { w.ndt().bits(u16(buffer.len()).unwrap()) });
        dma1.s6par
            .write(|w| unsafe { w.bits(&usart2.dr as *const _ as u32) });
        dma1.s6m0ar
            .write(|w| unsafe { w.bits(buffer.as_ptr() as u32) });
        dma1.s6cr.modify(|_, w| w.en().set_bit());

        Ok(())
    }
}
///
pub struct U8Writer<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> U8Writer<'a> {
    ///
    pub fn new(buf: &'a mut [u8]) -> Self {
        U8Writer {
            buf: buf,
            offset: 0,
        }
    }
}

impl<'a> fmt::Write for U8Writer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        // Skip over already-copied data
        let remainder = &mut self.buf[self.offset..];
        // Make the two slices the same length
        let remainder = &mut remainder[..bytes.len()];
        // Copy
        remainder.copy_from_slice(bytes);
        // Increment offset by number of copied bytes
        self.offset += bytes.len();
        Ok(())
    }
}

/// Macro for printing formatted strings over serial through DMA.
/// Uses the cortex-m-rtfm resource model and can thus not be used
/// outside rtfm tasks.
#[macro_export]
macro_rules! uprint {
    ($T:ident, $USART:expr, $DMA:expr, $TX_BUFFER:expr) => {
        // Transmit the contents of the buffer using DMA
        $TX_BUFFER.claim($T, |tx, t| {
            $DMA.claim(t, |dma, t| {
                $USART.claim(t, |usart, _| {
                    let serial = Serial(&**usart);
                    serial.write_all(dma, tx).unwrap();
                });
            });
        });
    };
    ($T:ident, $USART:expr, $DMA:expr, $TX_BUFFER:expr, $s:expr) => {
        use rtfm::{Resource};
        use core::fmt::Write;
        use f4::U8Writer;
        // Claim the transmit buffer and write a string literal into it
        $TX_BUFFER.claim_mut($T, |tx, _| {
            let len = (*tx).deref().borrow().len();
            let buf = &mut (*tx).deref().borrow_mut();
            write!(U8Writer::new(&mut buf[..len]), $s).unwrap();
        });
        uprint!($T, $USART, $DMA, $TX_BUFFER);
    };
    ($T:ident, $USART:expr, $DMA:expr, $TX_BUFFER:expr, $($arg:tt)* ) => {
        use rtfm::{Resource};
        use core::fmt::Write;
        use f4::U8Writer;
        // Claim the transmit buffer and write a formatted string into it
        $TX_BUFFER.claim_mut($T, |tx, _| {
            let len = (*tx).deref().borrow().len();
            let buf = &mut (*tx).deref().borrow_mut();
            write!(U8Writer::new(&mut buf[..len]), $($arg)*).unwrap();
        });
        uprint!($T, $USART, $DMA, $TX_BUFFER);
    };
}

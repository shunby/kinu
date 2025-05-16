use core::fmt::{self, Write};

use alloc::vec::Vec;
use reg::{Reg, Registers};

use crate::mutex::Mutex;

mod reg;

const COM1: u16 = 0x3f8;
static UART_REGS: Mutex<Registers> = Mutex::new(Registers::new(COM1));

pub fn init() {
    let regs = UART_REGS.lock();
    regs.write(Reg::LineCtrl, 0);
    regs.write(Reg::InterruptEnable, 0);
    regs.write(Reg::FifoCtrl, 0);

    // 115200 baud
    regs.write(Reg::DivisorLow, 1);
    regs.write(Reg::DivisorHigh, 0);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::uart::print(core::format_args!($($arg)*));
    }};
}

pub fn print(args: fmt::Arguments<'_>) {
    let mut serial = Serial {};
    serial.write_fmt(args).unwrap();
}

pub fn read() -> Vec<u8> {
    Serial::read()
}

struct Serial {}

impl Serial {
    fn is_tx_busy() -> bool {
        let regs = UART_REGS.lock();
        (regs.read(Reg::LineStatus) & 0b0010_0000) == 0
    }

    fn is_rx_empty() -> bool {
        let regs = UART_REGS.lock();
        (regs.read(Reg::LineStatus) & 1) == 0
    }

    fn write(val: u8) {
        while Self::is_tx_busy() {}
        let regs = UART_REGS.lock();
        regs.write(Reg::Tx, val);
    }

    fn read_one_byte() -> u8 {
        while Self::is_rx_empty() {}
        let byte = UART_REGS.lock().read(Reg::Rx);
        // echo back
        Self::write(byte);
        if byte == b'\r' {
            Self::write(b'\n');
        }
        byte
    }

    fn read() -> Vec<u8> {
        let mut v = Vec::new();
        while !v.last().is_some_and(|&c|c==b'\r') {
            v.push(Self::read_one_byte());
        }
        v
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            Self::write(c);
        }
        Ok(())
    }
}

use core::{cell::Cell, marker::PhantomData};

use x86_64::structures::port::{PortRead, PortWrite};

use crate::mutex::Mutex;

enum AccessType {
    Read,
    Write,
    ReadWrite,
}

pub enum Reg {
    Rx,
    Tx,
    InterruptEnable,
    DivisorLow,
    DivisorHigh,
    InterruptId,
    FifoCtrl,
    LineCtrl,
    ModemCtrl,
    LineStatus,
    ModemStatus,
    Scratch,
}

struct RegSpec {
    offset: u16,
    dlab: bool,
    ty: AccessType,
}

impl RegSpec {
    fn new(offset: u16, dlab: bool, ty: AccessType) -> Self {
        Self { offset, dlab, ty }
    }
}

impl Reg {
    pub fn spec(self) -> RegSpec {
        match self {
            Reg::Rx => RegSpec::new(0, false, AccessType::Read),
            Reg::Tx => RegSpec::new(0, false, AccessType::Write),
            Reg::InterruptEnable => RegSpec::new(1, false, AccessType::ReadWrite),
            Reg::DivisorLow => RegSpec::new(0, true, AccessType::ReadWrite),
            Reg::DivisorHigh => RegSpec::new(1, true, AccessType::ReadWrite),
            Reg::InterruptId => todo!(),
            Reg::FifoCtrl => RegSpec::new(2, false, AccessType::Write),
            Reg::LineCtrl => RegSpec::new(3, false, AccessType::ReadWrite),
            Reg::ModemCtrl => todo!(),
            Reg::LineStatus => RegSpec::new(5, false, AccessType::Read),
            Reg::ModemStatus => todo!(),
            Reg::Scratch => todo!(),
        }
    }
}

pub struct Registers {
    port: u16,
    _marker_unsync: PhantomData<Cell<()>>,
}

impl Registers {
    pub(crate) const fn new(port: u16) -> Self {
        Self {
            port,
            _marker_unsync: PhantomData,
        }
    }

    pub fn read(&self, reg: Reg) -> u8 {
        let spec = reg.spec();
        if spec.dlab {
            self.enable_dlab();
        }
        let v = match spec.ty {
            AccessType::Read | AccessType::ReadWrite => unsafe {
                u8::read_from_port(self.port + spec.offset)
            },
            AccessType::Write => panic!("uart: tried to read from a write-only register"),
        };
        if spec.dlab {
            self.disable_dlab();
        }
        v
    }

    pub fn write(&self, reg: Reg, val: u8) {
        let spec = reg.spec();
        if spec.dlab {
            self.enable_dlab();
        }
        match spec.ty {
            AccessType::Write | AccessType::ReadWrite => unsafe {
                u8::write_to_port(self.port + spec.offset, val);
            },
            AccessType::Read => panic!("uart: tried to write to a read-only register"),
        }
        if spec.dlab {
            self.disable_dlab();
        }
    }

    fn enable_dlab(&self) {
        let port = self.port + Reg::LineCtrl.spec().offset;
        unsafe {
            let lcr = u8::read_from_port(port);
            u8::write_to_port(port, lcr | 0b1000_0000);
        }
    }

    fn disable_dlab(&self) {
        let port = self.port + Reg::LineCtrl.spec().offset;
        unsafe {
            let lcr = u8::read_from_port(port);
            u8::write_to_port(port, lcr & 0b0111_1111);
        }
    }
}

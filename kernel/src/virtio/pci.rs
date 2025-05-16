use virtio_drivers::transport::pci::bus::{ConfigurationAccess, DeviceFunction};
// use bitfield::bitfield;
use x86_64::structures::port::{PortRead, PortWrite};

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    let (bus, device, function, reg_addr) =
        (bus as u32, device as u32, function as u32, reg_addr as u32);
    1 << 31 | bus << 16 | device << 11 | function << 8 | (reg_addr & 0xfc)
}

fn read_confreg(address: u32) -> u32 {
    unsafe {
        u32::write_to_port(CONFIG_ADDRESS, address);
        u32::read_from_port(CONFIG_DATA)
    }
}

fn write_confreg(address: u32, value: u32) {
    unsafe {
        u32::write_to_port(CONFIG_ADDRESS, address);
        u32::write_to_port(CONFIG_DATA, value);
    }
}

pub struct IoCam();

impl IoCam {
    /// Safety: do not instantiate twice
    pub unsafe fn new() -> Self {
        IoCam()
    }
}

impl ConfigurationAccess for IoCam {
    fn read_word(&self, device_function: DeviceFunction, register_offset: u8) -> u32 {
        let addr = make_address(
            device_function.bus,
            device_function.device,
            device_function.function,
            register_offset,
        );
        read_confreg(addr)
    }

    fn write_word(&mut self, device_function: DeviceFunction, register_offset: u8, data: u32) {
        let addr = make_address(
            device_function.bus,
            device_function.device,
            device_function.function,
            register_offset,
        );
        write_confreg(addr, data);
    }

    unsafe fn unsafe_clone(&self) -> Self {
        Self()
    }
}

#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

extern crate alloc;
use alloc::{sync::Arc, vec::Vec};
use core::{arch::asm, panic::PanicInfo};
use lib::mmap::CMemoryMap;
use uefi::boot::MemoryDescriptor;
use wasmtime::{Config, CustomCodeMemory, Engine, Instance, Module, Store};

mod memory;
mod mutex;
mod uart;

struct Stack([u8; 1024 * 1024]);

#[unsafe(no_mangle)]
static mut KMAIN_STACK: Stack = Stack([0u8; 1024 * 1024]);

#[unsafe(no_mangle)]
#[allow(unreachable_code)]
pub unsafe extern "sysv64" fn kmain(mm: CMemoryMap) -> ! {
    unsafe {
        asm!("lea rsp, [KMAIN_STACK + 1024 * 1024]");
        kmain2(mm);
        loop {}
    }
}

const DATA: &[u8] = include_bytes!("fib.cwasm");

struct CCM {}

impl CustomCodeMemory for CCM {
    fn required_alignment(&self) -> usize {
        1
    }
    fn publish_executable(&self, ptr: *const u8, len: usize) -> anyhow::Result<()> {
        Ok(())
    }
    fn unpublish_executable(&self, ptr: *const u8, len: usize) -> anyhow::Result<()> {
        Ok(())
    }
}
#[unsafe(no_mangle)]
pub fn kmain2(mmap: CMemoryMap) {
    uart::init();
    memory::init(&mmap);
    print!("char\n");

    let mut config = Config::default();
    config.target("x86_64-unknown-none").unwrap();
    config.memory_init_cow(false);
    config.memory_reservation(0);
    config.memory_reservation_for_growth(0);
    config.memory_guard_size(0);
    config.signals_based_traps(false);
    config.debug_info(false);
    config.memory_may_move(false);
    config.guard_before_linear_memory(false);
    config.table_lazy_init(false);
    config.wasm_backtrace(false);
    config.wasm_bulk_memory(false);
    config.with_custom_code_memory(Some(Arc::new(CCM {})));

    let engine = Engine::new(&config).unwrap();
    unsafe {
        let module = Module::deserialize(&engine, DATA).unwrap();
        let mut store = Store::new(&engine, ());

        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        print!("{:?}\r\n", instance);
        print!("{:?}\r\n", module);
        let fnc = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "gcd")
            .unwrap();
        print!("func: {:?}\r\n", fnc.func());
        let ret = fnc.call(&mut store, (5, 20)).unwrap();
        print!("{ret}\r\n");
    }

    print!("end\r\n");

    loop {
        unsafe { asm!("hlt") }
    }
}

#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    print!("{}", panic.message());
    loop {}
}

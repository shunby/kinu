#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

extern crate alloc;
use alloc::{string::String, sync::Arc};
use core::{arch::asm, panic::PanicInfo};
use lib::mmap::CMemoryMap;
use virtio::VirtIO;
use virtio_drivers::transport::pci::PciTransport;
use wasi_random::{HostRandom, RandomImpl, wasi::random::random::add_to_linker};
use wasmtime::{
    Config, CustomCodeMemory, Engine, Store,
    component::{Component, Linker},
};

mod memory;
mod mutex;
mod uart;
mod virtio;

#[allow(dead_code)]
#[repr(align(1024))]
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

const DATA: &[u8] = include_bytes!("sample_app.cwasm");

struct CCM {}

impl CustomCodeMemory for CCM {
    fn required_alignment(&self) -> usize {
        1
    }
    fn publish_executable(&self, _ptr: *const u8, _len: usize) -> anyhow::Result<()> {
        Ok(())
    }
    fn unpublish_executable(&self, _ptr: *const u8, _len: usize) -> anyhow::Result<()> {
        Ok(())
    }
}
struct HostState<T: HostRandom> {
    rand: RandomImpl<T>,
}
#[unsafe(no_mangle)]
pub fn kmain2(mmap: CMemoryMap) {
    uart::init();
    memory::init(&mmap);
    let virtio = virtio::init();
    let rand = RandomImpl::new(virtio.clone());
    print!("Hello from kernel\r\n");

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
    config.wasm_bulk_memory(true);
    config.with_custom_code_memory(Some(Arc::new(CCM {})));

    let engine = Engine::new(&config).unwrap();
    let mut linker: Linker<HostState<VirtIO<PciTransport>>> = Linker::new(&engine);
    add_to_linker(&mut linker, move |s: &mut HostState<_>| &mut s.rand).unwrap();
    linker
        .root()
        .func_wrap("print", |_, s: (String,)| {
            print!("{}", s.0);
            Ok(())
        })
        .unwrap();
    linker
        .root()
        .func_wrap("input", |_, ()| {
            let bytes = uart::read();
            let res: u64 = String::from_utf8_lossy(&bytes)
                .trim_ascii()
                .parse()
                .unwrap_or(0);
            Ok((res,))
        })
        .unwrap();

    // instance.func_wrap(name, func)

    unsafe {
        let state = HostState { rand };
        let component = Component::deserialize(&engine, DATA).unwrap();
        let mut store = Store::new(&engine, state);

        let instance = linker.instantiate(&mut store, &component).unwrap();
        let fnc = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .unwrap();
        if let Err(e) = fnc.call(&mut store, ()) {
            print!("{e}");
        }
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

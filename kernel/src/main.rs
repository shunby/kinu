#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

extern crate alloc;
use core::{alloc::GlobalAlloc, arch::asm, cell::SyncUnsafeCell, panic::PanicInfo, ptr::{null_mut, NonNull}};
use alloc::vec::Vec;
use lib::mmap::CMemoryMap;
use mutex::Mutex;
use uefi::boot::{MemoryDescriptor, MemoryType};

// mod paging;
mod mutex;
mod memory;

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

pub fn kmain2(mm: CMemoryMap) {
    let mmap: &[MemoryDescriptor] = unsafe {
        mm.entries()
    };
    memory::init(mmap);
    let mut v = Vec::new();
    v.push(0);
    v.push(1);
    v.push(2);

    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

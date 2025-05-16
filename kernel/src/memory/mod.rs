use core::cell::SyncUnsafeCell;

use bitmap::BitMapMemoryManager;
use lib::mmap::CMemoryMap;
use slob::SlobAlloc;

mod bitmap;
mod slob;

const MM_MAX_PHYS_ADDR: usize = 0x1_0000_0000;
const MM_BYTES_PER_PAGE: usize = 0x1000;

static PAGE_ALLOCATOR: SyncUnsafeCell<BitMapMemoryManager> =
    SyncUnsafeCell::new(BitMapMemoryManager::new_null());

#[global_allocator]
static GLOBAL_ALLOCATOR: SlobAlloc = SlobAlloc::new();

pub fn init(mmap: &CMemoryMap) {
    unsafe {
        (*PAGE_ALLOCATOR.get()).init(mmap);
    }
}

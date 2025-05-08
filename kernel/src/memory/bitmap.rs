use core::ptr::null_mut;
use uefi::boot::{MemoryDescriptor, MemoryType};

use super::{MM_BYTES_PER_PAGE, MM_MAX_PHYS_ADDR};

pub(super) struct BitMapMemoryManager {
    bits: [u8; MM_MAX_PHYS_ADDR / MM_BYTES_PER_PAGE / 8]
}

impl BitMapMemoryManager {
    pub const fn new_null() -> Self {
        Self {
            bits: [0u8; MM_MAX_PHYS_ADDR / MM_BYTES_PER_PAGE / 8]
        }
    }

    pub fn init(&mut self, mmap: &[MemoryDescriptor]) {
        for desc in mmap {
            if matches!(desc.ty, MemoryType::CONVENTIONAL | MemoryType::BOOT_SERVICES_CODE) {
                let start_page = desc.phys_start / 0x1000;
                let end_page = start_page + desc.page_count;
                

                for p in start_page..end_page {
                    let line = p / 8;
                    let off  = p % 8;
                    self.bits[line as usize] |= 1<<off;
                }
            }
        }
    }

    pub fn alloc(&mut self, pages: usize) -> *mut u8 {
        let mut cnt = 0;
        for i in 0..(MM_MAX_PHYS_ADDR / MM_BYTES_PER_PAGE / 8) {
            let line = self.bits[i];
            for j in 0..8 {
                if (line >> j) & 1 == 1 {
                    cnt += 1;
                    if cnt == pages {
                        self.bits[i] ^= 1 << j;
                        let page_idx = i * 8 + j - (cnt - 1);
                        return (page_idx << 12) as usize as _;
                    }
                } else {
                    cnt = 0;
                }
            }
        }
        null_mut()
    }

    pub fn dealloc(&mut self, ptr: *mut u8, pages: usize) {
        let start_page = ptr as usize >> 12;
        let end_page = start_page + pages;
        for page in start_page..end_page {
            let line = page / 8;
            let off = page % 8;
            self.bits[line] |= 1<<off;
        }
    }
}

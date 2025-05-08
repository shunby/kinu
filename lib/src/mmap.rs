use uefi::{boot::MemoryDescriptor, mem::memory_map::{MemoryMap, MemoryMapOwned}};

#[repr(C)]
pub struct CMemoryMap {
    buf: *const MemoryDescriptor,
    map_size: usize,
    desc_size: usize,
    len: usize
}

impl CMemoryMap {
    pub fn new (value: &MemoryMapOwned) -> Self {
        Self {
            buf: value.buffer().as_ptr() as *const MemoryDescriptor,
            map_size: value.meta().map_size,
            desc_size: value.meta().desc_size,
            len: value.len()
        }
    }

    pub unsafe fn entries(self) -> &'static [MemoryDescriptor] {
        let descs = unsafe {
            core::slice::from_raw_parts(self.buf, self.len)
        };
        descs
    }
}

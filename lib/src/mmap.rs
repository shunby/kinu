use uefi::{boot::MemoryDescriptor, mem::memory_map::{MemoryMap, MemoryMapOwned}};

#[repr(C)]
pub struct CMemoryMap {
    buf: *const u8,
    map_size: usize,
    desc_size: usize,
    len: usize
}

impl CMemoryMap {
    pub fn new (value: &MemoryMapOwned) -> Self {
        Self {
            buf: value.buffer().as_ptr(),
            map_size: value.meta().map_size,
            desc_size: value.meta().desc_size,
            len: value.len()
        }
    }
    
    pub fn iter<'a>(&'a self) -> MMapIter<'a> {
        MMapIter { mmap: self, index: 0 }
    }
}


pub struct MMapIter<'a> {
    mmap: &'a CMemoryMap,
    index: usize
}

impl<'a> Iterator for MMapIter<'a> {
    type Item = &'a MemoryDescriptor;
    fn next(&mut self) -> Option<Self::Item> {
        if self.mmap.len <= self.index {
            None
        } else {
            unsafe {
                let p = self.mmap.buf.add(self.mmap.desc_size * self.index) as *const MemoryDescriptor;
                self.index += 1;
                Some(&*p)
            }
        }
    }
}

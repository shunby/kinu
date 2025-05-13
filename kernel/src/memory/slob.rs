use core::{
    alloc::GlobalAlloc,
    ptr::{NonNull, null_mut},
};

use crate::{mutex::Mutex, print, uart::print};

use super::{BitMapMemoryManager, MM_BYTES_PER_PAGE, PAGE_ALLOCATOR};

#[derive(Clone, Copy, Default)]
struct FreeList {
    next: Option<NonNull<FreeList>>,
}

fn alloc_freelist(mm: &mut BitMapMemoryManager, size: usize) -> Option<NonNull<FreeList>> {
    debug_assert!(MM_BYTES_PER_PAGE % size == 0);
    let page = NonNull::new(mm.alloc(1).unwrap().as_ptr())?;
    let blocks = MM_BYTES_PER_PAGE / size;
    let mut head = None;

    for i in (0..blocks).rev() {
        unsafe {
            let mut p: NonNull<FreeList> = page.add(i * size).cast();
            p.as_mut().next = head;
            head = Some(p);
        }
    }
    head
}

pub(super) struct SlobAlloc {
    heads: [Mutex<FreeList>; 13],
}

impl SlobAlloc {
    pub const fn new() -> Self {
        Self {
            heads: [const { Mutex::new(FreeList { next: None }) }; 13],
        }
    }
}

unsafe impl Sync for SlobAlloc {}

unsafe impl GlobalAlloc for SlobAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.align() > MM_BYTES_PER_PAGE {
            panic!("alloc: too large alignment value");
        }

        print!("alloc: {:x}\r\n", layout.size());

        if layout.size() >= MM_BYTES_PER_PAGE {
            let pages = (layout.size() + MM_BYTES_PER_PAGE - 1) / MM_BYTES_PER_PAGE;
            unsafe {
                return (*PAGE_ALLOCATOR.get())
                    .alloc(pages)
                    .expect("memory allocation failed")
                    .as_ptr();
            }
        }

        // layout size < MM_BYTES_PER_PAGE
        let alloc_size = usize::max(layout.size(), layout.align()).max(16);
        let idx = core::mem::size_of::<usize>() * 8 - (alloc_size - 1).leading_zeros() as usize;
        let mut head = self.heads[idx].lock();

        if head.next.is_none() {
            unsafe {
                head.next = alloc_freelist(&mut *PAGE_ALLOCATOR.get(), 1 << idx);
            }
        }

        unsafe {
            let mut next_ptr = head.next.unwrap();
            let nnext = next_ptr.as_mut().next;
            head.next = nnext;
            next_ptr.as_ptr() as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        print!("dealloc: {:x}\r\n", layout.size());
        if layout.align() > MM_BYTES_PER_PAGE {
            panic!("alloc: too large alignment value");
        }

        if layout.size() >= MM_BYTES_PER_PAGE {
            let pages = (layout.size() + MM_BYTES_PER_PAGE - 1) / MM_BYTES_PER_PAGE;
            unsafe {
                (*PAGE_ALLOCATOR.get()).dealloc(ptr, pages);
            }
            return;
        }

        // layout size < MM_BYTES_PER_PAGE
        let alloc_size = usize::max(layout.size(), layout.align()).max(16);
        let idx = core::mem::size_of::<usize>() * 8 - (alloc_size - 1).leading_zeros() as usize;
        let mut head = self.heads[idx].lock();

        unsafe {
            let ptr = ptr as *mut FreeList;
            (*ptr).next = head.next;
            head.next = Some(NonNull::new(ptr).unwrap());
        }
    }
}

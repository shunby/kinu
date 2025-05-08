use core::{alloc::GlobalAlloc, ptr::NonNull};

use crate::mutex::Mutex;

use super::{BitMapMemoryManager, MM_BYTES_PER_PAGE, PAGE_ALLOCATOR};


#[derive(Clone, Copy, Default)]
struct FreeList {
    next: Option<NonNull<FreeList>>
}

fn alloc_freelist(mm: &mut BitMapMemoryManager, size: usize) -> Option<NonNull<FreeList>> {
    let page = NonNull::new(mm.alloc(1) as *mut FreeList)?;
    let blocks = MM_BYTES_PER_PAGE / size;
    let mut head = None;

    for i in (0..blocks).rev() {
        unsafe {
            let mut p = page.add(i);
            p.as_mut().next = head;
            head = Some(p);
        }
    }
    head
}

pub(super) struct SlobAlloc {
    heads: [Mutex<FreeList>; 13]
}

impl SlobAlloc {
    pub const fn new() -> Self {
        Self {
            heads: [const {Mutex::new(FreeList {next: None})}; 13]
        }
    }
}

unsafe impl Sync for SlobAlloc {}

unsafe impl GlobalAlloc for SlobAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.align() > MM_BYTES_PER_PAGE {
            panic!("alloc: too large alignment value");
        }

        if layout.size() >= MM_BYTES_PER_PAGE {
            let pages = (layout.size() + MM_BYTES_PER_PAGE - 1) / MM_BYTES_PER_PAGE;
            unsafe {
                return (*PAGE_ALLOCATOR.get()).alloc(pages);
            }
        }

        // layout size < MM_BYTES_PER_PAGE
        let alloc_size = usize::max(layout.size(), layout.align()).max(16);
        let idx = core::mem::size_of::<usize>() - (alloc_size - 1).leading_zeros() as usize;
        let mut head = self.heads[idx].lock();

        if head.next.is_none() {
            unsafe {
                head.next = alloc_freelist(&mut *PAGE_ALLOCATOR.get(), 1<<idx);
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
        if layout.align() > MM_BYTES_PER_PAGE {
            panic!("alloc: too large alignment value");
        }

        if layout.size() >= MM_BYTES_PER_PAGE {
            let pages = (layout.size() + MM_BYTES_PER_PAGE - 1) / MM_BYTES_PER_PAGE;
            unsafe {
                (*PAGE_ALLOCATOR.get()).dealloc(ptr, pages);
            }
        }

        // layout size < MM_BYTES_PER_PAGE
        let alloc_size = usize::max(layout.size(), layout.align()).max(16);
        let idx = core::mem::size_of::<usize>() - (alloc_size - 1).leading_zeros() as usize;
        let mut head = self.heads[idx].lock();

        unsafe {
            let ptr = ptr as *mut FreeList;
            (*ptr).next = head.next;
            head.next = Some(NonNull::new(ptr).unwrap());
        }
    }
}


use super::Locked;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    allocations: usize,
    next: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            allocations: 0,
            next: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.allocations = 0;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut alloc = self.lock();

        let alloc_start = super::align_up(alloc.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return null_mut(),
        };

        if alloc_end >= alloc.heap_end {
            null_mut()
        } else {
            alloc.next = alloc_end;
            alloc.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        let mut alloc = self.lock();
        alloc.allocations -= 1;
        if alloc.allocations == 0 {
            alloc.next = alloc.heap_start;
        }
    }
}
